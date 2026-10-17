import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { gzipSync } from "node:zlib";
import {
  inspectArchive,
  observeInstalledArchive,
  readArchive,
  revalidatePublicArchives,
  verifyInstalledArchive,
  type InstalledArchive,
} from "../../tools/support/release/consumer_recovery/archive.ts";
import {
  commandPlan,
  verifyConsumer,
} from "../../tools/support/release/consumer_recovery/consumer.ts";
import {
  fixture as policyFixture,
  sha256,
} from "../../tools/support/release/consumer_recovery/policy.ts";
import { effectiveDenoConfig } from "../../tools/support/release/consumer_recovery/runtime.ts";
import { custodyProbe } from "../../tools/support/release/consumer_recovery/oracles.ts";

type TarFile = { name: string; body?: string; type?: string; link?: string };
const manifest = JSON.stringify({ name: "@vizejs/native-linux-x64-gnu", version: "0.457.0" });
const fixture: TarFile[] = [
  { name: "package/package.json", body: manifest },
  { name: "package/lib/provider.node", body: "native provider bytes" },
  { name: "package/LICENSE", body: "license" },
];
const sri = (bytes: Uint8Array) => `sha512-${createHash("sha512").update(bytes).digest("base64")}`;

function archive(files: TarFile[], corruptChecksum = false) {
  const pieces: Uint8Array[] = [];
  for (const file of files) {
    const header = Buffer.alloc(512);
    const body = Buffer.from(file.body ?? "");
    header.write(file.name, 0, 100);
    header.write("0000644\0", 100);
    header.write("0000000\0", 108);
    header.write("0000000\0", 116);
    header.write(body.length.toString(8).padStart(11, "0") + "\0", 124);
    header.write("00000000000\0", 136);
    header.fill(32, 148, 156);
    header.write(file.type ?? "0", 156);
    if (file.link) header.write(file.link, 157, 100);
    header.write("ustar\0", 257);
    header.write("00", 263);
    const checksum = header.reduce((total, value) => total + value, 0);
    header.write(checksum.toString(8).padStart(6, "0") + "\0 ", 148);
    if (corruptChecksum) header[0] ^= 1;
    pieces.push(header, body, Buffer.alloc((512 - (body.length % 512)) % 512));
  }
  pieces.push(Buffer.alloc(1024));
  return gzipSync(Buffer.concat(pieces));
}

function record(directory = "/tmp/public-install/provider"): InstalledArchive {
  const bytes = archive(fixture);
  const inspected = inspectArchive(bytes, sri(bytes));
  const native = inspected.archiveEntries.find((entry) => entry.path === "lib/provider.node")!;
  return {
    ...inspected,
    name: "@vizejs/native-linux-x64-gnu",
    version: "0.457.0",
    directory,
    integrity: sri(bytes),
    tarball:
      "https://registry.npmjs.org/@vizejs/native-linux-x64-gnu/-/native-linux-x64-gnu-0.457.0.tgz",
    manifestSha256: inspected.archiveManifest.sha256,
    installedFiles: structuredClone(inspected.archiveEntries),
    native: { path: `${directory}/lib/provider.node`, sha256: native.sha256 },
  };
}

test("public archive files, manifest and actual provider bytes form a complete offline observation", () => {
  const packet = record();
  verifyInstalledArchive(packet);
  assert.equal(packet.archiveEntries.length, 3);
  assert.equal(packet.archiveManifest.content, manifest);
  assert.equal(packet.archiveEntries[0].path, "lib/provider.node");
});

test("installed shipped files are read and compared to the verified archive", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-archive-files-"));
  try {
    for (const file of fixture) {
      const target = path.join(directory, file.name.slice(8));
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, file.body ?? "");
    }
    const packet = record(directory);
    Object.assign(packet, observeInstalledArchive(directory, packet.archiveEntries));
    verifyInstalledArchive(packet);
    fs.writeFileSync(path.join(directory, "lib/provider.node"), "different provider!!!");
    assert.throws(() => observeInstalledArchive(directory, packet.archiveEntries));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

const mutations: Record<string, (value: InstalledArchive) => void> = {
  "arbitrary loaded native digest": (value) => {
    value.native!.sha256 = "0".repeat(64);
  },
  "foreign loaded provider": (value) => {
    value.native!.path = "/tmp/other/provider.node";
  },
  "native path alias": (value) => {
    value.native!.path = value.directory + "/lib/../lib/provider.node";
  },
  "empty installed file observation": (value) => {
    value.installedFiles = [];
  },
  "missing provider file": (value) => {
    value.installedFiles = value.installedFiles.filter((file) => !file.path.endsWith(".node"));
  },
  "altered installed manifest": (value) => {
    value.manifestSha256 = "0".repeat(64);
  },
  "altered raw manifest": (value) => {
    value.archiveManifest.content = manifest.replace("0.457.0", "0.458.0");
  },
  "foreign package version": (value) => {
    value.version = "0.458.0";
  },
  "foreign package name": (value) => {
    value.name = "@vizejs/native-win32-x64-msvc";
  },
  "SRI substitution": (value) => {
    value.integrity = "sha512-" + "A".repeat(86) + "==";
  },
  "archive duplicate alias": (value) => {
    value.archiveEntries.push({ ...value.archiveEntries[0], path: "LIB/provider.node" });
  },
};
for (const [name, mutate] of Object.entries(mutations)) {
  test(`offline archive verification rejects ${name}`, () => {
    const packet = record();
    mutate(packet);
    assert.throws(() => verifyInstalledArchive(packet));
  });
}

test("offline Windows provider paths are checked on a POSIX verifier host", () => {
  const packet = record("C:\\runner-temp\\public-install\\provider");
  packet.native!.path = "C:\\runner-temp\\public-install\\provider\\lib\\provider.node";
  verifyInstalledArchive(packet);
  packet.native!.path = "C:\\runner-temp\\other\\provider.node";
  assert.throws(() => verifyInstalledArchive(packet));
});

for (const file of [
  { name: "package/../escape" },
  { name: "package/lib\\escape" },
  { name: "/package/absolute" },
  { name: "package/lib/FILE." },
  { name: "package/lib/NUL.txt" },
  { name: "package/link", type: "2", link: "package/package.json" },
  { name: "package/link", type: "1", link: "package/package.json" },
]) {
  test(`archive parser rejects unsafe TAR member ${JSON.stringify(file)}`, () => {
    const bytes = archive([...fixture, file]);
    assert.throws(() => inspectArchive(bytes, sri(bytes)));
  });
}

test("archive parser rejects duplicate names, aliases, file/directory conflicts and checksum errors", () => {
  for (const extra of [
    [{ name: "package/LICENSE" }],
    [{ name: "package/license" }],
    [{ name: "package/lib" }],
    [{ name: "package/LICENSE/subfile" }],
  ]) {
    const bytes = archive([...fixture, ...extra]);
    assert.throws(() => inspectArchive(bytes, sri(bytes)));
  }
  const corrupt = archive(fixture, true);
  assert.throws(() => inspectArchive(corrupt, sri(corrupt)), /checksum/u);
  const valid = archive(fixture);
  assert.throws(() => inspectArchive(valid, sri(archive(fixture.slice(0, 1)))), /SRI/u);
});

function paxPath(name: string) {
  const tail = ` path=${name}\n`;
  let length = Buffer.byteLength(tail) + 1;
  while (length !== String(length).length + Buffer.byteLength(tail))
    length = String(length).length + Buffer.byteLength(tail);
  return `${length}${tail}`;
}

test("GNU and PAX long names are accepted only after normal path validation", () => {
  const long = "package/" + "long/".repeat(30) + "file.txt";
  for (const extension of [
    { name: "././@LongLink", type: "L", body: long + "\0" },
    { name: "PaxHeader", type: "x", body: paxPath(long) },
  ]) {
    const bytes = archive([...fixture, extension, { name: "placeholder", body: "long body" }]);
    const inspected = inspectArchive(bytes, sri(bytes));
    assert.ok(inspected.archiveEntries.some((entry) => entry.path === long.slice(8)));
  }
  const malicious = archive([
    ...fixture,
    { name: "pax", type: "x", body: paxPath("package/../../escape") },
    { name: "placeholder" },
  ]);
  assert.throws(() => inspectArchive(malicious, sri(malicious)));
});

test("installed archive reader rejects symlink substitution even when bytes match", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-archive-link-"));
  try {
    const directory = path.join(root, "package");
    fs.mkdirSync(directory);
    for (const file of fixture.filter((value) => value.name !== "package/package.json")) {
      const target = path.join(directory, file.name.slice(8));
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, file.body ?? "");
    }
    fs.writeFileSync(path.join(root, "manifest.json"), manifest);
    fs.symlinkSync(path.join(root, "manifest.json"), path.join(directory, "package.json"));
    assert.throws(() => observeInstalledArchive(directory, record().archiveEntries));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("archive network reader rejects nonpublic origins before any request", async () => {
  const integrity = record().integrity;
  for (const url of [
    "https://example.com/file.tgz",
    "http://registry.npmjs.org/file.tgz",
    "https://user@registry.npmjs.org/file.tgz",
    "https://registry.npmjs.org/file.tgz?override=1",
  ]) {
    await assert.rejects(readArchive(url, integrity));
  }
});

function publicArchive() {
  const bytes = archive(fixture);
  const packet = record();
  const dependencies = {
    metadata: async () => ({
      name: packet.name,
      version: packet.version,
      dist: { tarball: packet.tarball, integrity: packet.integrity },
    }),
    inspect: async (_url: string, integrity: string) => inspectArchive(bytes, integrity),
  };
  return { packet, dependencies };
}

test("independent public revalidation recomputes archive bytes and rejects coherent forged observations", async () => {
  const { packet, dependencies } = publicArchive();
  const observed = await revalidatePublicArchives([packet], packet.native!, dependencies);
  assert.equal(observed[0].archiveManifest.name, packet.name);
  const forged = structuredClone(packet);
  const entry = forged.archiveEntries.find((value) => value.path.endsWith(".node"))!;
  entry.sha256 = "f".repeat(64);
  forged.installedFiles = structuredClone(forged.archiveEntries);
  forged.native!.sha256 = entry.sha256;
  verifyInstalledArchive(forged); // Self-consistent signed observations still need independent source bytes.
  await assert.rejects(
    revalidatePublicArchives([forged], forged.native!, dependencies),
    /actual public archive entries/u,
  );
});

test("independent public revalidation rejects SRI, URL, provider and empty-set substitutions", async () => {
  for (const mutate of [
    (packet: InstalledArchive) => {
      packet.integrity = "sha512-" + "A".repeat(86) + "==";
    },
    (packet: InstalledArchive) => {
      packet.tarball = "https://example.com/provider.tgz";
    },
    (packet: InstalledArchive) => {
      packet.native!.path = "/tmp/foreign/provider.node";
    },
    (packet: InstalledArchive) => {
      packet.native!.sha256 = "a".repeat(64);
    },
  ]) {
    const { packet, dependencies } = publicArchive();
    const changed = structuredClone(packet);
    mutate(changed);
    await assert.rejects(revalidatePublicArchives([changed], changed.native!, dependencies));
  }
  const { packet, dependencies } = publicArchive();
  await assert.rejects(revalidatePublicArchives([], packet.native!, dependencies));
  await assert.rejects(revalidatePublicArchives([packet, packet], packet.native!, dependencies));
});

test("independent public revalidation rejects a coherently rewritten manifest observation", async () => {
  const { packet, dependencies } = publicArchive();
  const content = JSON.stringify({ ...JSON.parse(packet.archiveManifest.content), forged: true });
  const digest = createHash("sha256").update(content).digest("hex");
  Object.assign(packet.archiveManifest, {
    content,
    bytes: Buffer.byteLength(content),
    sha256: digest,
  });
  Object.assign(
    packet.archiveEntries.find((entry) => entry.path === "package.json")!,
    { bytes: Buffer.byteLength(content), sha256: digest },
  );
  packet.installedFiles = structuredClone(packet.archiveEntries);
  packet.manifestSha256 = digest;
  verifyInstalledArchive(packet);
  await assert.rejects(
    revalidatePublicArchives([packet], packet.native!, dependencies),
    /actual public archive entries/u,
  );
});

function completeConsumer() {
  const project = "/tmp/vize-real-archive-control";
  const head = "b".repeat(40);
  const packages = [
    "vize",
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "@vizejs/vite-plugin",
  ].map((name) => {
    const files = fixture.map((file) =>
      file.name === "package/package.json"
        ? { ...file, body: JSON.stringify({ name, version: "0.457.0" }) }
        : file,
    );
    const bytes = archive(files);
    const inspected = inspectArchive(bytes, sri(bytes));
    const observed = {
      ...inspected,
      name,
      version: "0.457.0",
      directory: `${project}/node_modules/${name}`,
      integrity: sri(bytes),
      tarball: `https://registry.npmjs.org/${name}/-/fixture.tgz`,
      manifestSha256: inspected.archiveManifest.sha256,
      installedFiles: structuredClone(inspected.archiveEntries),
      source: head,
      published: "2026-10-15T00:00:00.000Z",
    };
    return { bytes, observed };
  });
  const provider = packages.find(
    (value) => value.observed.name === "@vizejs/native-linux-x64-gnu",
  )!.observed;
  const policy = {
    ...policyFixture,
    sha: "c".repeat(40),
    custodyProbeSha256: sha256(custodyProbe),
  };
  const packet = {
    schema: "vize-consumer-recovery-consumer-v1",
    status: "passed",
    role: "node24-linux" as const,
    identity: {
      cut: "a".repeat(40),
      head,
      tag: "v0.457.0",
      sourcePr: "9196",
      run: "38805261385",
      attempt: 4,
    },
    policy,
    consumerSha256: policy.consumerSha256,
    denoConfigSha256: policy.denoConfigSha256,
    effectiveDenoConfigSha256: sha256(effectiveDenoConfig),
    run: { id: "123456789", attempt: "1", sha: policy.sha },
    project,
    checkout: "/work/vize",
    output: "/tmp/consumer-output",
    nodeExecutable: "/opt/node/bin/node",
    runtime: { runtime: "node" as const, node: "24" as const, platform: "linux" as const },
    commands: [] as Parameters<typeof commandPlan>[0]["commands"],
    packages: packages.map((value) => value.observed),
    installed: {
      native: {
        path: `${provider.directory}/lib/provider.node`,
        loaded: true,
        sha256: provider.archiveEntries.find((value) => value.path.endsWith(".node"))!.sha256,
      },
      packages: packages.map(({ observed }) => ({
        name: observed.name,
        version: observed.version,
        directory: observed.directory,
        manifestSha256: observed.manifestSha256,
      })),
      runtime: { node: "v24.19.0", platform: "linux", arch: "x64" },
    },
    jsr: { version: "0.457.0", published: "2026-10-15T00:00:00.000Z" },
  };
  const oracle = { "consumer.ts": policy.consumerSha256, "custody.ts": policy.custodyProbeSha256 };
  packet.commands = commandPlan(packet).map(([label, command, args]) => ({
    label,
    command,
    args,
    cwd: project,
    began: "2026-10-17T00:00:00.000Z",
    ended: "2026-10-17T00:00:01.000Z",
    status: 0,
    signal: null,
    stdout:
      label === "node-version"
        ? "v24.19.0\n"
        : label === "consumer"
          ? "Verified @vizejs/vize (--vite)\n"
          : "",
    stderr: "",
    denoDir: null,
    oracleBefore: { ...oracle },
    oracleAfter: { ...oracle },
  }));
  const dependencies = {
    metadata: async (name: string) => {
      const record = packages.find((value) => value.observed.name === name)!.observed;
      return {
        name,
        version: record.version,
        dist: { tarball: record.tarball, integrity: record.integrity },
      };
    },
    inspect: async (url: string, integrity: string) =>
      inspectArchive(packages.find((value) => value.observed.tarball === url)!.bytes, integrity),
  };
  return { packet, dependencies };
}

test("complete consumer packet rejects detached native, empty installed packages and manifest substitutions", () => {
  const { packet } = completeConsumer();
  verifyConsumer(packet);
  const mutations = [
    (value: typeof packet) => {
      value.installed.native.sha256 = "a".repeat(64);
    },
    (value: typeof packet) => {
      value.installed.packages = [];
    },
    (value: typeof packet) => {
      value.installed.packages[0].manifestSha256 = "f".repeat(64);
    },
    (value: typeof packet) => {
      value.packages[0].manifestSha256 = "e".repeat(64);
    },
    (value: typeof packet) => {
      value.commands[0].oracleAfter!["consumer.ts"] = "0".repeat(64);
    },
  ];
  for (const mutate of mutations) {
    const changed = structuredClone(packet);
    mutate(changed);
    assert.throws(() => verifyConsumer(changed));
  }
});

test("complete consumer's coherently forged native observation fails independent public archive inspection", async () => {
  const { packet, dependencies } = completeConsumer();
  const provider = packet.packages.find((value) => value.name === "@vizejs/native-linux-x64-gnu")!;
  provider.archiveEntries.find((value) => value.path.endsWith(".node"))!.sha256 = "e".repeat(64);
  provider.installedFiles = structuredClone(provider.archiveEntries);
  packet.installed.native.sha256 = "e".repeat(64);
  verifyConsumer(packet);
  await assert.rejects(
    revalidatePublicArchives(packet.packages, packet.installed.native, dependencies),
    /actual public archive entries/u,
  );
});

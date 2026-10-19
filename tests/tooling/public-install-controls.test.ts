// Inert rejection controls. No public installation, native success or Corsa execution.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { describe, test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  archiveComparison,
  recheckFiles,
} from "../../tools/support/release/public_install/archive.ts";
import {
  exactPath,
  freshOutputs,
  overrides,
  producerAuthority,
  producerFiles,
  rejectOverrides,
} from "../../tools/support/release/public_install/identity.ts";
import { verifyJournal } from "../../tools/support/release/public_install/probe.ts";
import {
  packageMetadata,
  provenancePayloadBinding,
  publicUrl,
  sriSha512,
} from "../../tools/support/release/public_install/registry.ts";
import { writeTarGz, type TarEntry } from "../../tools/support/archive/tar.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
for (const key of [...overrides, "VIZE_PUBLIC_NATIVE_CUSTODY", "VIZE_OXLINT_NATIVE_CUSTODY"])
  delete process.env[key];
globalThis.fetch = () => {
  throw new Error("inert controls must not reach the network");
};

const temporaryDirectory = (prefix: string): string =>
  fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), prefix)));
const integrityOf = (payload: Buffer): string =>
  `sha512-${createHash("sha512").update(payload).digest("base64")}`;

describe("ArchiveControls", () => {
  const files: Readonly<Record<string, string>> = {
    "package.json": "{}",
    "dist/cli.mjs": "inert JavaScript routing",
    "native-targets.js": "inert target map",
    "vize.node": "not a native library",
    "lib/tsc": "inert Corsa launcher",
  };
  const setUp = (t: { after: (fn: () => void) => void }): string => {
    const temporary = temporaryDirectory("public-install-archive-");
    t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
    const packageDir = path.join(temporary, "package");
    for (const [name, data] of Object.entries(files)) {
      fs.mkdirSync(path.dirname(path.join(packageDir, name)), { recursive: true });
      fs.writeFileSync(path.join(packageDir, name), data);
    }
    return packageDir;
  };
  const entries = (): TarEntry[] =>
    Object.entries(files).map(([name, data]) => ({
      name: `package/${name}`,
      data: Buffer.from(data),
    }));
  const packet = (members: readonly TarEntry[] = entries()): [Buffer, string] => {
    const payload = writeTarGz(members);
    return [payload, integrityOf(payload)];
  };
  const rejected = (packageDir: string, members?: readonly TarEntry[]): void => {
    assert.throws(() => archiveComparison(packageDir, ...packet(members)));
  };

  test("entire inert file set is compared", (t) => {
    const packageDir = setUp(t);
    const receipt = archiveComparison(packageDir, ...packet());
    assert.equal(receipt.fileCount, 5);
    assert.deepEqual(
      new Set(receipt.files.map((item) => path.relative(packageDir, item.path))),
      new Set(Object.keys(files)),
    );
  });

  test("tampered JS, native targets, native and Corsa are rejected", (t) => {
    const packageDir = setUp(t);
    for (const name of ["dist/cli.mjs", "native-targets.js", "vize.node", "lib/tsc"]) {
      fs.writeFileSync(path.join(packageDir, name), "tampered");
      rejected(packageDir);
      fs.writeFileSync(path.join(packageDir, name), files[name] ?? "");
    }
  });

  test("missing, extra, wrong and duplicate archive files are rejected", (t) => {
    const packageDir = setUp(t);
    const all = entries();
    rejected(packageDir, all.slice(0, -1));
    rejected(packageDir, [...all, { name: "package/missing.js", data: Buffer.from("extra") }]);
    rejected(packageDir, [...all, ...all.slice(0, 1)]);
    rejected(
      packageDir,
      all.map((entry) => ({ ...entry, name: entry.name.replace(/^package\//u, "wrong/") })),
    );
    fs.writeFileSync(path.join(packageDir, "extra.js"), "extra installed file");
    rejected(packageDir);
  });

  test("path traversal and redirects are rejected", (t) => {
    const packageDir = setUp(t);
    for (const name of [
      "package/../escape",
      "/package/package.json",
      "package/./package.json",
      "package//package.json",
      "package\\package.json",
    ])
      rejected(packageDir, [{ name, data: Buffer.from("inert") }]);
    const target = path.join(packageDir, "dist/cli.mjs");
    fs.unlinkSync(target);
    fs.symlinkSync(path.join(packageDir, "native-targets.js"), target);
    rejected(packageDir);
  });

  test("tar symlink and SRI mismatch are rejected", (t) => {
    const packageDir = setUp(t);
    const payload = writeTarGz([
      { name: "package/redirect", typeflag: "2", linkname: "../outside" },
    ]);
    assert.throws(() => archiveComparison(packageDir, payload, integrityOf(payload)));
    const forged = `sha512-${Buffer.alloc(64, "0").toString("base64")}`;
    assert.throws(() => archiveComparison(packageDir, packet()[0], forged));
  });

  test("post-probe byte and file-set mutations are rejected", (t) => {
    const packageDir = setUp(t);
    const receipt = { packageDirectory: packageDir, ...archiveComparison(packageDir, ...packet()) };
    fs.writeFileSync(path.join(packageDir, "lib/tsc"), "mutation after archive verification");
    assert.throws(() => recheckFiles([receipt]));
    fs.writeFileSync(path.join(packageDir, "lib/tsc"), files["lib/tsc"] ?? "");
    fs.writeFileSync(path.join(packageDir, "new.js"), "later file");
    assert.throws(() => recheckFiles([receipt]));
  });
});

describe("RegistryControls", () => {
  const source = { C: "c".repeat(40), H: "a".repeat(40), tag: "v0.999.0", R: "123" };
  const dist = {
    integrity: `sha512-${Buffer.alloc(64, "x").toString("base64")}`,
    attestations: { url: "https://registry.npmjs.org/-/npm/v1/attestations/inert" },
  };
  const payload = () => ({
    predicateType: "https://slsa.dev/provenance/v1",
    subject: [{ digest: { sha512: Buffer.alloc(64, "x").toString("hex") } }] as unknown[],
    predicate: {
      buildDefinition: {
        externalParameters: {
          workflow: {
            ref: "refs/heads/release/v0.999.0",
            repository: "https://github.com/ubugeeei-prod/vize",
            path: ".github/workflows/release.yml",
          },
        },
        resolvedDependencies: [{ digest: { gitCommit: "a".repeat(40) } }],
      },
      runDetails: {
        metadata: {
          invocationId: "https://github.com/ubugeeei-prod/vize/actions/runs/123/attempts/1",
        },
      },
    },
  });
  const packet = (value: unknown = payload()) => ({
    attestations: [
      {
        bundle: {
          dsseEnvelope: { payload: Buffer.from(JSON.stringify(value)).toString("base64") },
        },
      },
    ],
  });
  const entry = {
    resolved: "https://registry.npmjs.org/vize/-/vize-0.999.0.tgz",
    integrity: dist.integrity,
  };

  test("payload binding explicitly does not claim signature verification", async () => {
    const result = await provenancePayloadBinding(dist, source, async () => packet());
    assert.equal(result.payloadBindingChecked, true);
    assert.equal(result.cryptographicSignatureVerificationPerformed, false);
  });

  test("wrong source, run, workflow, subject and duplicate are rejected", async () => {
    for (const key of ["source", "run", "workflow", "subject"]) {
      const value = payload();
      if (key === "source")
        value.predicate.buildDefinition.resolvedDependencies[0] = {
          digest: { gitCommit: "b".repeat(40) },
        };
      else if (key === "run")
        value.predicate.runDetails.metadata.invocationId =
          "https://github.com/ubugeeei-prod/vize/actions/runs/124/attempts/1";
      else if (key === "workflow")
        value.predicate.buildDefinition.externalParameters.workflow.ref = "refs/heads/main";
      else value.subject = [];
      await assert.rejects(
        provenancePayloadBinding(dist, source, async () => packet(value)),
        key,
      );
    }
    const doubled = packet();
    doubled.attestations.push(...doubled.attestations);
    await assert.rejects(provenancePayloadBinding(dist, source, async () => doubled));
  });

  test("missing provenance and external URLs are rejected", async () => {
    await assert.rejects(
      provenancePayloadBinding(dist, source, async () => ({ attestations: [] })),
    );
    for (const url of [
      "http://registry.npmjs.org/a",
      "https://registry.npmjs.org.evil/a",
      "https://registry.npmjs.org:443/a",
      "https://user@registry.npmjs.org/a",
    ])
      assert.throws(() => publicUrl(url));
    for (const sri of [
      "sha256-abc",
      "sha512-abc",
      `sha512-${Buffer.alloc(63, "x").toString("base64")}`,
    ])
      assert.throws(() => sriSha512(sri));
  });

  test("public metadata cannot change name, version, URL or SRI", async () => {
    for (const field of ["name", "version", "tarball", "integrity"] as const) {
      const metadata = {
        name: "vize",
        version: "0.999.0",
        dist: { tarball: entry.resolved, integrity: entry.integrity },
      };
      if (field === "name" || field === "version") metadata[field] = "different";
      else metadata.dist[field] = "different";
      await assert.rejects(packageMetadata("vize", "0.999.0", entry, source, async () => metadata));
    }
  });

  test("whole package and provenance reader binding is inert", async () => {
    const metadata = {
      name: "vize",
      version: "0.999.0",
      dist: { ...dist, tarball: entry.resolved },
    };
    const requests: string[] = [];
    const reader = async (url: string): Promise<unknown> => {
      requests.push(url);
      return url === dist.attestations.url ? packet() : metadata;
    };
    const receipt = await packageMetadata("vize", "0.999.0", entry, source, reader);
    assert.equal(receipt.payloadBindingChecked, true);
    assert.equal(receipt.cryptographicSignatureVerificationPerformed, false);
    assert.deepEqual(requests, ["https://registry.npmjs.org/vize/0.999.0", dist.attestations.url]);
    for (const changed of [
      { ...source, H: "b".repeat(40) },
      { ...source, R: "124" },
    ])
      await assert.rejects(packageMetadata("vize", "0.999.0", entry, changed, reader));
  });
});

describe("IdentityControls", () => {
  test("nine initial overrides are rejected before the Node probe", () => {
    for (const key of overrides)
      assert.throws(() => rejectOverrides({ [key]: "nonempty" }), new RegExp(key, "u"));
    rejectOverrides(Object.fromEntries(overrides.map((key) => [key, ""])));
  });

  test("output symlink and each existing receipt are rejected", (t) => {
    const parent = temporaryDirectory("public-install-outputs-");
    t.after(() => fs.rmSync(parent, { recursive: true, force: true }));
    for (const filename of freshOutputs(path.join(parent, "receipt.json"))) {
      fs.writeFileSync(filename, "preserved evidence");
      assert.throws(() => freshOutputs(path.join(parent, "receipt.json")));
      assert.equal(fs.readFileSync(filename, "utf8"), "preserved evidence");
      fs.unlinkSync(filename);
    }
    const link = path.join(parent, "link");
    fs.symlinkSync(parent, link);
    assert.throws(() => freshOutputs(path.join(link, "receipt.json")));
    assert.throws(() => exactPath(link, undefined, true));
  });

  test("reviewed collector digest is required", () => {
    const authority = producerAuthority(root);
    assert.deepEqual(producerAuthority(root, authority.sha256), authority);
    assert.throws(() => producerAuthority(root, "0".repeat(64)));
  });

  test("a copied collector prints the authority of its own reviewed bytes", (t) => {
    const copied = temporaryDirectory("public-install-copy-");
    t.after(() => fs.rmSync(copied, { recursive: true, force: true }));
    for (const name of producerFiles) {
      fs.mkdirSync(path.dirname(path.join(copied, name)), { recursive: true });
      fs.copyFileSync(path.join(root, name), path.join(copied, name));
    }
    const collector = path.join(copied, "tools/support/release/public_install/collect.ts");
    const result = spawnSync(process.execPath, [collector, "--print-collector-authority"], {
      encoding: "utf8",
      timeout: 30_000,
    });
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(JSON.parse(result.stdout), producerAuthority(copied));
  });
});

describe("JournalControls", () => {
  test("attempt, failed, wrong PID and Corsa never grant return credit", (t) => {
    const temporary = temporaryDirectory("public-install-journal-");
    t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
    const journal = path.join(temporary, "inert-journal.jsonl");
    const configuration = {
      installRoot: "/inert/install",
      nativePath: "/inert/install/vize.node",
      nativeSha256: "a".repeat(64),
    };
    const common = { schema: "vize-public-native-custody-event-v1", pid: 123 };
    const initialized = {
      ...common,
      event: "initialized",
      ...configuration,
      sha256: "a".repeat(64),
      node: "/inert/node",
    };
    const attempt = {
      ...common,
      event: "attempt",
      expectedNative: true,
      actualPath: configuration.nativePath,
      sha256: "a".repeat(64),
    };
    const wrongRoute = { ...attempt, event: "returned", corsaPath: "/source/tsgo" };
    for (const events of [
      [initialized, attempt],
      [initialized, attempt, { ...attempt, event: "failed" }],
      [initialized, { ...attempt, pid: 999 }],
      [initialized, attempt, wrongRoute],
    ]) {
      fs.writeFileSync(journal, events.map((event) => `${JSON.stringify(event)}\n`).join(""));
      assert.throws(() =>
        verifyJournal(journal, configuration, "/inert/node", 123, "/inert/corsa"),
      );
    }
  });
});

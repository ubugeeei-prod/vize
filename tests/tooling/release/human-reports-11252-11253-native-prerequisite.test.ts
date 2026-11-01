import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { readTarGz } from "../../../tools/support/archive/tar.ts";
import { capturePublicPayload } from "../../../tools/support/release/human_reports_11252_11253_public/capture-payload.ts";
import {
  probeAfterNativePrerequisite,
  readNativeHostInput,
  recheckNativeLoadPrerequisite,
  validateNativeLoadRecord,
  type NativeHostInput,
} from "../../../tools/support/release/human_reports_11252_11253_public/native-loader-prerequisite.ts";
import { nativeLoaderProgram } from "../../../tools/support/release/human_reports_11252_11253_public/native-loader-program.ts";
import type { ReleaseAuthority } from "../../../tools/support/release/human_reports_11252_11253_public/model.ts";
import { sha256 } from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";
import { pkg } from "./human-reports-11252-11253-bin.fixture.ts";
import {
  programBodies,
  programSource,
} from "./human-reports-11252-11253-native-program.fixture.ts";

// Native-looking bytes are inert data. No require(), dlopen(), shell, manager,
// compiler, native probe or product command is executed by these controls.
function setup(localPublic = false, host = readNativeHostInput()) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "human-native-precheck-")));
  const slot = join(root, "node_modules/.pnpm/synthetic/node_modules");
  const coreDirectory = join(slot, "@vizejs/native"),
    cliDirectory = join(slot, "vize");
  const target =
    host.platform === "darwin"
      ? "darwin-" + host.arch
      : "linux-" + host.arch + (host.musl ? "-musl" : "-gnu");
  const packageName = "@vizejs/native-" + target,
    nodeName = "vize-vitrine." + target + ".node";
  const version = "0.627.0",
    image = Buffer.from([0, 19, 82, 7]);
  const programs = Object.fromEntries(
    Object.entries(programBodies).map(([name, body]) => [name, Buffer.from(body, "base64")]),
  );
  const declarations = Object.fromEntries(
    Array.from({ length: 10 }, (_, i) => ["types/sdk" + i + ".d.ts", Buffer.from("export {};\n")]),
  );
  const sources = [
    pkg(
      "@vizejs/native",
      version,
      coreDirectory,
      { name: "@vizejs/native", version, main: "index.js" },
      { ...programs, ...(localPublic ? { [nodeName]: image } : {}) },
    ),
    pkg(
      packageName,
      version,
      join(slot, packageName),
      { name: packageName, version, main: nodeName },
      { [nodeName]: image },
    ),
    pkg(
      "vize",
      version,
      cliDirectory,
      { name: "vize", version, bin: "bin/vize" },
      { "bin/vize": Buffer.from("// inert CLI entry\n"), ...declarations },
    ),
  ];
  for (const source of sources) {
    assert(source.archive);
    for (const member of readTarGz(source.archive)) {
      const path = join(source.directory, member.name.slice(8));
      mkdirSync(dirname(path), { recursive: true });
      writeFileSync(path, member.data);
    }
  }
  symlinkSync(cliDirectory, join(root, "node_modules/vize"));
  const events: unknown[] = [],
    sink = (kind: string, value: unknown) => events.push({ kind, value });
  const packages = sources.map((source) => capturePublicPayload(source, sink));
  const cli = packages[2]!;
  const sdk = cli.files.filter((file) => file.path.endsWith(".d.ts"));
  const run = "123",
    candidate = "a".repeat(40),
    token = "b".repeat(32);
  const receipt = Buffer.from(
    JSON.stringify({
      schema: "vize.platformVsix.v1",
      source: candidate,
      version,
      provider: {
        source: candidate,
        targetPlatform: host.platform + "-" + host.arch,
        releaseVersion: version,
        nodeFile: nodeName,
        nativeBuildToken: token,
        files: [{ path: nodeName, bytes: image.length, sha256: sha256(image) }],
      },
    }),
  );
  const authority: ReleaseAuthority = {
    version,
    candidate,
    run,
    source: "c".repeat(40),
    workflowHead: "d".repeat(40),
    sdk: [],
    observations: [],
    produced: null,
    capsule: {
      npmSRI: Object.fromEntries(
        sources.map((source) => [
          source.name,
          { integrity: source.integrity, tarball: source.resolved },
        ]),
      ),
      receipts: [
        { rawBase64: receipt.toString("base64"), sha256: sha256(receipt), origin: { runId: run } },
      ],
    },
  };
  const input = { root, packages, sdk, authority, env: {}, sink, host };
  let probes = 0,
    products = 0;
  const execute = () => {
    const result = probeAfterNativePrerequisite(input, () => {
      probes++;
      return "probe-spy-only";
    });
    products++;
    return result;
  };
  const write = (path: string, body = "UNKNOWN_CANDIDATE_SECRET_731") => {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, body);
  };
  return {
    ...input,
    coreDirectory,
    cliDirectory,
    slot,
    target,
    packageName,
    nodeName,
    image,
    events,
    execute,
    write,
    counts: () => ({ probes, products }),
    close: () => rmSync(root, { recursive: true, force: true }),
  };
}
test("whole reviewed source program is independently matched to synthetic public archive bytes before probe", () => {
  assert(nativeLoaderProgram.sources.includes(programSource));
  for (const row of nativeLoaderProgram.modules) {
    const raw = Buffer.from(programBodies[row.name]!, "base64");
    assert.equal(raw.length, row.bytes);
    assert.equal(sha256(raw), row.sha256);
  }
  const f = setup();
  try {
    const result = f.execute();
    assert.deepEqual(f.counts(), { probes: 1, products: 1 });
    assert.equal(result.result, "probe-spy-only");
    assert.equal(result.proof.selectedOwner, f.packageName);
    assert.equal(result.proof.accepted, false);
    assert.equal(result.proof.productQualified, false);
    assert.equal(result.proof.nonExecutingPrerequisiteOnly, true);
    assert(result.proof.checkedBases.includes(join(f.cliDirectory, "bin/vize")));
    assert(result.proof.checkedBases.includes(f.sdk[0]!.path));
  } finally {
    f.close();
  }
});
test("unknown prioritized, extensionless, nested, package and WASI shadows prevent both probe and product callback", () => {
  const mutations = [
    (f: ReturnType<typeof setup>) => f.write(join(f.coreDirectory, f.nodeName)),
    (f: ReturnType<typeof setup>) => f.write(join(f.coreDirectory, f.nodeName + ".js")),
    (f: ReturnType<typeof setup>) => f.write(join(f.coreDirectory, f.nodeName, "index.js")),
    (f: ReturnType<typeof setup>) => f.write(join(f.coreDirectory, "native-targets")),
    (f: ReturnType<typeof setup>) => f.write(join(f.coreDirectory, "native-binding.json")),
    (f: ReturnType<typeof setup>) =>
      f.write(join(f.cliDirectory, "bin/node_modules/@vizejs/native/index.js")),
    (f: ReturnType<typeof setup>) =>
      f.write(join(f.cliDirectory, "types/node_modules/@vizejs/native/index.js")),
    (f: ReturnType<typeof setup>) =>
      f.write(join(f.coreDirectory, "node_modules", f.packageName, "index.js")),
    (f: ReturnType<typeof setup>) => f.write(join(f.coreDirectory, "vize-vitrine.wasi.cjs")),
    (f: ReturnType<typeof setup>) => f.write(join(f.slot, "@vizejs/native-wasm32-wasi/index.js")),
    (f: ReturnType<typeof setup>) => f.write(join(f.coreDirectory, "native-targets.js")),
    (f: ReturnType<typeof setup>) =>
      f.write(join(f.coreDirectory, "package.json"), '{"name":"@vizejs/native","main":"extra.js"}'),
  ];
  for (const mutate of mutations) {
    const f = setup();
    try {
      mutate(f);
      assert.throws(f.execute);
      assert.deepEqual(f.counts(), { probes: 0, products: 0 });
      assert(!JSON.stringify(f.events).includes("UNKNOWN_CANDIDATE_SECRET_731"));
    } finally {
      f.close();
    }
  }
});
test("an earlier darwin-universal candidate is inspected even when a known arm64 image is available", () => {
  const host: NativeHostInput = {
    platform: "darwin",
    arch: "arm64",
    musl: false,
    classifier: "NOT_LINUX",
    filesystemSha256: null,
  };
  for (const name of [
    "vize-vitrine.darwin-universal.node",
    "vize-vitrine.darwin-universal.node.js",
  ]) {
    const f = setup(false, host);
    try {
      f.write(join(f.coreDirectory, name));
      assert.throws(f.execute);
      assert.deepEqual(f.counts(), { probes: 0, products: 0 });
    } finally {
      f.close();
    }
  }
});
test("authentic shipped local native image is permitted by public archive and producer proof rather than basename", () => {
  const f = setup(true);
  try {
    const { proof } = f.execute();
    assert.equal(proof.selectedNative.path, join(f.coreDirectory, f.nodeName));
    assert.equal(proof.selectedOwner, "@vizejs/native");
    assert.equal(proof.issueQualified, false);
  } finally {
    f.close();
  }
});
test("independently authenticated target symlink can pass PRE while final full-set verdict remains pending", () => {
  const f = setup();
  try {
    symlinkSync(join(f.slot, f.packageName, f.nodeName), join(f.coreDirectory, f.nodeName));
    const { proof } = f.execute();
    assert.equal(proof.selectedOwner, f.packageName);
    assert.equal(proof.productQualified, false);
  } finally {
    f.close();
  }
});
test("a later resolution shadow refuses on recheck despite an earlier cached Node resolve answer", () => {
  const f = setup();
  try {
    const { proof } = f.execute();
    const install = {
      root: f.root,
      packages: f.packages,
      sdk: f.sdk,
      nativeLoad: proof,
    };
    recheckNativeLoadPrerequisite(install, f.env, f.sink);
    f.write(join(f.coreDirectory, "native-targets"));
    assert.throws(() => recheckNativeLoadPrerequisite(install, f.env, f.sink));
    assert.deepEqual(f.counts(), { probes: 1, products: 1 });
  } finally {
    f.close();
  }
});
test("unknown native detector, override, false producer image and incomplete SDK refuse before probe", () => {
  const f = setup();
  try {
    const unknown: NativeHostInput = {
      platform: "linux",
      arch: "x64",
      musl: null,
      classifier: "UNKNOWN",
      filesystemSha256: null,
    };
    assert.throws(() =>
      probeAfterNativePrerequisite({ ...f, host: unknown }, () => assert.fail("probe reached")),
    );
    assert.throws(() =>
      probeAfterNativePrerequisite(
        { ...f, env: { NAPI_RS_NATIVE_LIBRARY_PATH: "/unknown/native.node" } },
        () => assert.fail("probe reached"),
      ),
    );
    assert.throws(() =>
      probeAfterNativePrerequisite({ ...f, sdk: f.sdk.slice(1) }, () =>
        assert.fail("probe reached"),
      ),
    );
    const authority = structuredClone(f.authority);
    authority.candidate = "0".repeat(40);
    assert.throws(() =>
      probeAfterNativePrerequisite({ ...f, authority }, () => assert.fail("probe reached")),
    );
    assert.deepEqual(f.counts(), { probes: 0, products: 0 });
  } finally {
    f.close();
  }
});
test("archived native proof joins selected native bytes and source-owned bases without rereading runner paths", () => {
  const f = setup();
  try {
    const { proof } = f.execute();
    const install: Parameters<typeof validateNativeLoadRecord>[0] = {
      root: f.root,
      platform: f.host.platform,
      arch: f.host.arch,
      packages: f.packages,
      sdk: f.sdk,
      nativeLoad: proof,
      native: {
        ...proof.selectedNative,
        packageName: proof.selectedOwner,
        token: proof.input.image.token,
        identityProbeOnly: true,
      },
    };
    f.close();
    validateNativeLoadRecord(install, f.authority);
    for (const mutate of [
      (row: typeof install) => {
        row.nativeLoad.selectedNative.sha256 = "f".repeat(64);
      },
      (row: typeof install) => {
        row.nativeLoad.checkedBases.pop();
      },
      (row: typeof install) => {
        row.nativeLoad.probeBase = "/old-runner/wrong-base";
      },
    ]) {
      const bad = structuredClone(install);
      mutate(bad);
      assert.throws(() => validateNativeLoadRecord(bad, f.authority));
    }
  } finally {
    f.close();
  }
});

import assert from "node:assert/strict";
import { runCiBuild } from "../../npm/native/scripts/build-ci.mjs";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import {
  ciRequestPath,
  validateCiRequest,
} from "../../npm/native/scripts/formatter-history-ci-request.mjs";
import {
  ciBuildCommand,
  validateCiBuildCommand,
  validateCiProfile,
  ciSourceIdentity,
  validateCiStreamCustody,
  validateCiArtifactCopies,
} from "../../npm/native/scripts/formatter-history-ci-build.mjs";
import {
  hash,
  bytes,
  nativeHistorySource,
  validateNativeHistoryBuild,
  emittedNativeArtifact,
} from "../../npm/native/scripts/formatter-history-build.mts";

void test("ci capture retains original one-build arguments; substituted commands fail", () => {
  const original =
    "napi build --platform --profile ci --manifest-path ../../crates/vize_vitrine/Cargo.toml -p vize_vitrine --features napi,legacy --output-dir .";
  assert.deepEqual(ciBuildCommand.slice(0, -3), original.split(" "));
  validateCiBuildCommand(ciBuildCommand);
  for (const [index, value] of [
    [0, "cargo"],
    [4, "debug"],
    [4, "release"],
    [6, "elsewhere/Cargo.toml"],
    [8, "another"],
    [10, "napi"],
    [12, ".artifacts/native"],
  ]) {
    const changed = [...ciBuildCommand];
    changed[index] = value;
    assert.throws(() => validateCiBuildCommand(changed));
  }
  assert.throws(() => validateCiBuildCommand(ciBuildCommand.slice(0, -1)));
  assert.throws(() => validateCiBuildCommand([...ciBuildCommand, "--release"]));
  assert.throws(() => emittedNativeArtifact(Buffer.alloc(0)));
  assert.throws(() =>
    emittedNativeArtifact(Buffer.from('{"reason":"build-finished","success":false}\n')),
  );
});

void test("capture intent binds the actual Actions run and exact source; it is never build authority", () => {
  // Controlled context models qualify request validation only, without any native execution.
  const source = { head: "a".repeat(40), tree: "b".repeat(40) };
  const context = { platform: "linux", githubActions: "true", runId: "38125005499" };
  const request = {
    schema: "vize.public-native-formatter-ci-build-request",
    version: 1,
    source,
    runId: context.runId,
  };
  assert.equal(validateCiRequest(request, source, context), request);
  for (const altered of [
    { ...request, schema: "pretend-build-receipt" },
    { ...request, version: 2 },
    { ...request, source: { ...source, head: "stale" } },
    { ...request, source: { ...source, tree: "altered" } },
    { ...request, source: { ...source, authoredChanges: "allowed" } },
    { ...request, runId: "other-run" },
    { ...request, buildPassed: true },
  ])
    assert.throws(() => validateCiRequest(altered, source, context));
  assert.throws(() => validateCiRequest(null, source, context));
  for (const altered of [
    { ...context, platform: "darwin" },
    { ...context, githubActions: undefined },
    { ...context, githubActions: "false" },
    { ...context, runId: undefined },
    { ...context, runId: "fabricated" },
  ])
    assert.throws(() => validateCiRequest(request, source, altered));
  assert.throws(() => validateNativeHistoryBuild(os.tmpdir(), request));
});
void test("ci authority rejects test, optimized, debug, and missing profiles", () => {
  const profile = {
    test: false,
    opt_level: "0",
    debuginfo: 0,
    debug_assertions: true,
    overflow_checks: true,
  };
  validateCiProfile(profile);
  for (const [key, value] of [
    ["test", true],
    ["opt_level", "3"],
    ["debuginfo", 2],
    ["debug_assertions", false],
    ["overflow_checks", false],
  ])
    assert.throws(() => validateCiProfile({ ...profile, [key]: value }));
  assert.throws(() => validateCiProfile({}));
  assert.throws(() =>
    validateNativeHistoryBuild("/missing", {
      schema: "vize.public-native-formatter-ci-build",
      version: 1,
    }),
  );
  assert.throws(() =>
    validateNativeHistoryBuild("/missing", {
      schema: "vize.public-native-formatter-build",
      version: 1,
      source: {},
    }),
  );
});
function scratch(run) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-ci-custody-"));
  try {
    run(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
void test("a capture request cannot silently select ordinary build mode when Task removes the flag", () =>
  scratch((nativeDir) => {
    // Missing native source/host authority must stop before either controlled process.
    const file = ciRequestPath(nativeDir);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, "{}\n");
    const previous = process.env.VIZE_NATIVE_CI_FORMATTER_HISTORY;
    delete process.env.VIZE_NATIVE_CI_FORMATTER_HISTORY;
    let count = 0;
    try {
      assert.throws(() =>
        runCiBuild(nativeDir, {
          run: () => {
            count++;
            return { status: 0 };
          },
        }),
      );
      assert.equal(count, 0, "a request requires strict admission before native execution");
    } finally {
      if (previous === undefined) delete process.env.VIZE_NATIVE_CI_FORMATTER_HISTORY;
      else process.env.VIZE_NATIVE_CI_FORMATTER_HISTORY = previous;
    }
  }));
void test("only the two actual generated outputs can differ; debug clean-source gate stays strict", () =>
  scratch((root) => {
    const files = {
      "Cargo.lock": "lock",
      "pnpm-lock.yaml": "lock",
      "crates/vize_vitrine/src/lib.rs": "vitrine",
      "crates/vize_glyph/src/lib.rs": "glyph",
      "npm/native/index.js": "js",
      "npm/native/index.d.ts": "types",
    };
    for (const [name, value] of Object.entries(files)) {
      const file = path.join(root, name);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, value);
    }
    const git = (...args) => execFileSync("git", args, { cwd: root, stdio: "pipe" });
    git("init");
    git("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "add", ".");
    git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "-m",
      "fixture",
    );
    const native = path.join(root, "npm/native");
    const original = nativeHistorySource(native);
    assert.deepEqual(ciSourceIdentity(native), original);
    fs.writeFileSync(path.join(native, "index.d.ts"), "entire generated output");
    const changed = ciSourceIdentity(native);
    assert.equal(changed.head, original.head);
    assert.notEqual(changed.workingDiff, original.workingDiff);
    assert.throws(() => nativeHistorySource(native));
    fs.writeFileSync(path.join(root, "crates/vize_vitrine/src/lib.rs"), "mutated producer");
    assert.throws(() => ciSourceIdentity(native), /authored source changed/);
    fs.writeFileSync(path.join(root, "crates/vize_vitrine/src/lib.rs"), "vitrine");
    fs.writeFileSync(path.join(native, "other.d.ts"), "not an exempt declaration");
    assert.throws(() => ciSourceIdentity(native));
  }));
void test("whole raw streams reject altered hashes, bodies and absent authority", () =>
  scratch((native) => {
    const dir = path.join(native, ".artifacts/native/formatter-history");
    fs.mkdirSync(dir, { recursive: true });
    const receipt = {
      process: { stdout: bytes(Buffer.from("cargo\n")), stderr: bytes(Buffer.alloc(0)) },
      synchronize: { stdout: bytes(Buffer.from("sync\n")), stderr: bytes(Buffer.alloc(0)) },
    };
    for (const { label, packet } of [
      { label: "cargo", packet: receipt.process },
      { label: "sync", packet: receipt.synchronize },
    ])
      for (const stream of ["stdout", "stderr"])
        fs.writeFileSync(
          path.join(dir, `${label}.${stream}.bin`),
          Buffer.from(packet[stream].base64, "base64"),
        );
    validateCiStreamCustody(native, receipt);
    const altered = structuredClone(receipt);
    altered.process.stdout.sha256 = "wrong";
    assert.throws(() => validateCiStreamCustody(native, altered));
    fs.writeFileSync(path.join(dir, "cargo.stdout.bin"), "different actual stream");
    assert.throws(() => validateCiStreamCustody(native, receipt));
    fs.rmSync(path.join(dir, "sync.stderr.bin"));
    assert.throws(() => validateCiStreamCustody(native, receipt));
  }));
void test("byte-custody guard rejects non-ELF, changed local/frozen copies and path substitution", () =>
  scratch((native) => {
    // These controlled bytes test admission negatives only; no Cargo or public build credit is claimed.
    const dir = path.join(native, ".artifacts/native/formatter-history");
    fs.mkdirSync(dir, { recursive: true });
    const data = Buffer.alloc(64);
    Buffer.from([0x7f, 0x45, 0x4c, 0x46]).copy(data);
    const filename = path.join(native, "emitted.so"),
      local = path.join(native, "vize-vitrine.fixture.node"),
      frozen = path.join(dir, "formatter-observe.node");
    for (const file of [filename, local, frozen]) fs.writeFileSync(file, data);
    const receipt = {
      emitted: { filename },
      generated: { name: path.basename(local), sha256: hash(data) },
      frozen: { path: frozen, sha256: hash(data) },
    };
    const badHash = structuredClone(receipt);
    badHash.generated.sha256 = "wrong";
    assert.throws(() => validateCiArtifactCopies(native, badHash));
    fs.writeFileSync(local, Buffer.alloc(64));
    assert.throws(() => validateCiArtifactCopies(native, receipt));
    fs.writeFileSync(local, data);
    fs.writeFileSync(frozen, Buffer.alloc(64));
    assert.throws(() => validateCiArtifactCopies(native, receipt));
    fs.writeFileSync(frozen, data);
    const badPath = structuredClone(receipt);
    badPath.frozen.path = filename;
    assert.throws(() => validateCiArtifactCopies(native, badPath));
    fs.writeFileSync(filename, Buffer.alloc(64));
    assert.throws(() => validateCiArtifactCopies(native, receipt));
    fs.rmSync(filename);
    assert.throws(() => validateCiArtifactCopies(native, receipt));
  }));

void test("ordinary ci wrapper runs the original native command once, then original synchronizer", () => {
  const calls = [];
  const nativeDir = "/controlled/native";
  assert.equal(
    runCiBuild(nativeDir, {
      capture: false,
      run: (command, args, options) => {
        calls.push({ command, args, cwd: options.cwd });
        return { status: 0 };
      },
    }),
    0,
  );
  assert.deepEqual(calls, [
    { command: "napi", args: ciBuildCommand.slice(1, -3), cwd: nativeDir },
    {
      command: process.execPath,
      args: [path.join(nativeDir, "scripts/sync-entrypoint.mjs")],
      cwd: nativeDir,
    },
  ]);
  let count = 0;
  assert.equal(
    runCiBuild(nativeDir, {
      capture: false,
      run: () => {
        count++;
        return { status: 7 };
      },
    }),
    7,
  );
  assert.equal(count, 1, "failed actual native command prevents synchronization");
});

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  bytes,
  emittedNativeArtifact,
  captureNativeHistoryBuild,
  nativeHistoryBuildEnvironment,
  validateNativeHistoryEnvironment,
  validateNativeHistoryCommand,
  validateNativeHistoryProfile,
} from "../../npm/native/scripts/formatter-history-build.mjs";

void test("captured native test builds change only debug information and retain exact Cargo custody", () => {
  const nativeDir = path.resolve("synthetic-native");
  const command = [
    "pnpm",
    "exec",
    "napi",
    "build",
    "--platform",
    "--manifest-path",
    "../../crates/vize_vitrine/Cargo.toml",
    "-p",
    "vize_vitrine",
    "--features",
    "napi,legacy",
    "--output-dir",
    path.join(nativeDir, ".artifacts/native"),
    "--no-js",
    "--",
    "--message-format=json-render-diagnostics",
    "--locked",
    "--config",
    'profile.dev.debug="line-tables-only"',
  ];
  validateNativeHistoryCommand(nativeDir, command);
  for (const [original, replacement] of [
    ["napi,legacy", "napi"],
    ["--no-js", "--release"],
    ["--locked", "--offline"],
    ['profile.dev.debug="line-tables-only"', "profile.dev.debug=0"],
    ['profile.dev.debug="line-tables-only"', "profile.dev.debug=2"],
  ]) {
    assert.throws(() =>
      validateNativeHistoryCommand(
        nativeDir,
        command.map((value) => (value === original ? replacement : value)),
      ),
    );
  }
  assert.throws(() => validateNativeHistoryCommand(nativeDir, command.slice(0, -2)));
  assert.throws(() => validateNativeHistoryCommand(nativeDir, [...command, "--release"]));
  const profile = {
    test: false,
    opt_level: "0",
    debuginfo: "line-tables-only",
    debug_assertions: true,
    overflow_checks: true,
  };
  validateNativeHistoryProfile(profile);
  for (const [key, value] of [
    ["test", true],
    ["opt_level", "1"],
    ["debuginfo", 0],
    ["debuginfo", 1],
    ["debuginfo", 2],
    ["debuginfo", "line-directives-only"],
    ["debug_assertions", false],
    ["overflow_checks", false],
  ])
    assert.throws(() => validateNativeHistoryProfile({ ...profile, [key]: value }));
  for (const key of Object.keys(profile)) {
    const missing = { ...profile };
    delete missing[key];
    assert.throws(() => validateNativeHistoryProfile(missing));
  }
});

// Synthetic admission controls only. Real emitted addon custody is proved by
// the mandatory existing hosted native build, not these generated vectors.
void test("Cargo admission requires genuine complete success and unambiguous typed native artifact", () => {
  const artifact = {
    reason: "compiler-artifact",
    target: { name: "vize_vitrine", kind: ["cdylib", "rlib"] },
    profile: { test: false },
    features: ["napi", "legacy"],
    filenames: ["/synthetic/libvize_vitrine.so"],
  };
  const complete = (rows) => Buffer.from(rows.map((row) => JSON.stringify(row)).join("\n") + "\n");
  const finished = { reason: "build-finished", success: true };
  assert.deepEqual(emittedNativeArtifact(complete([artifact, finished])), {
    cargo: artifact,
    filename: artifact.filenames[0],
  });
  for (const rows of [
    [artifact],
    [artifact, { ...finished, success: false }],
    [artifact, artifact, finished],
    [{ ...artifact, features: ["legacy"] }, finished],
    [{ ...artifact, filenames: ["/synthetic/one.so", "/synthetic/two.so"] }, finished],
    [{ ...artifact, target: { name: "vize_vitrine", kind: ["rlib"] } }, finished],
  ])
    assert.throws(() => emittedNativeArtifact(complete(rows)));
});

void test("failed build admission retains raw actual command/process streams before qualification", (t) => {
  const nativeDir = fs.mkdtempSync(path.join(os.tmpdir(), "native-build-custody-"));
  t.after(() => fs.rmSync(nativeDir, { recursive: true, force: true }));
  const stdout = Buffer.from("whole failed build stdout\0\n");
  const stderr = Buffer.from("whole failed build stderr\0\n");
  assert.throws(() =>
    captureNativeHistoryBuild(nativeDir, null, ["synthetic", "control"], {
      stdout,
      stderr,
      status: 1,
      signal: null,
      error: undefined,
    }),
  );
  const directory = path.join(nativeDir, ".artifacts/native/formatter-history");
  assert(fs.readFileSync(path.join(directory, "cargo.stdout.bin")).equals(stdout));
  assert(fs.readFileSync(path.join(directory, "cargo.stderr.bin")).equals(stderr));
  const frame = JSON.parse(fs.readFileSync(path.join(directory, "build-process.json")));
  assert.deepEqual(frame.command, ["synthetic", "control"]);
  assert.deepEqual(frame.stdout, bytes(stdout));
  assert.deepEqual(frame.stderr, bytes(stderr));
  assert.equal(frame.exitStatus, 1);
  assert(!fs.existsSync(path.join(directory, "build-receipt.json")));
});

void test("build environment custody compares the actual builder before and after, not a later observer", () => {
  const builder = { ...nativeHistoryBuildEnvironment(), CARGO_BUILD_JOBS: "12" };
  const observer = { ...builder, CARGO_BUILD_JOBS: null };
  assert.notDeepEqual(builder, observer);
  assert.deepEqual(validateNativeHistoryEnvironment({ before: builder, after: { ...builder } }), {
    before: builder,
    after: builder,
  });
  assert.throws(() => validateNativeHistoryEnvironment({ before: builder, after: observer }));
  assert.throws(() =>
    validateNativeHistoryEnvironment({ before: builder, after: { ...builder, extra: "unknown" } }),
  );
  assert.throws(() =>
    validateNativeHistoryEnvironment({ before: builder, after: { ...builder, CARGO: 12 } }),
  );
});

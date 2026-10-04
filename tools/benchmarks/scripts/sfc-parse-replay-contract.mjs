import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export const BASE_SHA = "1649cd0a40ec22aa887f15b1509a27de007ccf88";
export const HEAD_SHA = "947879a5427334b6f2524c92a5cd37a19fdc0b3d";
export const TOOLCHAIN = "1.98.0";
export const BENCH_PATH = "crates/vize_atelier_sfc/benches/sfc_parse.rs";
export const BENCH_SHA256 = "eda70eb21f390c6d8fe72a354534db97fbac2f1e3f906ee4e5295a11c328e1e8";
export const LOCK_SHA256 = {
  base: "70892dc5bc35bfa9acbce24d3ec3b1df5ecaf670c9e68f6febc48ad253b59eaa",
  head: "6ed7cf0b7a2436c1972fa0eea1c809a3f6e3ef0ed8a07a1f0ca713538492499b",
};
export const CASES = ["simple", "medium", "complex"];
export const SETTINGS = {
  pairs: 6,
  sampleSize: 100,
  warmupSeconds: 3,
  measurementSeconds: 5,
  resamples: 100_000,
  confidenceLevel: 0.95,
};
const CASE_ORDERS = [
  ["simple", "medium", "complex"],
  ["complex", "medium", "simple"],
  ["medium", "complex", "simple"],
  ["simple", "complex", "medium"],
  ["complex", "simple", "medium"],
  ["medium", "simple", "complex"],
];

export function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

export function validateRevisionInputs({ base, head, conflictingModes }) {
  assert.equal(base, BASE_SHA, "historical replay requires the frozen #6189 base");
  assert.equal(head, HEAD_SHA, "historical replay requires the frozen #6189 head");
  assert.equal(conflictingModes, false, "historical replay cannot select another route");
}

export function validateFrozenFiles(side, benchmark, lock, toolchain, manifest) {
  assert.ok(side === "base" || side === "head");
  assert.equal(sha256(benchmark), BENCH_SHA256, "benchmark inputs or timed body changed");
  assert.equal(sha256(lock), LOCK_SHA256[side], "historical locked dependencies changed");
  assert.match(toolchain, /channel\s*=\s*"1\.98\.0"/u);
  validateReleaseProfile(manifest);
}

export function validateReleaseProfile(manifest) {
  const release = manifest.match(/\[profile\.release\]([\s\S]*?)(?=\n\[|$)/u)?.[1];
  assert.ok(release, "release profile must be present");
  for (const line of [
    "opt-level = 3",
    'lto = "fat"',
    "codegen-units = 1",
    'panic = "abort"',
    "debug = false",
    "debug-assertions = false",
    "overflow-checks = false",
    "incremental = false",
  ])
    assert.ok(release.split("\n").includes(line), `release profile lost ${line}`);
}

export function validateBuildEnvironment(env) {
  assert.equal(env.RUSTUP_TOOLCHAIN, TOOLCHAIN);
  for (const key of Object.keys(env)) {
    assert.ok(!key.startsWith("CARGO_PROFILE_"), `unexpected profile override ${key}`);
  }
  for (const key of [
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_TARGET",
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTFLAGS",
  ]) {
    assert.ok(!env[key], `unexpected build override ${key}`);
  }
}

export function pairPlan() {
  return CASE_ORDERS.map((cases, pair) => ({
    pair,
    cases,
    sides: pair % 2 === 0 ? ["base", "head"] : ["head", "base"],
  }));
}

export function buildArgs(targetDir) {
  return [
    `+${TOOLCHAIN}`,
    "bench",
    "--locked",
    "--no-run",
    "--message-format=json",
    "-vv",
    "-p",
    "vize_atelier_sfc",
    "--bench",
    "sfc_parse",
    "--target-dir",
    targetDir,
  ];
}

export function parseBuiltExecutable(output) {
  // Cargo -vv also emits labelled build-script stdout. The complete stream is
  // retained; only Cargo's JSON message lines participate in artifact selection.
  const messages = output
    .split("\n")
    .filter((line) => line.startsWith("{"))
    .map((line) => JSON.parse(line));
  const finished = messages.filter((row) => row.reason === "build-finished");
  assert.deepEqual(finished, [{ reason: "build-finished", success: true }]);
  const artifacts = messages.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target.name === "sfc_parse" &&
      row.target.kind.includes("bench") &&
      row.executable,
  );
  assert.equal(artifacts.length, 1, "exactly one SFC parse executable is required");
  return artifacts[0].executable;
}

export function measureArgs(name) {
  assert.ok(CASES.includes(name), "only the three frozen SFC parse cases are measured");
  return [
    "--bench",
    "--noplot",
    "--sample-size",
    String(SETTINGS.sampleSize),
    "--warm-up-time",
    String(SETTINGS.warmupSeconds),
    "--measurement-time",
    String(SETTINGS.measurementSeconds),
    "--nresamples",
    String(SETTINGS.resamples),
    "--confidence-level",
    String(SETTINGS.confidenceLevel),
    "--exact",
    `sfc_parse/${name}`,
    "--save-baseline",
    "observed",
  ];
}

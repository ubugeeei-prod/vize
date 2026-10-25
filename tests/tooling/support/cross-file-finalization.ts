import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { files } from "./cross-file-finalization/corpus.ts";
import { hash, record, run, save, sourceProfile } from "./cross-file-finalization/capture.ts";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const output = path.join(repo, "target/vize-tests/cross-file-finalization");
const inherited = path.join(repo, "target/vize-tests/cross-file-sfc-effects");
fs.mkdirSync(output, { recursive: true });
const git = (...args: string[]) =>
  execFileSync("git", args, { cwd: repo, encoding: "utf8" }).trim();
const sha = git("rev-parse", "HEAD");
assert.equal(sha, process.env.EXPECTED_SOURCE_SHA);
assert.equal(
  git("status", "--porcelain", "--untracked-files=no", "--", "crates", "davinci", "Cargo.lock"),
  "",
);

// Reuse the same physical public provider already authenticated by the 13-state
// qualifier in this job. Recheck its loaded native image bytes before running.
const publicIdentity = record(
  JSON.parse(fs.readFileSync(path.join(inherited, "public-entrypoint.json"), "utf8")),
);
assert.equal(typeof publicIdentity.path, "string");
const publicBinary = publicIdentity.path as string;
assert.equal(hash(fs.readFileSync(publicBinary)), publicIdentity.sha256);
const nativeImages: unknown = JSON.parse(
  fs.readFileSync(path.join(inherited, "public-native-images.json"), "utf8"),
);
assert.ok(Array.isArray(nativeImages) && nativeImages.length === 1);
for (const image of nativeImages.map(record)) {
  assert.equal(typeof image.path, "string");
  assert.equal(hash(fs.readFileSync(image.path as string)), image.sha256);
}
for (const name of [
  "public-packages.json",
  "public-package-lock.json",
  "public-entrypoint.json",
  "public-native-images.json",
]) {
  fs.copyFileSync(path.join(inherited, name), path.join(output, name));
}
const sourceBinary = path.join(repo, "target/ci-opt/vize");
const sourceImage = hash(fs.readFileSync(sourceBinary));
const publicVersion = run(output, "public-version", publicBinary, ["--version"], repo);
assert.equal(publicVersion.stdout.toString("utf8"), "vize 0.565.0\n");
const sourceVersion = run(output, "source-version", sourceBinary, ["--version"], repo);
save(output, "source-build.json", {
  schema: "vize.cross-file.finalization.optimized-source",
  sourceSha: sha,
  binary: sourceBinary,
  binarySha256: sourceImage,
  version: sourceVersion.stdout.toString("utf8"),
  cargoLockSha256: hash(fs.readFileSync(path.join(repo, "Cargo.lock"))),
  rustc: execFileSync("rustc", ["--version", "--verbose"], { encoding: "utf8" }),
  cargo: execFileSync("cargo", ["--version"], { encoding: "utf8" }),
  command: ["cargo", "build", "--locked", "--profile", "ci-opt", "-p", "vize"],
  profile: { inherits: "release", lto: "thin", codegenUnits: 16 },
  sourceTrees: Object.fromEntries(
    ["crates/vize_croquis_cf/src", "crates/vize/src", "davinci/vize_l0"].map((name) => [
      name,
      git("rev-parse", `HEAD:${name}`),
    ]),
  ),
});

const project = path.join(output, "physical-project");
fs.mkdirSync(project, { recursive: true });
const inputs = Object.entries(files()).map(([name, source]) => {
  fs.writeFileSync(path.join(project, name), source);
  return { path: name, source, sha256: hash(source), bytes: Buffer.byteLength(source) };
});
assert.equal(inputs.length, 241);
save(output, "complete-inputs.json", inputs);
const flags = ["lint", "--no-config", "--cross-file-complexity", "--help-level", "short"];
const expectedJson = run(
  output,
  "public-json",
  publicBinary,
  [...flags, "--format", "json", "*.vue"],
  project,
);
const expectedPlain = run(
  output,
  "public-plain",
  publicBinary,
  [...flags, "--format", "plain", "*.vue"],
  project,
);
const sourceJson = run(
  output,
  "source-json",
  sourceBinary,
  [...flags, "--format", "json", "*.vue"],
  project,
);
assert.deepEqual(sourceJson.stdout, expectedJson.stdout, "complete 241-file diagnostic packet");
assert.equal(sourceJson.packet.status, expectedJson.packet.status);

const observations: unknown[] = [];
for (let ordinal = 0; ordinal < 9; ordinal++) {
  // Alternate enabled/disabled order. First two rounds are retained warmups;
  // setup, compilation and public-provider calls are excluded from observations.
  for (const enabled of ordinal % 2 ? [true, false] : [false, true]) {
    const label = `source-${ordinal}-${enabled ? "profiled" : "disabled"}`;
    const profileFlags = enabled
      ? ["--profile-json", path.join(output, label + ".profile.json")]
      : [];
    const actual = run(
      output,
      label,
      sourceBinary,
      [...flags, "--format", "plain", ...profileFlags, "*.vue"],
      project,
    );
    assert.deepEqual(
      actual.stdout,
      expectedPlain.stdout,
      label + ": complete report and diagnostics",
    );
    assert.equal(actual.packet.status, expectedPlain.packet.status);
    observations.push({
      ordinal,
      warmup: ordinal < 2,
      enabled,
      packet: actual.packet,
      sourcePhases: enabled ? sourceProfile(output, label) : null,
    });
    save(output, "complete-observations.json", observations);
  }
}
const optOut = run(
  output,
  "source-opt-out",
  sourceBinary,
  [
    "lint",
    "--no-config",
    "--format",
    "json",
    "--profile-json",
    path.join(output, "source-opt-out.profile.json"),
    "*.vue",
  ],
  project,
);
const beforeOptOut = run(
  output,
  "public-opt-out",
  publicBinary,
  ["lint", "--no-config", "--format", "json", "*.vue"],
  project,
);
assert.deepEqual(optOut.stdout, beforeOptOut.stdout);
assert.equal(optOut.packet.status, beforeOptOut.packet.status);
const offProfile = record(
  JSON.parse(fs.readFileSync(path.join(output, "source-opt-out.profile.json"), "utf8")),
);
assert.ok(Array.isArray(offProfile.spans) && Array.isArray(offProfile.counters));
assert.deepEqual(
  offProfile.spans
    .map(record)
    .filter((row) => String(row.key).startsWith("croquis_cf.dependencies.")),
  [],
);
assert.deepEqual(
  offProfile.counters
    .map(record)
    .filter((row) => String(row.key).startsWith("croquis_cf.dependencies.")),
  [],
);
assert.equal(
  hash(fs.readFileSync(sourceBinary)),
  sourceImage,
  "observed executable remained byte-identical",
);
save(output, "qualification.json", {
  passed: true,
  sourceSha: sha,
  sourceImage,
  files: inputs.length,
  measuredRounds: 7,
  warmupRounds: 2,
  observations,
  diagnosticsSha256: hash(sourceJson.stdout),
  reportSha256: hash(expectedPlain.stdout),
  note: "Actual optimized source-phase/call observations. Enabled and disabled whole-process timings are retained separately; no throughput improvement, internal WASM phase or instruction-budget claim.",
});
process.stdout.write(
  "cross-file dependency profiling: 241 physical SFCs and complete packets passed\n",
);

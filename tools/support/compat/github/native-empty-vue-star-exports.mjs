/** Run the actual emitted test binary once and retain complete source evidence. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { lstatSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const target = "empty_vue_star_exports";
const testNames = [
  "authentic_six_counterfactual_packets_and_provider_bytes_remain_complete",
  "configured_native_preserves_public_types_and_real_consumer_errors",
  "configured_native_refuses_erasure_at_unproved_source_boundaries",
];
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const regular = (file) => {
  const stat = lstatSync(file);
  assert(stat.isFile(), "whole regular file required: " + file);
  return readFileSync(file);
};
function git(root, args) {
  const run = spawnSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
  });
  assert.equal(run.error, undefined);
  assert.equal(run.status, 0, run.stderr);
  return run.stdout.trim();
}
export function validateArtifact(root, artifact) {
  assert.equal(artifact.target.name, target);
  assert.deepEqual(artifact.target.kind, ["test"]);
  assert(artifact.features.includes("native"), "required native feature absent");
  assert.equal(artifact.profile.test, true);
  assert.equal(artifact.profile.opt_level, "3");
  assert.equal(
    realpathSync(artifact.target.src_path),
    realpathSync(join(root, "crates/vize_canon/tests/empty_vue_star_exports/main.rs")),
  );
}
export function validateRun(root, capture, artifact, result) {
  validateArtifact(root, artifact);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  const stdout = result.stdout.toString("utf8");
  assert.deepEqual(Buffer.from(stdout), result.stdout, "test output must be complete UTF8");
  const passed = [...stdout.matchAll(/^test ([^\r\n]+) \.\.\. ok$/gmu)].map((row) => row[1]);
  assert.deepEqual(
    passed.toSorted((a, b) => a.localeCompare(b)),
    testNames.toSorted((a, b) => a.localeCompare(b)),
    "all three actual laws required",
  );
  assert.match(
    stdout,
    /test result: ok\. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out/u,
  );
  const corpus = join(root, "tests/_fixtures/differential/typechecker/empty-vue-star-exports");
  const supplement = JSON.parse(regular(join(corpus, "supplement.json")));
  assert.equal(supplement.cases.length, 16);
  assert.equal(new Set(supplement.cases).size, 16);
  const rows = [];
  for (const id of supplement.cases) {
    const bytes = regular(join(capture, "empty-vue-star-exports", id + ".json"));
    const packet = JSON.parse(bytes);
    const fixture = JSON.parse(regular(join(corpus, "production", id, "case.json")));
    assert.deepEqual(packet.case, fixture);
    const exact = fixture.nativeContract === "exact";
    assert.equal(
      packet.phase,
      exact
        ? "EXACT_AUTHORED_AND_SCOPED_RAW_ORACLES_ACCEPTED"
        : "EXACT_PROJECTION_REFUSAL_ACCEPTED_NATIVE_SEMANTICS_UNQUALIFIED",
    );
    assert.equal(packet.officialNativeCustody.phase, "OFFICIAL_NATIVE_RUNTIME_VALIDATED");
    const cli = packet.wholeSameConfigNativeCli;
    assert.equal(cli.nativeSHA256, packet.officialNativeCustody.wholeExecutableSHA256);
    assert.equal(cli.nativePath, packet.officialNativeCustody.executable);
    for (const stream of ["wholeStdoutBytes", "wholeStderrBytes"])
      assert(Array.isArray(cli[stream]), "complete raw native stream required");
    assert.equal(sha(Buffer.from(cli.wholeStdoutBytes)), cli.stdoutSha256);
    assert.equal(sha(Buffer.from(cli.wholeStderrBytes)), cli.stderrSha256);
    assert(packet.wholeGeneratedProgram.length > 0);
    for (const file of packet.wholeGeneratedProgram) {
      assert.equal(file.wholeCode, file.actualMaterializedRead.wholeUtf8);
      assert.equal(sha(Buffer.from(file.wholeCode)), file.generatedSHA256);
      assert.equal(typeof file.wholeSourceMap, "string");
      assert(file.wholeSourceMap.length > 0, "complete map representation required");
    }
    assert(packet.wholeNativeLoadedFiles.length > 0);
    assert(packet.actualNativeLoadedProviderRoles);
    if (exact) {
      assert(Array.isArray(packet.wholeMappedDiagnosticVector));
      assert.deepEqual(cli.wholeStderrBytes, []);
      assert.equal(packet.success, packet.wholeMappedDiagnosticVector.length === 0);
    }
    rows.push({ id, bytes: bytes.length, sha256: sha(bytes), contract: fixture.nativeContract });
  }
  assert.equal(rows.filter((row) => row.contract === "exact").length, 10);
  assert.equal(rows.filter((row) => row.contract === "projection-refusal-observation").length, 6);
  return rows;
}

function main() {
  const root = realpathSync(process.env.NATIVE_PHASE_SOURCE_ROOT);
  const capture = realpathSync(process.env.VIZE_NATIVE_SYNTAX_CAPTURE_DIR);
  assert.equal(process.env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(process.env.VIZE_TEST_DISABLE_TSGO, undefined);
  const source = git(root, ["rev-parse", "HEAD"]);
  assert.equal(source, process.env.SOURCE_SHA);
  assert.equal(git(root, ["status", "--porcelain", "--untracked-files=no"]), "");
  const builder = regular(join(capture, "builder.ndjson"));
  const artifacts = builder.toString("utf8").split(/\r?\n/u).filter(Boolean).map(JSON.parse);
  const selected = artifacts.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target.name === target &&
      row.executable &&
      row.profile.test,
  );
  assert.equal(selected.length, 1, "exactly one actual emitted test binary required");
  const artifact = selected[0];
  validateArtifact(root, artifact);
  const binary = realpathSync(artifact.executable);
  const args = ["--nocapture", "--test-threads=1"];
  const result = spawnSync(binary, args, {
    cwd: root,
    env: process.env,
    timeout: 600_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  writeFileSync(join(capture, "test.stdout"), stdout);
  writeFileSync(join(capture, "test.stderr"), stderr);
  const receipt = {
    source,
    tree: git(root, ["rev-parse", "HEAD^{tree}"]),
    root,
    artifact,
    binary,
    binarySHA256: sha(regular(binary)),
    argv: [binary, ...args],
    cwd: root,
    pid: result.pid,
    exitCode: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    builderSHA256: sha(builder),
    builderStderrSHA256: sha(regular(join(capture, "builder.stderr"))),
    stdoutSHA256: sha(stdout),
    stderrSHA256: sha(stderr),
    nativePIDAndNormalizedOptionsAuthority: "unobserved",
    sourceAndNativeAcceptance: "pending; exact failure evidence retained",
  };
  writeFileSync(join(capture, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  receipt.cases = validateRun(root, capture, artifact, result);
  receipt.sourceAndNativeAcceptance =
    "three actual laws; ten exact vectors and six projection-only observations";
  writeFileSync(join(capture, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();

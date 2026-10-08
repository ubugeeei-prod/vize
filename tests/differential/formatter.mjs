import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { compareBytes } from "./compare.mjs";
import { FORMATTER_ARGV, loadFormatterManifest, sha256 } from "./manifest.mjs";
import { expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.ts";
import { runPlannedCases, validateResultEnvelope } from "./harness.mjs";
import { expressionWidthReference } from "./formatter-expression-width-reference.mjs";
import { soleChildWidthCliReference } from "./formatter-sole-child-width-reference.mjs";

export function assertProcessSucceeded(result) {
  if (result.error) throw result.error;
  assert.equal(result.signal, null, `formatter terminated by ${result.signal}`);
  assert.equal(
    result.status,
    0,
    `formatter exited ${result.status}: ${result.stderr?.toString() ?? ""}`,
  );
}

const processObservation = (result) => ({
  exitStatus: result.status,
  signal: result.signal,
  stdoutBase64: result.stdout?.toString("base64") ?? "",
  stderrBase64: result.stderr?.toString("base64") ?? "",
  processError: result.error?.message ?? null,
});

export function formatterAttempt(pass, input, output, result, expected, currentExpected) {
  return {
    pass,
    inputBase64: input.toString("base64"),
    inputSha256: sha256(input),
    ...processObservation(result),
    outputBase64: output?.toString("base64") ?? null,
    outputSha256: output ? sha256(output) : null,
    referenceComparison: output ? compareBytes(expected, output) : null,
    ...(currentExpected
      ? { currentReferenceComparison: output ? compareBytes(currentExpected, output) : null }
      : {}),
  };
}

export function validateFormatterReport(loaded, report, expectedBuild) {
  validateResultEnvelope(loaded, report, expectedBuild.sourceRevision);
  assert.deepEqual(report.argv, FORMATTER_ARGV);
  if (report.buildReceipt) validateBuildReceipt(report.buildReceipt, expectedBuild);
  if (report.binary) {
    assert(report.buildReceipt, "executable observation requires a source-build receipt");
    assert.equal(report.binary.path, expectedBuild.binaryPath);
    assert.equal(report.binary.sha256, expectedBuild.binarySha256);
    assert.equal(report.binary.version, expectedBuild.cliVersion);
  }
  const rows = new Map(report.rows.map((row) => [row.id, row]));
  const summary = {
    plannedCases: loaded.cases.length,
    legacyMatches: 0,
    legacyFailures: 0,
    baselineDrift: 0,
    nativeUnsupported: loaded.cases.length,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
    ...(loaded.cases.some((fixture) => fixture.currentReference)
      ? { currentReferenceMatches: 0 }
      : {}),
  };
  for (const fixture of loaded.cases) {
    const row = rows.get(fixture.id);
    assert.deepEqual(
      row.currentReference,
      fixture.currentReference,
      "current qualification changed",
    );
    assert.equal(row.native.state, "unsupported");
    assert.equal(row.native.reason, fixture.adapters.reasons.native);
    assert.equal(row.comparison.state, "not-compared");
    assert.equal(row.comparison.reason, "native formatter adapter unavailable");
    assert(["completed", "failed"].includes(row.legacy.state));
    if (fixture.config.length) {
      assert.deepEqual(row.legacy.argv, fixture.argv, "configured invocation is required");
      assert.deepEqual(
        row.legacy.config,
        fixture.config.map(({ path, sha256 }) => ({ path, sha256 })),
      );
    }
    assert(Array.isArray(row.legacy.passes) && row.legacy.passes.length <= 3);
    let previous = fixture.input;
    for (const [index, observation] of row.legacy.passes.entries()) {
      assert.equal(observation.pass, index + 1);
      const input = Buffer.from(observation.inputBase64, "base64");
      assert.equal(sha256(input), observation.inputSha256, "observed input digest mismatch");
      assert(input.equals(previous), "formatter pass input does not match the previous output");
      assert.equal(typeof observation.stdoutBase64, "string");
      assert.equal(typeof observation.stderrBase64, "string");
      if (observation.outputBase64 === null) {
        assert.equal(row.legacy.state, "failed");
        assert.equal(index, row.legacy.passes.length - 1);
        assert.equal(observation.outputSha256, null);
        assert.equal(observation.referenceComparison, null);
        assert.equal(
          observation.currentReferenceComparison,
          fixture.currentExpected ? null : undefined,
        );
      } else {
        const output = Buffer.from(observation.outputBase64, "base64");
        assert.equal(sha256(output), observation.outputSha256, "observed output digest mismatch");
        assert.deepEqual(
          observation.referenceComparison,
          compareBytes(fixture.expected, output),
          "forged reference comparison",
        );
        assert.deepEqual(
          observation.currentReferenceComparison,
          fixture.currentExpected ? compareBytes(fixture.currentExpected, output) : undefined,
          "forged current reference comparison",
        );
        previous = output;
      }
      if (row.legacy.state === "completed" || index < row.legacy.passes.length - 1) {
        assert.equal(observation.exitStatus, 0);
        assert.equal(observation.signal, null);
        assert.equal(observation.processError, null);
      }
    }
    if (row.legacy.state === "failed") {
      assert.equal(row.legacy.verdict, "failed");
      assert.equal(typeof row.legacy.error, "string");
      assert(row.legacy.error.length > 0);
      summary.legacyFailures += 1;
    } else {
      assert(report.binary, "completed formatter comparison requires observed executable identity");
      if (row.legacy.verdict === "matched-current-reference") {
        assert(fixture.currentReference, "unqualified current comparison");
        assert.equal(row.legacy.passes.length, 3, "all current passes are required");
        assert(row.legacy.passes.every((pass) => pass.referenceComparison.state === "different"));
        assert(
          row.legacy.passes.every((pass) => pass.currentReferenceComparison.state === "equal"),
        );
        summary.currentReferenceMatches += 1;
      } else if (row.legacy.verdict === "matched-reference") {
        assert(
          !fixture.currentReference,
          "qualified current behavior cannot match the historical drift",
        );
        assert.equal(
          row.legacy.passes.length,
          3,
          "output and both idempotence passes are required",
        );
        assert(row.legacy.passes.every((pass) => pass.referenceComparison.state === "equal"));
        summary.legacyMatches += 1;
      } else {
        assert.equal(row.legacy.verdict, "baseline-drift");
        assert(row.legacy.passes.some((pass) => pass.referenceComparison?.state === "different"));
        summary.baselineDrift += 1;
      }
    }
  }
  assert.deepEqual(report.summary, summary, "result accounting does not match every planned case");
  return summary;
}

function runFormatterCase(fixture, binaryPath, binaryFailure) {
  const row = {
    id: fixture.id,
    legacy: { state: "failed", verdict: "failed", passes: [] },
    native: { state: "unsupported", reason: fixture.adapters.reasons.native },
    comparison: { state: "not-compared", reason: "native formatter adapter unavailable" },
    ...(fixture.currentReference ? { currentReference: fixture.currentReference } : {}),
  };
  if (fixture.config.length) {
    row.legacy.argv = [...fixture.argv];
    row.legacy.config = fixture.config.map(({ path, sha256 }) => ({ path, sha256 }));
  }
  let workspace;
  try {
    if (binaryFailure) throw new Error(binaryFailure);
    workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-differential-formatter-"));
    const entry = path.join(workspace, "App.vue");
    fs.writeFileSync(entry, fixture.input);
    for (const file of fixture.config) {
      fs.writeFileSync(path.join(workspace, file.path), file.bytes);
    }
    for (let pass = 1; pass <= 3; pass += 1) {
      const input = fs.readFileSync(entry);
      const result = spawnSync(binaryPath, fixture.argv, {
        cwd: workspace,
        timeout: 30_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const output = fs.existsSync(entry) ? fs.readFileSync(entry) : null;
      const observation = formatterAttempt(
        pass,
        input,
        output,
        result,
        fixture.expected,
        fixture.currentExpected,
      );
      row.legacy.passes.push(observation);
      assertProcessSucceeded(result);
      assert(output, "formatter removed the entry file");
      row.legacy.state = "completed";
      row.legacy.verdict =
        observation.currentReferenceComparison?.state === "equal"
          ? "matched-current-reference"
          : observation.referenceComparison.state === "equal"
            ? "matched-reference"
            : "baseline-drift";
      if (row.legacy.verdict === "baseline-drift") break;
    }
  } catch (error) {
    row.legacy.state = "failed";
    row.legacy.verdict = "failed";
    row.legacy.error = error.message;
  } finally {
    if (workspace) fs.rmSync(workspace, { recursive: true, force: true });
  }
  return row;
}

export function runFormatterPack({ manifestPath, binaryPath, sourceRevision, repoRoot }) {
  const loaded = loadFormatterManifest(manifestPath);
  assert.match(sourceRevision, /^[a-f0-9]{40}$/, "actual source revision is required");
  const report = {
    schema: "vize.differential.result",
    version: 1,
    product: "formatter",
    manifestSha256: loaded.manifestSha256,
    sourceRevision,
    buildReceipt: null,
    binary: null,
    argv: FORMATTER_ARGV,
    rows: [],
  };
  let build = { sourceRevision };
  let binaryFailure;
  try {
    assert.equal(typeof binaryPath, "string", "explicit source-built executable is required");
    assert(
      path.isAbsolute(binaryPath),
      "executable must be an absolute path, without PATH fallback",
    );
    assert(fs.statSync(binaryPath).isFile(), "executable must be a file");
    fs.accessSync(binaryPath, fs.constants.X_OK);
    build = expectedBuildIdentity(repoRoot);
    assert.equal(
      build.sourceRevision,
      sourceRevision,
      "checkout revision changed before execution",
    );
    assert.equal(
      fs.realpathSync(binaryPath),
      fs.realpathSync(path.join(repoRoot, build.binaryPath)),
    );
    const receipt = JSON.parse(fs.readFileSync(`${binaryPath}.differential-build.json`, "utf8"));
    validateBuildReceipt(receipt, build);
    report.buildReceipt = receipt;
    const version = spawnSync(binaryPath, ["--version"], { timeout: 30_000 });
    report.binaryProbe = processObservation(version);
    assertProcessSucceeded(version);
    assert.equal(version.stdout.toString().trim(), build.cliVersion);
    loaded.cases = loaded.cases.map((fixture) => ({
      ...fixture,
      ...expressionWidthReference(repoRoot, fixture),
      ...soleChildWidthCliReference(repoRoot, fixture),
    }));
    report.binary = {
      path: build.binaryPath,
      sha256: build.binarySha256,
      version: build.cliVersion,
    };
  } catch (error) {
    binaryFailure = error.message;
  }
  report.rows = runPlannedCases(loaded, {
    runCase: (fixture) => runFormatterCase(fixture, binaryPath, binaryFailure),
  });
  report.summary = {
    plannedCases: report.rows.length,
    legacyMatches: report.rows.filter((row) => row.legacy.verdict === "matched-reference").length,
    legacyFailures: report.rows.filter((row) => row.legacy.verdict === "failed").length,
    baselineDrift: report.rows.filter((row) => row.legacy.verdict === "baseline-drift").length,
    nativeUnsupported: report.rows.length,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
    ...(loaded.cases.some((fixture) => fixture.currentReference)
      ? {
          currentReferenceMatches: report.rows.filter(
            (row) => row.legacy.verdict === "matched-current-reference",
          ).length,
        }
      : {}),
  };
  validateFormatterReport(loaded, report, build);
  return report;
}

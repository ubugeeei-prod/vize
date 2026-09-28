import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { expectedBuildIdentity, validateBuildReceipt } from "./build-receipt.mjs";
import { compareBytes } from "./compare.mjs";
import {
  loadProductManifest,
  readPinnedArtifact,
  runPlannedCases,
  sha256,
  validateResultEnvelope,
} from "./harness.mjs";

export const COMPILER_ARGV = [
  "build",
  "Layout.vue",
  "--format",
  "json",
  "--output",
  "out",
  "--ssr",
  "--no-config",
];
const OUTPUT_FILE = "out/Layout.json";
const NOT_COMPARED = "whole-product native compiler path unavailable";

export function loadCompilerManifest(manifestPath) {
  const loaded = loadProductManifest(manifestPath, "compiler");
  const root = path.dirname(manifestPath);
  return {
    ...loaded,
    cases: loaded.cases.map((fixture) => {
      assert.deepEqual(fixture.targets, ["ssr"]);
      assert.equal(fixture.state, "active");
      assert.equal(fixture.adapters.legacy, "compiler-cli-ssr-v1");
      assert.equal(fixture.adapters.native, null);
      assert.equal(typeof fixture.adapters.reasons.native, "string");
      assert.equal(fixture.inputs.root, "ssr-slot-scope");
      assert.equal(fixture.inputs.files.length, 1);
      const entry = fixture.inputs.files[0];
      assert.equal(entry.path, "Layout.vue");
      assert.equal(entry.role, "entry");
      assert.equal(fixture.comparison.contract, "compiler-ssr-module-bytes-v1");
      assert.deepEqual(fixture.comparison.requiredFacets, [
        "code",
        "css",
        "errors",
        "warnings",
        "macro_artifacts",
      ]);
      const input = readPinnedArtifact(path.join(root, fixture.inputs.root), entry);
      const expected = readPinnedArtifact(root, fixture.reference);
      return { ...fixture, input, expected };
    }),
  };
}

export function compilerAttempt(result, output, expected) {
  const observation = {
    exitStatus: result.status,
    signal: result.signal,
    stdoutBase64: result.stdout?.toString("base64") ?? "",
    stderrBase64: result.stderr?.toString("base64") ?? "",
    processError: result.error?.message ?? null,
    jsonBase64: output?.toString("base64") ?? null,
    jsonSha256: output ? sha256(output) : null,
    codeBase64: null,
    codeSha256: null,
    referenceComparison: null,
    facetError: null,
  };
  if (output) {
    try {
      const parsed = JSON.parse(output.toString("utf8"));
      assert.deepEqual(Object.keys(parsed), [
        "filename",
        "code",
        "css",
        "errors",
        "warnings",
        "script_lang",
        "macro_artifacts",
      ]);
      assert.equal(parsed.filename, "Layout.vue");
      assert.equal(parsed.script_lang, "js");
      assert.equal(parsed.css, null);
      assert.deepEqual(parsed.errors, []);
      assert.deepEqual(parsed.warnings, []);
      assert.deepEqual(parsed.macro_artifacts, []);
      const code = Buffer.from(parsed.code, "utf8");
      observation.codeBase64 = code.toString("base64");
      observation.codeSha256 = sha256(code);
      observation.referenceComparison = compareBytes(expected, code);
    } catch (error) {
      observation.facetError = error.message;
    }
  }
  return observation;
}

export function validateCompilerReport(loaded, report, expectedBuild) {
  validateResultEnvelope(loaded, report, expectedBuild.sourceRevision);
  assert.deepEqual(report.argv, COMPILER_ARGV);
  if (report.buildReceipt) validateBuildReceipt(report.buildReceipt, expectedBuild);
  if (report.binary) {
    assert(report.buildReceipt, "executable observation requires a build receipt");
    assert.equal(report.binary.path, expectedBuild.binaryPath);
    assert.equal(report.binary.sha256, expectedBuild.binarySha256);
    assert.equal(report.binary.version, expectedBuild.cliVersion);
  }
  const summary = {
    plannedCases: loaded.cases.length,
    legacyMatches: 0,
    legacyFailures: 0,
    baselineDrift: 0,
    nativeUnsupported: loaded.cases.length,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  };
  for (const [index, fixture] of loaded.cases.entries()) {
    const row = report.rows[index];
    assert.equal(row.id, fixture.id);
    assert.equal(row.target, "ssr");
    assert.deepEqual(row.native, { state: "unsupported", reason: fixture.adapters.reasons.native });
    assert.deepEqual(row.comparison, { state: "not-compared", reason: NOT_COMPARED });
    assert.deepEqual(row.legacy.argv, COMPILER_ARGV);
    assert.equal(row.legacy.inputSha256, sha256(fixture.input));
    assert(["completed", "failed"].includes(row.legacy.state));
    const attempt = row.legacy.observation;
    if (attempt) {
      assert.equal(typeof attempt.stdoutBase64, "string");
      assert.equal(typeof attempt.stderrBase64, "string");
      const json = attempt.jsonBase64 === null ? null : Buffer.from(attempt.jsonBase64, "base64");
      assert.equal(json ? sha256(json) : null, attempt.jsonSha256);
      const replay = compilerAttempt(
        {
          status: attempt.exitStatus,
          signal: attempt.signal,
          stdout: Buffer.from(attempt.stdoutBase64, "base64"),
          stderr: Buffer.from(attempt.stderrBase64, "base64"),
          error: attempt.processError ? new Error(attempt.processError) : null,
        },
        json,
        fixture.expected,
      );
      assert.deepEqual(attempt, replay, "compiler observation or byte comparison was forged");
    }
    if (row.legacy.state === "completed") {
      assert(report.binary, "completed observation requires executable identity");
      assert(attempt, "completed observation is required");
      assert.equal(attempt.exitStatus, 0);
      assert.equal(attempt.signal, null);
      assert.equal(attempt.processError, null);
      assert.equal(attempt.facetError, null, "invalid compiler facets cannot complete");
      assert.equal(
        row.legacy.verdict,
        attempt.referenceComparison.state === "equal" ? "matched-reference" : "baseline-drift",
      );
      summary[row.legacy.verdict === "matched-reference" ? "legacyMatches" : "baselineDrift"] += 1;
    } else {
      assert.equal(row.legacy.verdict, "failed");
      assert.equal(typeof row.legacy.error, "string");
      if (attempt?.facetError) assert.equal(row.legacy.error, attempt.facetError);
      summary.legacyFailures += 1;
    }
  }
  assert.deepEqual(report.summary, summary);
  return summary;
}

function runCompilerCase(fixture, binaryPath, binaryFailure) {
  const row = {
    id: fixture.id,
    target: "ssr",
    legacy: {
      state: "failed",
      verdict: "failed",
      argv: COMPILER_ARGV,
      inputSha256: sha256(fixture.input),
      observation: null,
    },
    native: { state: "unsupported", reason: fixture.adapters.reasons.native },
    comparison: { state: "not-compared", reason: NOT_COMPARED },
  };
  let workspace;
  try {
    if (binaryFailure) throw new Error(binaryFailure);
    workspace = fs.mkdtempSync(path.join(os.tmpdir(), "vize-differential-compiler-"));
    fs.writeFileSync(path.join(workspace, "Layout.vue"), fixture.input);
    const result = spawnSync(binaryPath, COMPILER_ARGV, {
      cwd: workspace,
      timeout: 30_000,
      maxBuffer: 4 * 1024 * 1024,
    });
    const output = fs.existsSync(path.join(workspace, OUTPUT_FILE))
      ? fs.readFileSync(path.join(workspace, OUTPUT_FILE))
      : null;
    row.legacy.observation = compilerAttempt(result, output, fixture.expected);
    if (row.legacy.observation.facetError) throw new Error(row.legacy.observation.facetError);
    if (result.error) throw result.error;
    assert.equal(result.status, 0, result.stderr?.toString());
    assert(output, "compiler JSON output is missing");
    row.legacy.state = "completed";
    row.legacy.verdict =
      row.legacy.observation.referenceComparison.state === "equal"
        ? "matched-reference"
        : "baseline-drift";
  } catch (error) {
    row.legacy.error = error.message;
  } finally {
    if (workspace) fs.rmSync(workspace, { recursive: true, force: true });
  }
  return row;
}

export function runCompilerPack({ manifestPath, binaryPath, sourceRevision, repoRoot }) {
  const loaded = loadCompilerManifest(manifestPath);
  assert.match(sourceRevision, /^[a-f0-9]{40}$/);
  const report = {
    schema: "vize.differential.result",
    version: 1,
    product: "compiler",
    manifestSha256: loaded.manifestSha256,
    sourceRevision,
    argv: COMPILER_ARGV,
    buildReceipt: null,
    binary: null,
    rows: [],
    summary: null,
  };
  let build = { sourceRevision };
  let binaryFailure;
  try {
    assert.equal(typeof binaryPath, "string", "explicit source-built executable is required");
    assert(path.isAbsolute(binaryPath), "executable path must be absolute");
    assert(fs.statSync(binaryPath).isFile());
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
    const version = spawnSync(binaryPath, ["--version"], { timeout: 30_000 });
    assert.equal(version.status, 0, version.stderr?.toString());
    assert.equal(version.stdout.toString().trim(), build.cliVersion);
    report.buildReceipt = receipt;
    report.binary = {
      path: build.binaryPath,
      sha256: build.binarySha256,
      version: build.cliVersion,
    };
  } catch (error) {
    binaryFailure = error.message;
  }
  report.rows = runPlannedCases(loaded, {
    runCase: (fixture) => [runCompilerCase(fixture, binaryPath, binaryFailure)],
  });
  report.summary = {
    plannedCases: loaded.cases.length,
    legacyMatches: report.rows.filter((row) => row.legacy.verdict === "matched-reference").length,
    legacyFailures: report.rows.filter((row) => row.legacy.verdict === "failed").length,
    baselineDrift: report.rows.filter((row) => row.legacy.verdict === "baseline-drift").length,
    nativeUnsupported: report.rows.length,
    pairedComparisons: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
  };
  validateCompilerReport(loaded, report, build);
  return report;
}

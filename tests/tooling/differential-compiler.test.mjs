import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { summarizeNativeAcceptance } from "../differential/acceptance-rates.mjs";
import { BUILD_RECIPE } from "../differential/build-receipt.mjs";
import {
  COMPILER_ARGV,
  compilerAttempt,
  loadCompilerManifest,
  runCompilerPack,
  validateCompilerReport,
} from "../differential/compiler.mjs";
import { sha256 } from "../differential/harness.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const pack = path.join(root, "tests/_fixtures/differential/compiler");
const manifestPath = path.join(pack, "manifest.json");
const loaded = loadCompilerManifest(manifestPath);
const fixture = loaded.cases[0];
const sourceRevision = loaded.manifest.baseRevision;
const build = {
  sourceRevision,
  binaryPath: "target/ci/vize",
  binarySha256: "0".repeat(64),
  cliVersion: "vize 0.429.0",
};

function syntheticReport(code = fixture.expected) {
  const output = Buffer.from(
    JSON.stringify({
      filename: "Layout.vue",
      code: code.toString(),
      css: null,
      errors: [],
      warnings: [],
      script_lang: "js",
      macro_artifacts: [],
    }),
  );
  const observation = compilerAttempt(
    { status: 0, signal: null, stdout: Buffer.alloc(0), stderr: Buffer.alloc(0) },
    output,
    fixture.expected,
  );
  const verdict =
    observation.referenceComparison.state === "equal" ? "matched-reference" : "baseline-drift";
  return {
    schema: "vize.differential.result",
    version: 1,
    product: "compiler",
    manifestSha256: loaded.manifestSha256,
    sourceRevision,
    argv: COMPILER_ARGV,
    buildReceipt: { schema: "vize.differential.build", version: 1, recipe: BUILD_RECIPE, ...build },
    binary: { path: build.binaryPath, sha256: build.binarySha256, version: build.cliVersion },
    rows: [
      {
        id: fixture.id,
        target: "ssr",
        legacy: {
          state: "completed",
          verdict,
          argv: COMPILER_ARGV,
          inputSha256: sha256(fixture.input),
          observation,
        },
        native: { state: "unsupported", reason: fixture.adapters.reasons.native },
        comparison: {
          state: "not-compared",
          reason: "whole-product native compiler path unavailable",
        },
      },
    ],
    summary: {
      plannedCases: 1,
      legacyMatches: verdict === "matched-reference" ? 1 : 0,
      legacyFailures: 0,
      baselineDrift: verdict === "baseline-drift" ? 1 : 0,
      nativeUnsupported: 1,
      pairedComparisons: 0,
      nativeHandled: 0,
      nativeEquivalent: 0,
    },
  };
}

void test("compiler fixture pins a real full-module reference without native credit", () => {
  assert.equal(loaded.cases.length, 1);
  assert.equal(fixture.id, "compiler/sfc/ssr-slot-scope");
  const report = syntheticReport();
  assert.deepEqual(validateCompilerReport(loaded, report, build), report.summary);
  assert.deepEqual(summarizeNativeAcceptance(loaded, report, { sourceRevision }).total, {
    planned: 1,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: 1,
    legacyBacked: 0,
    unverified: 0,
  });
});

void test("compiler comparison checks every code byte and rejects forged native completion", () => {
  const changed = syntheticReport(Buffer.concat([fixture.expected, Buffer.from("\n")]));
  assert.deepEqual(validateCompilerReport(loaded, changed, build), changed.summary);
  assert.equal(changed.rows[0].legacy.observation.referenceComparison.state, "different");
  const forged = syntheticReport();
  forged.rows[0].native = { state: "completed" };
  assert.throws(() => validateCompilerReport(loaded, forged, build));
  forged.rows[0].native = { state: "unsupported", reason: fixture.adapters.reasons.native };
  forged.rows[0].comparison.state = "equal";
  assert.throws(() => validateCompilerReport(loaded, forged, build));
  const tampered = syntheticReport();
  tampered.rows[0].legacy.observation.codeSha256 = "f".repeat(64);
  assert.throws(() => validateCompilerReport(loaded, tampered, build), /forged/);
});

void test("compiler input and reference digests fail closed", () => {
  for (const relative of ["ssr-slot-scope/Layout.vue", "ssr-slot-scope/module.expected.txt"]) {
    const copy = fs.mkdtempSync(path.join(os.tmpdir(), "vize-compiler-manifest-"));
    try {
      fs.cpSync(pack, copy, { recursive: true });
      fs.appendFileSync(path.join(copy, relative), "drift");
      assert.throws(
        () => loadCompilerManifest(path.join(copy, "manifest.json")),
        /SHA256 mismatch/,
      );
    } finally {
      fs.rmSync(copy, { recursive: true, force: true });
    }
  }
});

void test("malformed compiler output retains raw evidence and cannot complete", () => {
  const raw = Buffer.from("{broken JSON");
  const observation = compilerAttempt(
    { status: 0, signal: null, stdout: Buffer.from("log"), stderr: Buffer.from("warning") },
    raw,
    fixture.expected,
  );
  assert.equal(observation.jsonBase64, raw.toString("base64"));
  assert.equal(observation.jsonSha256, sha256(raw));
  assert.equal(observation.stdoutBase64, Buffer.from("log").toString("base64"));
  assert.equal(typeof observation.facetError, "string");
  const report = syntheticReport();
  report.rows[0].legacy = {
    ...report.rows[0].legacy,
    state: "failed",
    verdict: "failed",
    error: observation.facetError,
    observation,
  };
  report.summary = { ...report.summary, legacyMatches: 0, legacyFailures: 1 };
  assert.deepEqual(validateCompilerReport(loaded, report, build), report.summary);
  const forged = structuredClone(report);
  forged.rows[0].legacy.state = "completed";
  assert.throws(() => validateCompilerReport(loaded, forged, build));
});

void test("missing source-built CLI retains the planned failed row", () => {
  const report = runCompilerPack({
    manifestPath,
    binaryPath: path.join(root, "target/ci/missing-vize"),
    sourceRevision,
    repoRoot: root,
  });
  assert.equal(report.rows.length, 1);
  assert.equal(report.rows[0].legacy.state, "failed");
  assert.equal(report.rows[0].native.state, "unsupported");
  assert.equal(report.summary.nativeHandled, 0);
});

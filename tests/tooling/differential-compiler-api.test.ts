import assert from "node:assert/strict";
import { test } from "node:test";
import path from "node:path";
import fs from "node:fs";
import os from "node:os";
import { fileURLToPath } from "node:url";
import {
  compilerApiRows,
  loadCompilerApiManifest,
  runCompilerApiPack,
  validateCompilerApiReport,
} from "../differential/compiler-api.ts";
import { summarizeNativeAcceptance } from "../differential/acceptance-rates.mjs";
import { sha256 } from "../differential/harness.mjs";
import type { JsonObject, JsonValue } from "../differential/compiler-api-types.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath = path.join(
  root,
  "tests/_fixtures/differential/compiler/sfc-fix-history.manifest.json",
);
const loaded = loadCompilerApiManifest(manifestPath, root);

function object(value: JsonValue | undefined): JsonObject {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return value;
}

// Synthetic packets test the comparator only; the execution test builds and
// invokes the real product artifact before any completed report can validate.
function packet(cohort = loaded) {
  return {
    schema: cohort.contract.packetSchema,
    version: 1,
    cases: cohort.cases.map((fixture) => ({
      id: fixture.id,
      source: fixture.input.toString(),
      entrypoint: fixture.entrypoint,
      options: fixture.originalOptions,
      result: fixture.expected,
    })),
  };
}

function attempts(value = packet()) {
  const stdout = Buffer.from(JSON.stringify(value));
  const observation = {
    argv: ["--observe"],
    exitStatus: 0,
    signal: null,
    processError: null,
    stdoutBase64: stdout.toString("base64"),
    stderrBase64: "",
    stdoutSha256: sha256(stdout),
  };
  return [observation, structuredClone(observation)];
}

void test("complete SFC Result comparison preserves every CSS and module byte", () => {
  const original = compilerApiRows(loaded, attempts(), null);
  assert.equal(original.filter((row) => row.legacy.verdict === "matched-reference").length, 5);
  const changed = structuredClone(packet());
  const css = object(changed.cases[1].result.Ok);
  const code = object(changed.cases[3].result.Ok);
  assert(typeof css.css === "string");
  assert(typeof code.code === "string");
  css.css = `${css.css}\n`;
  code.code = `${code.code} `;
  const drift = compilerApiRows(loaded, attempts(changed), null);
  assert.equal(drift[1].legacy.verdict, "baseline-drift");
  assert.equal(drift[3].legacy.verdict, "baseline-drift");
  assert.equal(drift.filter((row) => row.legacy.verdict === "matched-reference").length, 3);
});

void test("missing, duplicate and wrong-option packets retain all five failed planned coordinates", () => {
  const missing = structuredClone(packet());
  missing.cases.pop();
  const duplicate = structuredClone(packet());
  duplicate.cases[1].id = duplicate.cases[0].id;
  const options = structuredClone(packet());
  object(object(options.cases[2].options).descriptorParseOptions).filename = "";
  const omitted = structuredClone(packet());
  delete object(omitted.cases[0].result.Ok).bindings;
  for (const invalid of [missing, duplicate, options, omitted]) {
    const rows = compilerApiRows(loaded, attempts(invalid), null);
    assert.equal(rows.length, 5);
    assert(rows.every((row) => row.legacy.state === "failed"));
    assert(rows.every((row) => row.native.state === "unsupported"));
  }
});

void test("a missing source-built observer keeps the complete denominator and no native credit", () => {
  const report = runCompilerApiPack({
    manifestPath,
    repoRoot: root,
    sourceRevision: loaded.manifest.baseRevision,
    binaryPath: null,
  });
  assert.equal(report.summary.plannedCases, 5);
  assert.equal(report.summary.legacyFailures, 5);
  const native = summarizeNativeAcceptance(loaded, report, {
    sourceRevision: report.sourceRevision,
  });
  assert.deepEqual(native.total, {
    planned: 5,
    nativeHandled: 0,
    nativeEquivalent: 0,
    unsupported: 5,
    legacyBacked: 0,
    unverified: 0,
  });
  const forged = structuredClone(report);
  Object.assign(forged.rows[0], { native: { state: "completed" } });
  assert.throws(
    () =>
      validateCompilerApiReport(loaded, forged, {
        repoRoot: root,
        binaryPath: null,
        sourceRevision: report.sourceRevision,
      }),
    /forged/,
  );
});

void test("non-repeating and failed processes cannot certify a compiler Result", () => {
  const changed = attempts();
  changed[1].stdoutBase64 = Buffer.from("{}").toString("base64");
  changed[1].stdoutSha256 = sha256(Buffer.from("{}"));
  const failed = attempts();
  failed[0].exitStatus = 1;
  failed[0].stderrBase64 = Buffer.from("compiler failed").toString("base64");
  for (const observations of [changed, failed]) {
    const rows = compilerApiRows(loaded, observations, null);
    assert.equal(rows.filter((row) => row.legacy.state === "failed").length, 5);
  }
});

void test("an empty build failure message retains every failed planned coordinate", () => {
  const rows = compilerApiRows(loaded, [], "");
  assert.equal(rows.length, 5);
  assert(rows.every((row) => row.legacy.state === "failed" && row.legacy.error === ""));
  const report = runCompilerApiPack({
    manifestPath,
    repoRoot: root,
    sourceRevision: loaded.manifest.baseRevision,
    binaryPath: null,
    buildFailure: "",
  });
  assert.equal(report.failure, "");
  assert.equal(report.buildReceipt, null);
  assert.equal(report.summary.legacyFailures, 5);
  assert.equal(report.summary.nativeHandled, 0);
  assert(report.rows.every((row) => row.legacy.result === null && row.legacy.error === ""));
});

for (const target of ["ssr", "vapor"]) {
  const targetManifest = path.join(
    root,
    `tests/_fixtures/differential/compiler/${target}-fix-history.manifest.json`,
  );
  const cohort = loadCompilerApiManifest(targetManifest, root);
  void test(`${target} comparison preserves all public output fields and target coordinates`, () => {
    const original = compilerApiRows(cohort, attempts(packet(cohort)), null);
    assert.equal(original.filter((row) => row.legacy.verdict === "matched-reference").length, 5);
    assert(original.every((row) => row.target === target && row.native.state === "unsupported"));
    for (const field of cohort.contract.fields) {
      const changed = structuredClone(packet(cohort));
      const result = changed.cases[0].result;
      const value = result[field];
      if (Array.isArray(value)) value.push("changed ordered value");
      else {
        assert(value === null || typeof value === "string");
        result[field] = `${value ?? ""} `;
      }
      const rows = compilerApiRows(cohort, attempts(changed), null);
      assert.equal(rows[0].legacy.verdict, "baseline-drift", field);
      assert.equal(rows.filter((row) => row.legacy.verdict === "matched-reference").length, 4);
    }
  });

  void test(`${target} rejects missing fields, foreign cases, source or original-option drift without dropping planned rows`, () => {
    const missing = structuredClone(packet(cohort));
    delete missing.cases[0].result.map;
    const foreign = structuredClone(packet(cohort));
    foreign.cases[0].id = "compiler/sfc/imported-component-before-prop";
    const source = structuredClone(packet(cohort));
    source.cases[0].source += "\n";
    const options = structuredClone(packet(cohort));
    object(object(options.cases[4].options).experimental).sourceMap = true;
    for (const changed of [missing, foreign, source, options]) {
      const rows = compilerApiRows(cohort, attempts(changed), null);
      assert.equal(rows.length, 5);
      assert(
        rows.every((row) => row.legacy.state === "failed" && row.native.state === "unsupported"),
      );
    }
    const report = runCompilerApiPack({
      manifestPath: targetManifest,
      repoRoot: root,
      sourceRevision: cohort.manifest.baseRevision,
      binaryPath: null,
    });
    assert.deepEqual(report.summary, {
      plannedCases: 5,
      legacyMatches: 0,
      baselineDrift: 0,
      legacyFailures: 5,
      nativeUnsupported: 5,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    });
    const evidenceDir = fs.mkdtempSync(path.join(os.tmpdir(), `compiler-${target}-build-failure-`));
    try {
      const buildFailed = runCompilerApiPack({
        manifestPath: targetManifest,
        repoRoot: root,
        sourceRevision: cohort.manifest.baseRevision,
        binaryPath: null,
        buildFailure: "Cargo failed before producing the selected observer artifact",
        evidenceDir,
      });
      assert.equal(
        buildFailed.failure,
        "Cargo failed before producing the selected observer artifact",
      );
      assert.deepEqual(buildFailed.summary, report.summary);
      assert(buildFailed.rows.every((row) => row.legacy.error === buildFailed.failure));
      assert.deepEqual(
        JSON.parse(fs.readFileSync(path.join(evidenceDir, "report.json"), "utf8")),
        buildFailed,
      );
      assert.equal(
        JSON.parse(fs.readFileSync(path.join(evidenceDir, "unverified-report.json"), "utf8"))
          .failure,
        buildFailed.failure,
      );
    } finally {
      fs.rmSync(evidenceDir, { recursive: true, force: true });
    }
    const forged = structuredClone(report);
    Object.assign(forged.rows[0], { native: { state: "completed" } });
    assert.throws(
      () =>
        validateCompilerApiReport(cohort, forged, {
          repoRoot: root,
          binaryPath: null,
          sourceRevision: report.sourceRevision,
        }),
      /forged/,
    );
  });
}

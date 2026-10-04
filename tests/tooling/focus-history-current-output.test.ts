import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import { focusSummary } from "../differential/focus-history-report.ts";
import {
  compareFocusCurrentOutputs,
  FOCUS_CURRENT_INDEX,
  loadFocusCurrentBaseline,
  validateFocusCurrentReport,
} from "../differential/focus-history-current-output.ts";
import { FOCUS_CASES_SHA256 } from "../differential/focus-history.ts";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const baseline = loadFocusCurrentBaseline(root);

function attempt(bytes: Buffer) {
  return {
    stdoutBase64: bytes.toString("base64"),
    stdoutSha256: sha256(bytes),
    stderrBase64: "",
    stderrSha256: sha256(Buffer.alloc(0)),
    exitStatus: 0,
    signal: null,
    processError: null,
  };
}

// Pure comparison-law inputs copy frozen reviewed bytes. They are not fresh
// source-built process observations and confer no runtime/native credit.
function comparisonInput() {
  const rows = baseline.rows.map((row) => ({
    id: row.id,
    inputBase64: row.fixture.input.toString("base64"),
    inputSha256: row.inputSha256,
    sourceSha256: row.sourceSha256,
    legacy: {
      state: "captured",
      argv: ["--legacy"],
      attempts: [attempt(row.legacy), attempt(row.legacy)],
    },
    native: {
      state: "refused",
      argv: ["--native"],
      attempts: [attempt(row.native), attempt(row.native)],
    },
    comparison: { state: "not-compared", reason: "no-reviewed-complete-oracle" },
  }));
  return {
    schema: "vize.focus-history.capture",
    version: 1,
    acceptance: "unreviewed",
    sourceRevision: baseline.observedReceipt.source.sourceRevision,
    fixtureSha256: FOCUS_CASES_SHA256,
    buildReceipt: baseline.observedReceipt,
    rows,
    summary: focusSummary(rows),
  };
}

void test("reviewed current baseline retains exact originals, complete outputs and observed source/build bindings", () => {
  assert.equal(
    baseline.observedReceipt.source.sourceRevision,
    "7f7b63122456fd066c86bcab7c281c9c6c9d389a",
  );
  assert.equal(
    baseline.observedReceipt.source.sourceTree,
    "44eeeedb7ed5198f04392f19b84d0ded92185cc1",
  );
  assert.equal(
    baseline.observedReceipt.source.productSourceTree,
    "aff9bdeb6983bebafa9f88298e29bd1af9c0e692",
  );
  assert.equal(
    baseline.observedReceipt.artifact.sha256,
    "ed20f95725dc9c807f781cb19e8307fa3e9b791c0159b292456bd7f9d937e2f0",
  );
  assert.equal(baseline.index.observed.checkRun, 37170632227);
  assert.equal(baseline.index.observed.artifact.id, 11291905348);
  assert.equal(baseline.index.review.authoredOptions, "None");
  assert.deepEqual(baseline.index.review.observedDefaults, { locale: "En", helpLevel: "Full" });
  const capture = comparisonInput();
  const report = compareFocusCurrentOutputs(root, capture, baseline.observedReceipt);
  validateFocusCurrentReport(root, report, baseline.observedReceipt);
  assert.equal(report.summary.legacyCurrentMatches, 8);
  assert.equal(report.summary.nativeRefusalMatches, 8);
  assert.equal(report.summary.wholeCurrentComparisons, 32);
  assert.equal(report.summary.nativeEquivalent, 0);
  assert.equal(report.summary.historicalCompleteOutputAuthorities, 0);
  assert.equal(report.capture.acceptance, "unreviewed");
});

void test("baseline source/options/authority and full expected bytes cannot be silently recaptured", () => {
  const original = JSON.parse(fs.readFileSync(path.join(root, FOCUS_CURRENT_INDEX), "utf8"));
  for (const mutate of [
    (index: any) => index.rows.pop(),
    (index: any) => {
      index.rows[1] = index.rows[0];
    },
    (index: any) => {
      index.observed.source.sourceRevision = "0".repeat(40);
    },
    (index: any) => {
      index.observed.captureSha256 = "0".repeat(64);
    },
    (index: any) => {
      index.rows[0].legacy.sha256 = "0".repeat(64);
    },
    (index: any) => {
      index.rows[0].native.path = index.rows[0].legacy.path;
    },
    (index: any) => {
      index.review.authoredOptions = "En/Full";
    },
    (index: any) => {
      index.review.historicalFullOutputAuthority = true;
    },
    (index: any) => {
      index.review.nativeEquivalenceAuthority = true;
    },
    (index: any) => {
      index.review.exercisedFixAuthority = true;
    },
  ]) {
    const mutated = structuredClone(original);
    mutate(mutated);
    assert.throws(() =>
      loadFocusCurrentBaseline(root, Buffer.from(`${JSON.stringify(mutated, null, 2)}\n`)),
    );
  }
});

void test("whole comparisons detect metadata, messages, geometry, help, fixes and unchanged-requery drift", () => {
  for (const [from, to] of [
    ['filename: "test.vue"', 'filename: "changed.vue"'],
    ["category: Accessibility", "category: Essential"],
    ["constructor_default_help: Full", "constructor_default_help: Short"],
    ["severity: Warning", "severity: Error"],
    ["It can disrupt navigation", "It cannot disrupt navigation"],
    ["start: 19", "start: 20"],
    ["end: 36", "end: 35"],
    ["Manage focus programmatically", "Manage focus automatically"],
    ["labels: []", 'labels: ["invented"]'],
    ["fix: None", 'fix: Some("invented")'],
    ["unchanged_requery: Some", "unchanged_requery: None"],
  ]) {
    const capture = comparisonInput();
    const changed = Buffer.from(baseline.rows[0].legacy.toString().replace(from, to));
    assert(!changed.equals(baseline.rows[0].legacy));
    capture.rows[0].legacy.attempts = [attempt(changed), attempt(changed)];
    const report = compareFocusCurrentOutputs(root, capture, baseline.observedReceipt);
    assert.equal(report.rows[0].legacy.state, "current-output-drift", from);
    assert.equal(report.summary.legacyCurrentMatches, 7);
    assert.equal(report.summary.currentOutputDrift, 1);
    validateFocusCurrentReport(root, report, baseline.observedReceipt);
  }
});

void test("negative complete outputs and native refusal contexts are compared in full", () => {
  const capture = comparisonInput();
  const empty = Buffer.from(
    baseline.rows[2].legacy
      .toString()
      .replace("diagnostics: []", 'diagnostics: ["invented warning"]'),
  );
  capture.rows[2].legacy.attempts = [attempt(empty), attempt(empty)];
  const refused = JSON.parse(baseline.rows[3].native.toString());
  refused.context = refused.context.replace('filename: "test.vue"', 'filename: "changed.vue"');
  const changed = Buffer.from(`${JSON.stringify(refused)}\n`);
  capture.rows[3].native.attempts = [attempt(changed), attempt(changed)];
  const report = compareFocusCurrentOutputs(root, capture, baseline.observedReceipt);
  assert.equal(report.rows[2].legacy.state, "current-output-drift");
  assert.equal(report.rows[3].native.state, "current-output-drift");
  assert.equal(report.summary.currentOutputDrift, 2);
  assert.equal(report.summary.nativeEquivalent, 0);
});

void test("failed lanes retain both raw attempts and differences before aggregate rejection", () => {
  const capture = comparisonInput();
  capture.rows[0].legacy.state = "failed";
  capture.rows[0].legacy.error = "first process failed";
  capture.rows[0].legacy.attempts[0] = { ...attempt(Buffer.from("partial\n")), exitStatus: 1 };
  capture.summary = focusSummary(capture.rows);
  const report = compareFocusCurrentOutputs(root, capture, baseline.observedReceipt);
  assert.equal(report.rows[0].legacy.state, "failed");
  assert.equal(report.rows[0].legacy.wholeComparisons[0].state, "different");
  assert.equal(report.rows[0].legacy.wholeComparisons[1].state, "equal");
  assert.deepEqual(report.capture, capture);
  assert.equal(report.summary.captureFailures, 1);
  assert.equal(report.summary.wholeCurrentComparisons, 32);
  validateFocusCurrentReport(root, report, baseline.observedReceipt);
});

void test("comparison reports reject false clean/acceptance/provenance and missing whole observations", () => {
  const report = compareFocusCurrentOutputs(root, comparisonInput(), baseline.observedReceipt);
  for (const mutate of [
    (r: any) => {
      r.authority = "historical-full-output";
    },
    (r: any) => {
      r.summary.nativeHandled = 8;
    },
    (r: any) => {
      r.summary.nativeEquivalent = 8;
    },
    (r: any) => {
      r.summary.pairedComparisons = 8;
    },
    (r: any) => {
      r.summary.historicalCompleteOutputAuthorities = 8;
    },
    (r: any) => {
      r.capture.acceptance = "accepted";
    },
    (r: any) => {
      r.rows[0].comparison = { state: "equal" };
    },
    (r: any) => {
      r.rows[0].native.provenance = { scope: "whole-product" };
    },
    (r: any) => {
      r.rows[0].legacy.wholeComparisons.pop();
    },
    (r: any) => {
      r.capture.rows[0].legacy.attempts.pop();
    },
    (r: any) => {
      r.capture.buildReceipt.source.sourceRevision = "0".repeat(40);
    },
  ]) {
    const mutated = structuredClone(report);
    mutate(mutated);
    assert.throws(() => validateFocusCurrentReport(root, mutated, baseline.observedReceipt));
  }
});

void test("reviewed-current fixtures/report do not admit product execution to T0 or remove it from T1", () => {
  const execution = "tests/tooling/focus-history-capture-execution.test.ts";
  const pure = "tests/tooling/focus-history-current-output.test.ts";
  const files = [
    FOCUS_CURRENT_INDEX,
    "tests/differential/focus-history-current-output.ts",
    "docs/davinci/decisions/2026-10-04-linter-focus-current-output.md",
  ];
  for (const row of baseline.index.rows) files.push(row.legacy.path, row.native.path);
  files.push(baseline.index.observed.buildReceipt.path);
  for (const input of files) {
    const pr = planToolingTests([input], { cwd: root }).tests;
    const full = planToolingTests([input], { tier: "merge", cwd: root }).tests;
    assert(!pr.includes(execution));
    assert(pr.includes(pure));
    assert(full.includes(execution) && full.includes(pure));
  }
  const source = fs.readFileSync(path.join(root, execution), "utf8");
  const write = source.indexOf('"current-output-report.json"');
  const aggregate = source.indexOf("current.summary");
  assert(write >= 0 && aggregate > write);
  assert.equal(source.split("buildProductObserver({").length - 1, 1);
  assert.equal(source.split("runFocusCapture({").length - 1, 1);
});

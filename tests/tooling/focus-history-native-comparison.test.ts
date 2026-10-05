import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  compareFocusNativeOutputs,
  validateFocusNativeComparison,
} from "../differential/focus-history-native-comparison.ts";
import {
  compareFocusCurrentOutputs,
  FOCUS_CURRENT_CAPTURE_PATH,
  FOCUS_CURRENT_CAPTURE_SHA256,
  loadFocusCurrentOracle,
} from "../differential/focus-history-current-oracle.ts";
import { currentFocusSummary } from "../differential/focus-history-current-report.ts";
import { sha256 } from "../differential/harness.mjs";
import {
  attempt,
  replaceWhole,
  root,
  syntheticCurrentCapture,
} from "./support/focus-history-current.ts";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

void test("synthetic v3 validator fixture joins full bytes without rewriting historical v1/v2 authority", () => {
  const originalBytes = fs.readFileSync(path.join(root, FOCUS_CURRENT_CAPTURE_PATH));
  const capture = syntheticCurrentCapture();
  const comparison = compareFocusNativeOutputs(root, capture, capture.buildReceipt);
  assert.deepEqual(comparison.summary, {
    reviewedCurrentOutputOracles: 8,
    completeLegacyMatches: 8,
    nativeHandled: 8,
    nativeEquivalent: 8,
    pairedComparisons: 8,
    wholeCurrentComparisons: 32,
    wholePairedComparisons: 32,
    currentOutputDrift: 0,
    captureFailures: 0,
    historicalCompleteOutputAuthorities: 0,
  });
  validateFocusNativeComparison(root, comparison, capture.buildReceipt);
  const historical = loadFocusCurrentOracle(root);
  assert.deepEqual(comparison.historicalCapture, historical);
  assert.equal(
    compareFocusCurrentOutputs(root, historical, historical.buildReceipt).summary.nativeEquivalent,
    0,
  );
  assert.equal(sha256(originalBytes), FOCUS_CURRENT_CAPTURE_SHA256);
  assert(fs.readFileSync(path.join(root, FOCUS_CURRENT_CAPTURE_PATH)).equals(originalBytes));
  for (const row of comparison.rows)
    assert.deepEqual(
      row.pairs.map((pair: any) => [pair.legacyAttempt, pair.nativeAttempt]),
      [
        [0, 0],
        [0, 1],
        [1, 0],
        [1, 1],
      ],
    );
});

void test("whole native output detects every diagnostic, identity, option and query metadata change", () => {
  for (const [before, after] of [
    ['filename: "test.vue"', 'filename: "changed.vue"'],
    ["locale: None", "locale: Some(En)"],
    ["vue_version: None", "vue_version: Some(V3)"],
    ["vapor: None", "vapor: Some(false)"],
    ["help_level: None", "help_level: Some(Full)"],
    ["severity: None", "severity: Some(Warning)"],
    ["NoAutofocus", "Namesake"],
    ['description: "Disallow the use of the autofocus attribute"', 'description: "changed"'],
    ["category: Accessibility", "category: Essential"],
    ["fixable: false", "fixable: true"],
    ["default_severity: Warning", "default_severity: Error"],
    ["locale: En", "locale: Ja"],
    ["constructor_default_help: Full", "constructor_default_help: None"],
    ["original_warning_count: 1", "original_warning_count: 0"],
    ['rule_name: "a11y/no-autofocus"', 'rule_name: "a11y/no-access-key"'],
    ["severity: Warning", "severity: Error"],
    ["The autofocus attribute should not be used", "Wrong warning"],
    ["Manage focus programmatically", "Change focus programmatically"],
    ["start: 19,", "start: 20,"],
    ["end: 36,", "end: 35,"],
    ["labels: [],", 'labels: ["invented"],'],
    ["fix: None", 'fix: Some("invented")'],
    ["error_count: 0", "error_count: 1"],
    ["warning_count: 1", "warning_count: 0"],
    ["applications: []", 'applications: ["invented edit"]'],
    ["unchanged_requery: Some", "unchanged_requery: None"],
  ]) {
    const capture = syntheticCurrentCapture();
    replaceWhole(capture.rows[0].native, before, after);
    const comparison = compareFocusNativeOutputs(root, capture, capture.buildReceipt);
    assert.equal(comparison.summary.completeLegacyMatches, 8);
    assert.equal(comparison.summary.nativeEquivalent, 7);
    assert.equal(comparison.summary.currentOutputDrift, 1);
    assert(
      comparison.rows[0].native.wholeComparisons.every((whole: any) => whole.state === "different"),
    );
    validateFocusNativeComparison(root, comparison, capture.buildReceipt);
  }
});

void test("whole comparison preserves original field ordering even when every value and repeat agrees", () => {
  const capture = syntheticCurrentCapture();
  replaceWhole(
    capture.rows[0].native,
    "    locale: En,\n    constructor_default_help: Full,",
    "    constructor_default_help: Full,\n    locale: En,",
  );
  const comparison = compareFocusNativeOutputs(root, capture, capture.buildReceipt);
  assert.equal(comparison.summary.nativeEquivalent, 7);
  assert.equal(comparison.summary.currentOutputDrift, 1);
});

void test("pairwise equal changed outputs and empty or counts-only bodies cannot evade the immutable authority", () => {
  const capture = syntheticCurrentCapture();
  for (const lane of [capture.rows[0].legacy, capture.rows[0].native])
    replaceWhole(lane, "warning_count: 1", "warning_count: 0");
  const comparison = compareFocusNativeOutputs(root, capture, capture.buildReceipt);
  assert.equal(comparison.rows[0].comparison.state, "equal");
  assert.equal(comparison.summary.nativeEquivalent, 7);
  assert.equal(comparison.summary.currentOutputDrift, 2);
  const empty = syntheticCurrentCapture();
  replaceWhole(empty.rows[2].native, "diagnostics: []", 'diagnostics: ["invented warning"]');
  assert.equal(
    compareFocusNativeOutputs(root, empty, empty.buildReceipt).summary.nativeEquivalent,
    7,
  );
  const structural = syntheticCurrentCapture();
  for (const row of structural.rows)
    row.native.attempts = [0, 1].map(() =>
      attempt(
        Buffer.from(
          `${JSON.stringify({
            state: "handled",
            observation: "Case {\n}\nRuleIdentity {\n}\nObservation {\n    warning_count: 1\n}\n",
          })}\n`,
        ),
      ),
    );
  assert.equal(
    compareFocusNativeOutputs(root, structural, structural.buildReceipt).summary.nativeEquivalent,
    0,
  );
});

void test("failed native and legacy attempts retain all raw observations and four cross-pairs without credit", () => {
  for (const name of ["legacy", "native"]) {
    const capture = syntheticCurrentCapture();
    const lane = capture.rows[0][name];
    lane.state = "failed";
    lane.error = "first actual process failed";
    lane.attempts[0] = { ...attempt(Buffer.from("partial\n")), exitStatus: 1 };
    capture.summary = currentFocusSummary(capture.rows);
    const comparison = compareFocusNativeOutputs(root, capture, capture.buildReceipt);
    assert.deepEqual(comparison.capture, capture);
    assert.equal(comparison.rows[0][name].state, "failed");
    assert.equal(comparison.rows[0].pairs.length, 4);
    assert.equal(comparison.summary.captureFailures, 1);
    assert.equal(comparison.summary.nativeEquivalent, 7);
    assert.equal(comparison.summary.pairedComparisons, 7);
    assert.equal(comparison.summary.wholeCurrentComparisons, 32);
    assert.equal(comparison.summary.wholePairedComparisons, 32);
    assert.deepEqual(comparison.rows[0].comparison, {
      state: "not-compared",
      reason: "capture-failed",
    });
    assert.equal(comparison.rows[0].native.wholeComparisons[1].state, "equal");
    validateFocusNativeComparison(root, comparison, capture.buildReceipt);
  }
});

void test("serialized v3 cannot invent source, comparison, provenance or historical authority", () => {
  const capture = syntheticCurrentCapture();
  const valid = compareFocusNativeOutputs(root, capture, capture.buildReceipt);
  for (const mutate of [
    (r: any) => {
      r.version = 2;
    },
    (r: any) => {
      r.summary.nativeEquivalent = 0;
    },
    (r: any) => {
      r.summary.historicalCompleteOutputAuthorities = 8;
    },
    (r: any) => {
      r.rows[0].native.provenance = { scope: "whole-product" };
    },
    (r: any) => r.rows[0].pairs.pop(),
    (r: any) => {
      r.rows[0].pairs[0].nativeAttempt = 1;
    },
    (r: any) => {
      r.rows[0].pairs[0].comparison = { state: "different" };
    },
    (r: any) => r.rows[0].native.wholeComparisons.pop(),
    (r: any) => {
      r.historicalCapture.rows[0].native.state = "handled";
    },
    (r: any) => {
      r.historicalCapture.acceptance = "accepted";
    },
    (r: any) => {
      r.qualification.authority = "historical-full-output";
    },
    (r: any) => {
      r.oraclePacketSha256 = "0".repeat(64);
    },
    (r: any) => {
      r.sourceRevision = "b".repeat(40);
    },
    (r: any) => r.capture.rows[0].native.attempts.pop(),
    (r: any) => {
      r.capture.acceptance = "accepted";
    },
    (r: any) => {
      r.capture.buildReceipt.source.sourceRevision = "b".repeat(40);
    },
  ]) {
    const changed = structuredClone(valid);
    mutate(changed);
    assert.throws(() => validateFocusNativeComparison(root, changed, capture.buildReceipt));
  }
});

void test("new current protocol controls stay pure T0 and source-built execution stays T1", () => {
  const execution = "tests/tooling/focus-history-capture-execution.test.ts";
  const pure = "tests/tooling/focus-history-native-comparison.test.ts";
  for (const input of [
    pure,
    "tests/differential/focus-history-native-comparison.ts",
    "tests/differential/focus-history-current-report.ts",
  ]) {
    const pr = planToolingTests([input], { cwd: root }).tests;
    const full = planToolingTests([input], { tier: "merge", cwd: root }).tests;
    assert(pr.includes(pure));
    assert(!pr.includes(execution));
    assert(full.includes(execution) && full.includes(pure));
  }
});

import assert from "node:assert/strict";
import { test } from "node:test";

import { assertCompatNonVacuity } from "../_helpers/compat-nonvacuity.ts";
import type { CompatSummary } from "../_helpers/compat-ratchet.ts";

const exactParity: CompatSummary = {
  vizeDiagnosticCount: 0,
  baselineDiagnosticCount: 0,
  sharedCount: 0,
  messageMismatchCount: 0,
  documentedDifferenceCount: 0,
  falsePositiveCount: 0,
  falseNegativeCount: 0,
  falsePositiveRatio: 0,
  falseNegativeRatio: 0,
};

test("compat non-vacuity requires seeded proof for diagnostic-free summaries", () => {
  assertCompatNonVacuity("fixture", exactParity, {
    file: "src/App.vue",
    passed: true,
    reason: null,
    summary: { ...exactParity, vizeDiagnosticCount: 1, baselineDiagnosticCount: 1, sharedCount: 1 },
  });
  assert.throws(
    () => assertCompatNonVacuity("fixture", exactParity, null),
    /diagnostic-free on both tools/,
  );
});

test("compat non-vacuity allows mutation-induced false positives after shared proof", () => {
  assertCompatNonVacuity("fixture", exactParity, {
    file: "src/App.vue",
    passed: true,
    reason: null,
    summary: {
      ...exactParity,
      vizeDiagnosticCount: 3,
      baselineDiagnosticCount: 1,
      sharedCount: 1,
      falsePositiveCount: 2,
      falsePositiveRatio: 2,
    },
  });
});

test("compat non-vacuity allows diagnostic-bearing summaries to use the normal ratchet", () => {
  assert.doesNotThrow(() =>
    assertCompatNonVacuity(
      "fixture",
      { ...exactParity, vizeDiagnosticCount: 1, baselineDiagnosticCount: 1, sharedCount: 1 },
      null,
    ),
  );
});

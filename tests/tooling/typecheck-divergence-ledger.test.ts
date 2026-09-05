import assert from "node:assert/strict";
import { test } from "node:test";

import {
  compare,
  suggestionBaseline,
  suggestionDifference,
  suggestionFiles,
} from "./_helpers/typecheck-divergence-fixture.ts";

/**
 * A message-only divergence is reviewable exactly like a code divergence: one
 * ledger entry at the same span, carrying both texts and a written reason,
 * moves it out of the bucket and into `documentedDifferences`.
 */
test("typecheck divergence cancels a documented message-only difference", () => {
  const files = [
    {
      file: "src/App.vue",
      diagnostics: ["error:3:7 [TS2322] Type 'A | B' is not assignable to type 'C'."],
    },
  ];
  const baseline = "src/App.vue(3,7): error TS2322: Type 'B | A' is not assignable to type 'C'.\n";
  const difference = {
    project: "fixture",
    file: "src/App.vue",
    severity: "error",
    line: 3,
    column: 7,
    vize: { code: 2322, message: "Type 'A | B' is not assignable to type 'C'." },
    baseline: { code: 2322, message: "Type 'B | A' is not assignable to type 'C'." },
    issue: 3447,
    reason: "tsgo and tsc order the members of this union independently when printing it.",
  };

  const result = compare(files, baseline, [difference]);
  assert.equal(result.summary.messageMismatchCount, 0);
  assert.equal(result.summary.documentedDifferenceCount, 1);
  assert.equal(result.summary.sharedCount, 0);
  assert.equal(result.summary.falsePositiveCount, 0);
  assert.equal(result.summary.falseNegativeCount, 0);

  // The entry only cancels the exact pair it describes; a reworded side leaves
  // the divergence in the bucket.
  const reworded = compare(files, baseline, [
    {
      ...difference,
      baseline: { code: 2322, message: "Type 'B | D' is not assignable to type 'C'." },
    },
  ]);
  assert.equal(reworded.summary.messageMismatchCount, 1);
  assert.equal(reworded.summary.documentedDifferenceCount, 0);
});

test("typecheck divergence cancels a documented difference against both buckets", () => {
  const result = compare(suggestionFiles, suggestionBaseline, [suggestionDifference]);

  assert.equal(result.summary.documentedDifferenceCount, 1);
  assert.equal(result.summary.falsePositiveCount, 0);
  assert.equal(result.summary.falseNegativeCount, 0);
  assert.equal(result.summary.falsePositiveRatio, 0);
  assert.equal(result.summary.falseNegativeRatio, 0);
  assert.equal(result.summary.vizeDiagnosticCount, 1);
  assert.equal(result.summary.baselineDiagnosticCount, 1);
  assert.deepEqual(result.documentedDifferences, [
    {
      file: "src/App.vue",
      severity: "error",
      line: 5,
      column: 16,
      vize: suggestionDifference.vize,
      baseline: suggestionDifference.baseline,
      issue: 3358,
      reason: suggestionDifference.reason,
    },
  ]);

  const withoutLedger = compare(suggestionFiles, suggestionBaseline);
  assert.equal(withoutLedger.summary.documentedDifferenceCount, 0);
  assert.equal(withoutLedger.summary.falsePositiveCount, 1);
  assert.equal(withoutLedger.summary.falseNegativeCount, 1);
});

test("typecheck divergence cancels reviewed one-sided differences", () => {
  const vizeOnly = {
    project: "fixture",
    file: "src/App.vue",
    severity: "error",
    line: 3,
    column: 7,
    vize: { code: 2339, message: "Property 'missing' does not exist on type '{}'." },
    baseline: null,
    issue: 5722,
    reason: "Vize reports this stricter diagnostic while vue-tsc accepts the same source.",
  };
  const baselineOnly = {
    project: "fixture",
    file: "src/App.vue",
    severity: "error",
    line: 4,
    column: 9,
    vize: null,
    baseline: {
      code: 1117,
      message: "An object literal cannot have multiple properties with the same name.",
    },
    issue: 5722,
    reason: "vue-tsc reports a generated duplicate listener key that Vize intentionally omits.",
  };

  const result = compare(
    [
      {
        file: "src/App.vue",
        diagnostics: ["error:3:7 [TS2339] Property 'missing' does not exist on type '{}'."],
      },
    ],
    "src/App.vue(4,9): error TS1117: An object literal cannot have multiple properties with the same name.\n",
    [vizeOnly, baselineOnly],
  );

  assert.equal(result.summary.documentedDifferenceCount, 2);
  assert.equal(result.summary.falsePositiveCount, 0);
  assert.equal(result.summary.falseNegativeCount, 0);
  assert.deepEqual(result.documentedDifferences, [
    {
      file: "src/App.vue",
      severity: "error",
      line: 3,
      column: 7,
      vize: vizeOnly.vize,
      baseline: null,
      issue: 5722,
      reason: vizeOnly.reason,
    },
    {
      file: "src/App.vue",
      severity: "error",
      line: 4,
      column: 9,
      vize: null,
      baseline: baselineOnly.baseline,
      issue: 5722,
      reason: baselineOnly.reason,
    },
  ]);
});

test("typecheck divergence keeps a documented difference that no longer reproduces", () => {
  const reworded = { code: 2552, message: "Cannot find name 'useRouter'." };
  for (const [difference, label] of [
    [{ ...suggestionDifference, project: "other" }, "another project"],
    [{ ...suggestionDifference, column: 15 }, "a shifted column"],
    [{ ...suggestionDifference, vize: reworded }, "a reworded vize message"],
    [{ ...suggestionDifference, baseline: { code: 2551, message: "x y" } }, "a new vue-tsc code"],
  ] as const) {
    const result = compare(suggestionFiles, suggestionBaseline, [difference]);
    assert.equal(result.summary.documentedDifferenceCount, 0, label);
    assert.equal(result.summary.falsePositiveCount, 1, label);
    assert.equal(result.summary.falseNegativeCount, 1, label);
  }
  // Only vize reports at 5:16, so there is nothing to cancel the false positive
  // against and the ledger entry must not hide it.
  const oneSided = compare(suggestionFiles, "", [suggestionDifference]);
  assert.equal(oneSided.summary.documentedDifferenceCount, 0);
  assert.equal(oneSided.summary.falsePositiveCount, 1);
});

test("typecheck divergence rejects an unreviewable documented difference", () => {
  for (const [difference, message] of [
    [{ ...suggestionDifference, reason: "cosmetic" }, /reason must explain/],
    [{ ...suggestionDifference, issue: 0 }, /issue must be the tracking issue/],
    [{ ...suggestionDifference, project: "" }, /must name a project/],
    [{ ...suggestionDifference, severity: "info" }, /severity must be error or warning/],
    [{ ...suggestionDifference, line: 0 }, /line must be a positive safe integer/],
    [{ ...suggestionDifference, file: "src/App.ts" }, /must reference a \.vue file/],
    [{ ...suggestionDifference, file: "../App.vue" }, /stay inside/],
    [{ ...suggestionDifference, baseline: suggestionDifference.vize }, /must record a difference/],
    [{ ...suggestionDifference, vize: null, baseline: null }, /must record at least one tool side/],
    [{ ...suggestionDifference, vize: { code: 2552 } }, /message must be a string/],
  ] as const) {
    assert.throws(() => compare(suggestionFiles, suggestionBaseline, [difference]), message);
  }
  assert.throws(
    () =>
      compare(suggestionFiles, suggestionBaseline, [suggestionDifference, suggestionDifference]),
    /duplicates an earlier documented difference/,
  );
  assert.throws(
    () => compare(suggestionFiles, suggestionBaseline, "no" as never),
    /must be an array/,
  );
});

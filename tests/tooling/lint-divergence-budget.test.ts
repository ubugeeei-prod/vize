import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertBudgetsPassed,
  attachBudget,
  parseBudgetMode,
  summarizeBudgets,
} from "../../legacy-tools/fixtures/lint-divergence-budget.mjs";

test("lint divergence passes only with readable input and zero unexplained drift", () => {
  const artifact = attachBudget(baseArtifact());

  assert.deepEqual(artifact.budget, {
    maxFalsePositiveCount: 0,
    maxFalseNegativeCount: 0,
    falsePositivePassed: true,
    falseNegativePassed: true,
    unusableReason: null,
    verdict: "passed",
    passed: true,
  });
  assert.doesNotThrow(() => assertBudgetsPassed([artifact]));
  assert.deepEqual(summarizeBudgets([artifact]), {
    status: "success",
    passed: true,
    projectCount: 1,
    passedCount: 1,
    failedCount: 0,
    unusableCount: 0,
    breachedCount: 0,
    failedProjects: [],
  });
});

test("lint divergence fails on any remaining false positive or false negative", () => {
  const artifact = attachBudget(
    baseArtifact({
      falsePositiveCount: 1,
      falseNegativeCount: 2,
    }),
  );

  assert.equal(artifact.budget.verdict, "breached");
  assert.equal(artifact.budget.passed, false);
  assert.throws(
    () => assertBudgetsPassed([artifact]),
    /Lint divergence budget breached for fixture: 1 false positives exceed maxFalsePositiveCount 0; 2 false negatives exceed maxFalseNegativeCount 0/u,
  );
});

test("lint divergence refuses empty shard evidence", () => {
  assert.throws(() => assertBudgetsPassed([]), /Lint divergence budget has no measured projects/u);
  assert.deepEqual(summarizeBudgets([]), {
    status: "failure",
    passed: false,
    projectCount: 0,
    passedCount: 0,
    failedCount: 0,
    unusableCount: 0,
    breachedCount: 0,
    failedProjects: [],
  });
});

test("lint divergence treats parse errors and empty measurements as unusable evidence", () => {
  assert.equal(
    attachBudget(baseArtifact({ baselineParseErrorCount: 1 })).budget.unusableReason,
    "eslint-plugin-vue could not parse 1 compared file(s)",
  );
  assert.equal(
    attachBudget(baseArtifact({ baselineInvalidRangeCount: 1 })).budget.unusableReason,
    "eslint-plugin-vue reported 1 finding(s) with invalid source ranges",
  );
  assert.equal(
    attachBudget({ ...baseArtifact(), files: { comparedCount: 0 } }).budget.unusableReason,
    "the project selected no Vue files",
  );
  assert.equal(
    attachBudget({ ...baseArtifact(), baseline: { comparedRuleCount: 0 } }).budget.unusableReason,
    "no mapped eslint-plugin-vue rule was comparable under the selected preset",
  );
});

test("record-only mode writes warnings without disarming validation", () => {
  const artifact = attachBudget(baseArtifact({ falseNegativeCount: 1 }));
  const writes: string[] = [];
  const originalWrite = process.stdout.write.bind(process.stdout);
  try {
    process.stdout.write = ((chunk: string | Uint8Array) => {
      writes.push(String(chunk));
      return true;
    }) as typeof process.stdout.write;
    assert.doesNotThrow(() => assertBudgetsPassed([artifact], "record-only"));
  } finally {
    process.stdout.write = originalWrite as typeof process.stdout.write;
  }

  assert.deepEqual(writes, [
    "::warning title=Lint divergence budget not enforced::Lint divergence budget breached for fixture: 1 false negatives exceed maxFalseNegativeCount 0\n",
  ]);
});

test("lint divergence budget mode rejects typos instead of falling back", () => {
  assert.equal(parseBudgetMode("enforce"), "enforce");
  assert.equal(parseBudgetMode("record-only"), "record-only");
  assert.throws(
    () => parseBudgetMode("off"),
    /--budget-mode must be one of: enforce, record-only/u,
  );
});

function baseArtifact(summary = {}) {
  return {
    project: "fixture",
    files: { comparedCount: 1 },
    baseline: { comparedRuleCount: 2 },
    divergence: {
      summary: {
        falsePositiveCount: 0,
        falseNegativeCount: 0,
        baselineParseErrorCount: 0,
        baselineInvalidRangeCount: 0,
        ...summary,
      },
    },
  };
}

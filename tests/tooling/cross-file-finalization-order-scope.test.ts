import assert from "node:assert/strict";
import { test } from "node:test";
import {
  HISTORICAL_FINALIZATION_PATHS,
  planFinalizationComparison,
} from "../../.github/scripts/cross-file-finalization-scope.ts";
import {
  HISTORICAL_ORDERING_PATHS,
  planFinalizationOrdering,
} from "../../.github/scripts/cross-file-finalization-order-scope.ts";

void test("the original three-path planner keeps its exact default result", () => {
  const expected = [...HISTORICAL_FINALIZATION_PATHS].sort();
  assert.deepEqual(planFinalizationComparison(HISTORICAL_FINALIZATION_PATHS, "pull_request"), {
    scopeMatches: true,
    runComparison: true,
    manualRequest: false,
    expectedPaths: expected,
    actualPaths: expected,
    reason: "historical changed-source scope matches",
    metricsProduced: false,
    measurementQualification:
      "not evaluated by the planner; the original comparator remains strict",
  });
  assert.equal(
    planFinalizationComparison([], "pull_request").reason,
    "outside the historical three-file changed-source certificate",
  );
});

void test("only the exact reviewed ordering scope selects automatic historical measurements", () => {
  assert.equal(
    planFinalizationOrdering(HISTORICAL_ORDERING_PATHS, "pull_request").runComparison,
    true,
  );
  assert.equal(
    planFinalizationOrdering([...HISTORICAL_ORDERING_PATHS].reverse(), "pull_request").scopeMatches,
    true,
  );
  for (const paths of [
    [HISTORICAL_ORDERING_PATHS[0]],
    [],
    [...HISTORICAL_ORDERING_PATHS, "crates/vize_croquis_cf/src/rules/element_id.rs"],
    [...HISTORICAL_ORDERING_PATHS, HISTORICAL_ORDERING_PATHS[0]],
  ]) {
    const plan = planFinalizationOrdering(paths, "pull_request");
    assert.equal(plan.runComparison, false);
    assert.equal(plan.scopeMatches, false);
    assert.equal(plan.metricsProduced, false);
  }
});

void test("manual requests retain strict prerequisites and malformed scope evidence fails", () => {
  const plan = planFinalizationOrdering([HISTORICAL_ORDERING_PATHS[0]], "workflow_dispatch");
  assert.equal(plan.runComparison, true);
  assert.equal(plan.manualRequest, true);
  assert.equal(plan.metricsProduced, false);
  assert.throws(() => planFinalizationOrdering([], "push"));
  assert.throws(() => planFinalizationOrdering(["crates/a\0b.rs"], "pull_request"));
  for (const paths of [
    ["../a.rs"],
    ["/crates/a.rs"],
    ["crates/a.txt"],
    ["crates/../a.rs"],
    ["crates/a.rs", "crates/a.rs"],
    [],
  ])
    assert.throws(() => planFinalizationComparison([], "pull_request", paths));
});

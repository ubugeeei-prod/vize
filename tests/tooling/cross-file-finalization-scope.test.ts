import assert from "node:assert/strict";
import { test } from "node:test";
import {
  HISTORICAL_FINALIZATION_PATHS,
  planFinalizationComparison,
} from "../../.github/scripts/cross-file-finalization-scope.ts";

void test("only the complete historical changed-source scope selects the automatic experiment", () => {
  const original = [...HISTORICAL_FINALIZATION_PATHS];
  assert.equal(planFinalizationComparison(original, "pull_request").scopeMatches, true);
  assert.equal(
    planFinalizationComparison([...original].reverse(), "pull_request").runComparison,
    true,
  );
  for (const changed of [
    original.slice(0, 2),
    [...original, "crates/vize/src/commands/lint/cross_file/tests.rs"],
    [original[0], "crates/vize/src/commands/lint/cross_file/tests.rs"],
    [...original, original[0]],
    [],
  ]) {
    const plan = planFinalizationComparison(changed, "pull_request");
    assert.equal(plan.scopeMatches, false);
    assert.equal(plan.runComparison, false);
    assert.equal(plan.metricsProduced, false);
  }
});

void test("an explicit manual request preserves the unchanged comparator prerequisites", () => {
  const plan = planFinalizationComparison(
    [
      "crates/vize/src/commands/lint/cross_file.rs",
      "crates/vize/src/commands/lint/cross_file/tests.rs",
    ],
    "workflow_dispatch",
  );
  assert.equal(plan.scopeMatches, false);
  assert.equal(plan.runComparison, true);
  assert.equal(plan.manualRequest, true);
  assert.equal(plan.metricsProduced, false);
  assert.match(plan.measurementQualification, /original comparator remains strict/u);
});

void test("unknown triggers and malformed Git path evidence fail before a scope decision", () => {
  assert.throws(
    () => planFinalizationComparison(HISTORICAL_FINALIZATION_PATHS, "push"),
    /unknown/u,
  );
  assert.throws(() => planFinalizationComparison(["crates/x\0y"], "pull_request"));
});

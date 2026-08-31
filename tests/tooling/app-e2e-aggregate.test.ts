import assert from "node:assert/strict";
import { test } from "node:test";

import { assertAppE2eAggregate } from "../../legacy-tools/github/app-e2e-aggregate.mjs";

test("stable aggregators fail closed for every incomplete producer state", () => {
  assert.deepEqual(
    assertAppE2eAggregate({
      profile: "readiness",
      suite: "all",
      runRequired: false,
      planResult: "success",
      producerResult: "skipped",
      plannedCount: 0,
    }),
    { expectedCount: 0, outcome: "success" },
  );
  assert.equal(
    assertAppE2eAggregate({
      profile: "full",
      suite: "all",
      runRequired: true,
      planResult: "success",
      producerResult: "success",
      plannedCount: 17,
    }).expectedCount,
    17,
  );
  assert.equal(
    assertAppE2eAggregate({
      profile: "readiness",
      suite: "all",
      runRequired: true,
      planResult: "success",
      producerResult: "success",
      plannedCount: 6,
    }).expectedCount,
    6,
  );
  for (const producerResult of ["failure", "cancelled", "skipped", "neutral"]) {
    assert.throws(
      () =>
        assertAppE2eAggregate({
          profile: "readiness",
          suite: "all",
          runRequired: true,
          planResult: "success",
          producerResult,
          plannedCount: 6,
        }),
      new RegExp(producerResult),
    );
  }
  assert.throws(
    () =>
      assertAppE2eAggregate({
        profile: "readiness",
        suite: "all",
        runRequired: true,
        planResult: "success",
        producerResult: "success",
        plannedCount: 5,
      }),
    /expected 6/,
  );
  assert.throws(
    () =>
      assertAppE2eAggregate({
        profile: "full",
        suite: "all",
        runRequired: true,
        planResult: "success",
        producerResult: "success",
        plannedCount: 16,
      }),
    /expected 17/,
  );
  assert.throws(
    () =>
      assertAppE2eAggregate({
        profile: "full",
        suite: "all",
        runRequired: true,
        planResult: "failure",
        producerResult: "skipped",
        plannedCount: 0,
      }),
    /planner is failure/,
  );
});

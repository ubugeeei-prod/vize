import assert from "node:assert/strict";
import { test } from "node:test";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
} from "../../legacy-tools/github/release-preflight-bootstrap.mjs";
import { requiredReleaseWorkflows } from "../../legacy-tools/github/release-preflight-evidence.mjs";
import { releaseSha } from "./support/release-preflight.ts";

test("release gate wait budget is bounded by the build matrix, not by a shard fallback", async () => {
  let elapsed = 0;
  await assert.rejects(
    bootstrapRequiredWorkflowRuns({
      sha: releaseSha,
      dispatchPlans: [],
      listRuns: async () => [],
      dispatchWorkflow: async () => {},
      sleep: async (milliseconds) => {
        elapsed += milliseconds;
      },
      now: () => elapsed,
      pollIntervalMs: 60 * 60 * 1000,
    }),
    /Timed out after 5400000ms waiting for release gates/,
  );
  assert.equal(elapsed, 90 * 60 * 1000);
});

/**
 * The gate set is the release's critical path, so it is asserted whole: a gate
 * added back without measuring it is exactly how this grew to hours. See
 * `requiredReleaseWorkflows` for what each removal costs and #4461 to restore.
 */
test("only gates that finish inside the build matrix block a release", () => {
  assert.deepEqual(requiredReleaseWorkflows, ["Check", "Benchmark", "Fuzz", "Miri", "Docs build"]);
  for (const removed of ["Real Project Matrix", "Native Smoke", "App E2E"]) {
    assert.equal(
      requiredReleaseWorkflows.includes(removed),
      false,
      `${removed} must not be back without a measurement`,
    );
  }
});

test("the release dispatches only the two gates that are not already green on main", () => {
  const plans = createReleaseGateDispatchPlans({
    ref: "v1.2.3",
    headSha: releaseSha,
    baseSha: "b".repeat(40),
  });

  assert.deepEqual(
    plans.map((plan) => plan.workflowName),
    ["Benchmark", "Fuzz"],
  );
  // Check, Miri and Docs build are push-triggered, so a release confirms them
  // rather than waiting on them.
  for (const pushTriggered of ["Check", "Miri", "Docs build"]) {
    assert.equal(
      plans.some((plan) => plan.workflowName === pushTriggered),
      false,
      `${pushTriggered} is already green on the release commit`,
    );
  }
});

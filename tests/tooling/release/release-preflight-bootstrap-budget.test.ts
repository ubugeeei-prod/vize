import assert from "node:assert/strict";
import { test } from "node:test";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
} from "../../../tools/support/compat/github/release-preflight-bootstrap.mjs";
import { requiredReleaseWorkflows } from "../../../tools/support/compat/github/release-preflight-evidence.mjs";
import { releaseSha } from "../support/release-preflight.ts";

test("release publication does not wait on an external workflow gate", async () => {
  let elapsed = 0;
  const selected = await bootstrapRequiredWorkflowRuns({
    sha: releaseSha,
    dispatchPlans: [],
    listRuns: async () => [],
    dispatchWorkflow: async () => {
      throw new Error("release publication must not dispatch a gate");
    },
    sleep: async (milliseconds) => {
      elapsed += milliseconds;
    },
    now: () => elapsed,
    pollIntervalMs: 60 * 60 * 1000,
  });
  assert.equal(selected.size, 0);
  assert.equal(elapsed, 0);
});

/**
 * The gate set is the release's critical path, so it is asserted whole. Real
 * Project Matrix stays in the release path for real-project smoke evidence,
 * while typecheck parity remains a separately enforced ratchet.
 */
test("publication does not wait for a CI workflow", () => {
  assert.deepEqual(requiredReleaseWorkflows, []);
  for (const removed of [
    "Check",
    "Benchmark",
    "Native Smoke",
    "App E2E",
    "Fuzz",
    "Miri",
    "Real Project Matrix",
    "Docs build",
  ]) {
    assert.equal(
      requiredReleaseWorkflows.includes(removed),
      false,
      `${removed} must not gate a release`,
    );
  }
});

test("the release dispatches only gates that need tag-bound evidence", () => {
  const plans = createReleaseGateDispatchPlans({
    ref: "v1.2.3",
    headSha: releaseSha,
    baseSha: "b".repeat(40),
  });

  assert.deepEqual(
    plans.map((plan) => plan.workflowName),
    [],
  );
});

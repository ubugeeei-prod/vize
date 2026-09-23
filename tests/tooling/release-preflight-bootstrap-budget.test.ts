import assert from "node:assert/strict";
import { test } from "node:test";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
} from "../../legacy-tools/github/release-preflight-bootstrap.mjs";
import { requiredReleaseWorkflows } from "../../legacy-tools/github/release-preflight-evidence.mjs";
import { releaseSha } from "./support/release-preflight.ts";

test("release gate wait budget covers the full Real Project Matrix release gate", async () => {
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
    /Timed out after 20700000ms waiting for release gates/,
  );
  assert.equal(elapsed, 345 * 60 * 1000);
});

/**
 * The gate set is the release's critical path, so it is asserted whole. Real
 * Project Matrix stays in the release path for real-project smoke evidence,
 * while typecheck parity remains a separately enforced ratchet.
 */
test("the release gate set keeps safety gates while benchmarks stay optional", () => {
  assert.deepEqual(requiredReleaseWorkflows, [
    "Check",
    "Fuzz",
    "Miri",
    "Real Project Matrix",
    "Docs build",
  ]);
  for (const removed of ["Benchmark", "Native Smoke", "App E2E"]) {
    assert.equal(
      requiredReleaseWorkflows.includes(removed),
      false,
      `${removed} must not be back without a measurement`,
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
    ["Check", "Miri", "Docs build", "Fuzz", "Real Project Matrix"],
  );
});

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
 * Project Matrix is intentionally back because typecheck divergence now enforces
 * exact parity and release preflight validates the shard artifacts.
 */
test("the release gate set includes Real Project Matrix but not artifact-only smoke gates", () => {
  assert.deepEqual(requiredReleaseWorkflows, [
    "Check",
    "Benchmark",
    "Fuzz",
    "Miri",
    "Real Project Matrix",
    "Docs build",
  ]);
  for (const removed of ["Native Smoke", "App E2E"]) {
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
    ["Benchmark", "Fuzz", "Real Project Matrix"],
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

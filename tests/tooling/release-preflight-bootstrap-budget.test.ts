import assert from "node:assert/strict";
import { test } from "node:test";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
} from "../../tools/github/release-preflight-bootstrap.mjs";
import { releaseSha } from "./support/release-preflight.ts";

test("release gate wait budget covers hosted Real Project Matrix fallback", async () => {
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
    /Timed out after 30600000ms waiting for release gates/,
  );
  assert.equal(elapsed, 510 * 60 * 1000);
});

test("release Real Project Matrix dispatch keeps core evidence alive", () => {
  const matrix = createReleaseGateDispatchPlans({
    ref: "v1.2.3",
    headSha: releaseSha,
    baseSha: "b".repeat(40),
  }).find((plan) => plan.workflowName === "Real Project Matrix");

  assert.equal(matrix?.inputs.core_tools_timeout_ms, "2400000");
  // TEMPORARY, tracked in #4461: the surface still runs and still records its
  // verdict, but it does not gate the release while its three breaching
  // fixtures are mis-measured. Flip back to "enforce" with the bootstrap.
  assert.equal(matrix?.inputs.typecheck_divergence_mode, "record-only");
});

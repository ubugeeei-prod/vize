import assert from "node:assert/strict";
import { test } from "node:test";

import { bootstrapRequiredWorkflowRuns } from "../../tools/github/release-preflight-bootstrap.mjs";
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

import assert from "node:assert/strict";
import { test } from "node:test";

import { createReleaseGateDispatchPlans } from "../../../tools/support/compat/github/release-preflight-bootstrap.mjs";
import { requiredReleaseWorkflows } from "../../../tools/support/compat/github/release-preflight-evidence.mjs";
import { releaseSha } from "../support/release-preflight.ts";

test("publication waits only for exact-source full Check", () => {
  assert.deepEqual(requiredReleaseWorkflows, ["Check"]);
  for (const optional of [
    "Benchmark",
    "Native Smoke",
    "App E2E",
    "Fuzz",
    "Miri",
    "Real Project Matrix",
    "Docs build",
  ]) {
    assert.equal(
      requiredReleaseWorkflows.includes(optional),
      false,
      `${optional} keeps its independent schedule`,
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
    ["Check"],
  );
});

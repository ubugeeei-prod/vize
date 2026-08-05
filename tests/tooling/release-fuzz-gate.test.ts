import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { createReleaseGateDispatchPlans } from "../../tools/github/release-preflight-bootstrap.mjs";
import { readRepoFile } from "./support/github-workflows.ts";
import { releaseSha } from "./support/release-preflight.ts";

function fuzzPlan() {
  const plans = createReleaseGateDispatchPlans({
    ref: "v1.2.3",
    headSha: releaseSha,
    baseSha: "b".repeat(40),
  });
  const plan = plans.find((candidate) => candidate.workflowName === "Fuzz");
  assert.ok(plan);
  return plan;
}

function fuzzWorkflow() {
  return parse(readRepoFile(".github", "workflows", "fuzz.yml")) as {
    on?: { workflow_dispatch?: { inputs?: Record<string, Record<string, unknown>> } };
    jobs?: { fuzz?: { steps?: Array<{ id?: string; run?: string }> } };
  };
}

test("release evidence replays the corpus instead of searching for new inputs", () => {
  // A fresh campaign is a randomized search, so gating a tag on one lets an
  // input discovered minutes earlier block a release it has nothing to do with.
  // Four of six tags around v0.325.0–v0.330.0 failed that way.
  const plan = fuzzPlan();

  assert.deepEqual(plan.inputs, { mode: "replay" });
  assert.equal(plan.expectedRunName, `Fuzz replay @ ${releaseSha}`);
});

test("the release gate cannot fall back to a timed campaign", () => {
  // Passing a budget again would restore the randomized search on the release
  // path, which is the whole failure mode this replaced.
  assert.doesNotMatch(JSON.stringify(fuzzPlan().inputs), /max-total-time/);
});

test("replay mode is an explicit choice the workflow understands", () => {
  const mode = fuzzWorkflow().on?.workflow_dispatch?.inputs?.mode;

  // A typed choice makes a typo fail at dispatch rather than silently running a
  // campaign as the release gate.
  assert.equal(mode?.type, "choice");
  assert.deepEqual(mode?.options, ["campaign", "replay"]);
  // Discovery stays the default, so the nightly schedule keeps searching.
  assert.equal(mode?.default, "campaign");
});

test("replay runs every corpus input exactly once", () => {
  const run = fuzzWorkflow().jobs?.fuzz?.steps?.find((step) => step.id === "fuzz")?.run ?? "";

  assert.match(run, /-runs=0/, "replay must execute the corpus and exit");
  assert.match(run, /-max_total_time=\$FUZZ_MAX_TOTAL_TIME/, "campaigns keep their budget");
});

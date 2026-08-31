import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  createReleaseGateDispatchPlans,
  releaseGateRunQualifiers,
} from "../../legacy-tools/github/release-preflight-bootstrap.mjs";
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

// Run the fuzz step's script with a `cargo` stub on PATH so the assertion sees
// the arguments the workflow actually assembles for a mode, not just tokens
// present somewhere in the file.
function fuzzCommand(mode: string): string {
  const script = fuzzWorkflow().jobs?.fuzz?.steps?.find((step) => step.id === "fuzz")?.run;
  assert.ok(script, "fuzz workflow must have a step with id: fuzz");
  const stubDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fuzz-command-"));
  fs.writeFileSync(path.join(stubDirectory, "cargo"), '#!/usr/bin/env bash\necho "$@"\n', {
    mode: 0o755,
  });

  const result = spawnSync("bash", ["-c", script], {
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${stubDirectory}${path.delimiter}${process.env.PATH ?? ""}`,
      FUZZ_MAX_TOTAL_TIME: "120",
      FUZZ_MODE: mode,
      FUZZ_TARGET: "sfc_parse",
    },
  });

  assert.equal(result.status, 0, `${result.stderr}${result.stdout}`);
  return result.stdout;
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
  const replay = fuzzCommand("replay");
  assert.match(replay, /-runs=0/, "replay must execute the corpus and exit");
  assert.doesNotMatch(replay, /-max_total_time/, "replay must not run a timed search");

  const campaign = fuzzCommand("campaign");
  assert.match(campaign, /-max_total_time=120/, "campaigns keep their budget");
  assert.doesNotMatch(campaign, /-runs=0/, "campaigns must keep searching for new inputs");
});

test("the release gate rejects the nightly campaign as replay evidence", () => {
  const plan = fuzzPlan();
  const qualify = releaseGateRunQualifiers([plan]).get("Fuzz");
  assert.ok(qualify);

  // The nightly campaign runs on main, so a tag can share its head SHA. Reusing
  // it as evidence would put the randomized search back on the release path.
  assert.equal(qualify({ event: "schedule", display_title: "Fuzz schedule" }), false);
  assert.equal(qualify({ event: "workflow_dispatch", display_title: plan.expectedRunName }), true);
});

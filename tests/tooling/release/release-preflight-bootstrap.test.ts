import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
  releaseGateRunQualifiers,
} from "../../../tools/support/compat/github/release-preflight-bootstrap.mjs";
import {
  requiredReleaseWorkflowEvidence,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "../../../tools/support/compat/github/release-preflight-evidence.mjs";
import { readRepoFile } from "../support/github-workflows.ts";
import { releaseSha, successfulReleaseRun } from "../support/release-preflight.ts";

function releasePlans() {
  return createReleaseGateDispatchPlans({
    ref: "v1.2.3",
    headSha: releaseSha,
    baseSha: "b".repeat(40),
  });
}

function readWorkflow(file: string) {
  return parse(readRepoFile(".github", "workflows", file)) as {
    name?: string;
    on?: { workflow_dispatch?: { inputs?: Record<string, Record<string, unknown>> } };
    "run-name"?: string;
  };
}

function findEvidenceRun<T extends { path: string }>(runs: T[], workflowName: string): T {
  const evidence = requiredReleaseWorkflowEvidence.get(workflowName);
  const run = runs.find((candidate) => candidate.path === evidence?.path);
  assert.ok(run);
  return run;
}

function findReleasePlan(workflowName: string) {
  const plans = releasePlans();
  const plan = plans.find((candidate) => candidate.workflowName === workflowName);
  assert.ok(plan);
  return plan;
}

test("release gate plans bind exact SHAs to expected evidence titles", () => {
  assert.deepEqual(
    releasePlans().map(({ workflowName, workflowId, ref, inputs, expectedRunName }) => ({
      workflowName,
      workflowId,
      ref,
      inputs,
      expectedRunName,
    })),
    [
      {
        workflowName: "Check",
        workflowId: "check.yml",
        ref: "v1.2.3",
        inputs: { "release-qualification": true },
        expectedRunName: `Check full @ ${releaseSha}`,
      },
    ],
  );
});

test("Check dispatch uses the current full workflow without obsolete inputs", () => {
  assert.equal(
    releasePlans().some((plan) => plan.workflowName === "Check"),
    true,
  );
  const workflow = readWorkflow("check.yml");
  const dispatch = workflow.on?.workflow_dispatch;
  assert.ok(dispatch);
  const inputs = dispatch.inputs;
  assert.ok(inputs);
  assert.deepEqual(Object.keys(inputs), ["release-qualification", "compare-rust-test-profile"]);
  assert.equal(inputs["compare-rust-test-profile"].required, false);
  assert.deepEqual(inputs["release-qualification"], {
    description: "Qualify an immutable release source without repeating daily source coverage",
    type: "boolean",
    required: false,
    default: false,
  });
  const runName = workflow["run-name"];
  assert.ok(runName);
  assert.match(runName, /Check full @/);
  assert.match(runName, /github\.sha/);
});

test("release gate plans reject ambiguous SHAs and missing refs", () => {
  assert.throws(
    () =>
      createReleaseGateDispatchPlans({ ref: "v1.2.3", headSha: releaseSha, baseSha: releaseSha }),
    /base SHA must differ/,
  );
  assert.throws(
    () =>
      createReleaseGateDispatchPlans({
        ref: "v1.2.3",
        headSha: "A".repeat(40),
        baseSha: "b".repeat(40),
      }),
    /Release head SHA must be a full lowercase/,
  );
  assert.throws(
    () =>
      createReleaseGateDispatchPlans({
        ref: "v1.2.3",
        headSha: releaseSha,
        baseSha: "short",
      }),
    /Release base SHA must be a full lowercase/,
  );
  for (const ref of [undefined, null, ""]) {
    assert.throws(
      () =>
        Reflect.apply(createReleaseGateDispatchPlans, undefined, [
          { ref, headSha: releaseSha, baseSha: "b".repeat(40) },
        ]),
      /Release dispatch ref is required/,
    );
  }
});

test("Benchmark keeps an on-demand exact range while release bootstrap leaves it optional", () => {
  assert.equal(
    releasePlans().some((plan) => plan.workflowName === "Benchmark"),
    false,
  );
  const benchmark = readWorkflow("benchmark.yml");
  const benchmarkInputs = benchmark.on?.workflow_dispatch?.inputs ?? {};
  assert.deepEqual(Object.keys(benchmarkInputs).sort(), ["base_sha", "head_sha"]);
  assert.equal(benchmarkInputs.base_sha?.required, true);
  assert.equal(benchmarkInputs.head_sha?.required, true);
  assert.match(benchmark["run-name"] ?? "", /^Benchmark /);
  assert.match(benchmark["run-name"] ?? "", /inputs\.base_sha/);
  assert.match(benchmark["run-name"] ?? "", /\.\.\./);
  assert.match(benchmark["run-name"] ?? "", /inputs\.head_sha/);
});

test("Fuzz dispatch identifies its mode and target", () => {
  const fuzz = readWorkflow("fuzz.yml");
  assert.equal(fuzz.on?.workflow_dispatch?.inputs?.["max-total-time"]?.default, "120");
  assert.match(fuzz["run-name"] ?? "", /^Fuzz /);
  assert.match(fuzz["run-name"] ?? "", /inputs\.max-total-time/);
  assert.match(fuzz["run-name"] ?? "", /inputs\.mode/);
  assert.match(fuzz["run-name"] ?? "", /github\.sha/);
});

test("release bootstrap authenticates exact Check source, workflow, event, title and ref", () => {
  const qualifiers = releaseGateRunQualifiers(releasePlans());
  const run = successfulReleaseRun("Check", 1);
  assert.equal(selectRequiredWorkflowRuns([run], releaseSha, undefined, qualifiers).size, 1);
  for (const patch of [
    { head_sha: "b".repeat(40) },
    { path: ".github/workflows/foreign.yml" },
    { event: "schedule" },
    { event: "pull_request" },
    { display_title: "Check fast" },
    { head_branch: "main" },
    { head_branch: "release/v1.2.3" },
  ]) {
    assert.throws(
      () => selectRequiredWorkflowRuns([{ ...run, ...patch }], releaseSha, undefined, qualifiers),
      /Check: missing/,
    );
  }
});

test("release gate bootstrap reuses evidence that already exists at the SHA", async () => {
  const plans = releasePlans();
  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  for (const workflowName of requiredReleaseWorkflows) {
    const run = findEvidenceRun(runs, workflowName);
    run.display_title = findReleasePlan(workflowName).expectedRunName;
    run.event = "workflow_dispatch";
  }
  const dispatched: string[] = [];

  const selected = await bootstrapRequiredWorkflowRuns({
    sha: releaseSha,
    dispatchPlans: plans,
    listRuns: async () => runs,
    dispatchWorkflow: async (plan) => dispatched.push(plan.workflowName),
  });

  assert.deepEqual(dispatched, []);
  assert.deepEqual([...selected.keys()], requiredReleaseWorkflows);
  for (const workflowName of requiredReleaseWorkflows) {
    assert.equal(findEvidenceRun(runs, workflowName).event, "workflow_dispatch");
  }
});

test("release gate bootstrap dispatches only missing gates and waits for exact evidence", async () => {
  const plans = releasePlans();
  const dispatched: string[] = [];
  const alwaysPresent = requiredReleaseWorkflows
    .filter((name) => !plans.some((plan) => plan.workflowName === name))
    .map((name, index) => successfulReleaseRun(name, index + 1));
  const dispatchedRuns = plans.map((plan, index) => ({
    ...successfulReleaseRun(plan.workflowName, index + 10),
    display_title: plan.expectedRunName,
    event: "workflow_dispatch",
  }));
  let listCount = 0;
  let clock = 0;

  const selected = await bootstrapRequiredWorkflowRuns({
    sha: releaseSha,
    dispatchPlans: plans,
    listRuns: async () => {
      listCount += 1;
      if (listCount === 1) return alwaysPresent;
      if (listCount === 2) {
        return dispatchedRuns.map((run) => ({ ...run, status: "in_progress", conclusion: null }));
      }
      return [...alwaysPresent, ...dispatchedRuns];
    },
    dispatchWorkflow: async (plan) => dispatched.push(plan.workflowName),
    sleep: async (milliseconds) => {
      clock += milliseconds;
    },
    now: () => clock,
    timeoutMs: 10_000,
    pollIntervalMs: 1_000,
  });

  assert.deepEqual(
    dispatched,
    plans.map((plan) => plan.workflowName),
  );
  assert.deepEqual([...selected.keys()], requiredReleaseWorkflows);
});

test("release gate bootstrap reports a failed Check dispatch", async () => {
  const attempts: string[] = [];
  await assert.rejects(
    bootstrapRequiredWorkflowRuns({
      sha: releaseSha,
      dispatchPlans: releasePlans(),
      listRuns: async () => [],
      dispatchWorkflow: async (plan) => {
        attempts.push(plan.workflowName);
        throw new Error("denied");
      },
    }),
    /Failed to dispatch release gates.*\n- Check: denied/,
  );
  assert.deepEqual(attempts, ["Check"]);
});

test("release gate bootstrap never retries or hides a red latest run", async () => {
  const plans = releasePlans();
  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  for (const plan of plans) {
    const run = findEvidenceRun(runs, plan.workflowName);
    run.display_title = plan.expectedRunName;
    run.event = "workflow_dispatch";
    run.conclusion = "failure";
  }
  const dispatched: string[] = [];

  await assert.rejects(
    bootstrapRequiredWorkflowRuns({
      sha: releaseSha,
      dispatchPlans: plans,
      listRuns: async () => runs,
      dispatchWorkflow: async (plan) => dispatched.push(plan.workflowName),
    }),
    /Check: completed\/failure/,
  );
  assert.deepEqual(dispatched, []);
});

test("release gate bootstrap bounds missing full Check evidence", async () => {
  let clock = 0;
  await assert.rejects(
    bootstrapRequiredWorkflowRuns({
      sha: releaseSha,
      dispatchPlans: releasePlans(),
      listRuns: async () => [],
      dispatchWorkflow: async () => {},
      sleep: async (milliseconds) => {
        clock += milliseconds;
      },
      now: () => clock,
      timeoutMs: 2,
      pollIntervalMs: 10,
    }),
    /Timed out.*waiting for release gates/,
  );
  assert.equal(clock, 2);
});

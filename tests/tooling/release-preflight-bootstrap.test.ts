import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
  releaseGateRunQualifiers,
} from "../../legacy-tools/github/release-preflight-bootstrap.mjs";
import {
  requiredReleaseWorkflowEvidence,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "../../legacy-tools/github/release-preflight-evidence.mjs";
import { readRepoFile } from "./support/github-workflows.ts";
import { releaseSha, successfulReleaseRun } from "./support/release-preflight.ts";

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
        workflowName: "Benchmark",
        workflowId: "benchmark.yml",
        inputs: { base_sha: "b".repeat(40), head_sha: releaseSha },
        expectedRunName: `Benchmark ${"b".repeat(40)}...${releaseSha}`,
      },
      {
        workflowName: "Fuzz",
        workflowId: "fuzz.yml",
        inputs: { mode: "replay" },
        expectedRunName: `Fuzz replay @ ${releaseSha}`,
      },
    ].map((plan) => ({ ref: "v1.2.3", ...plan })),
  );
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
      () => createReleaseGateDispatchPlans({ ref, headSha: releaseSha, baseSha: "b".repeat(40) }),
      /Release dispatch ref is required/,
    );
  }
});

test("Benchmark dispatch exposes an exact base and head range", () => {
  const benchmark = readWorkflow(findReleasePlan("Benchmark").workflowId);
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
  const fuzz = readWorkflow(findReleasePlan("Fuzz").workflowId);
  assert.equal(fuzz.on?.workflow_dispatch?.inputs?.["max-total-time"]?.default, "120");
  assert.match(fuzz["run-name"] ?? "", /^Fuzz /);
  assert.match(fuzz["run-name"] ?? "", /inputs\.max-total-time/);
  assert.match(fuzz["run-name"] ?? "", /inputs\.mode/);
  assert.match(fuzz["run-name"] ?? "", /github\.sha/);
});

test("on-demand gates correlate expanded display titles, never workflow names", () => {
  const plans = releasePlans();
  const qualifiers = releaseGateRunQualifiers(plans);
  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  assert.throws(
    () => selectRequiredWorkflowRuns(runs, releaseSha, requiredReleaseWorkflows, qualifiers),
    /Benchmark: missing workflow_dispatch run/,
  );
  for (const plan of plans) {
    const run = findEvidenceRun(runs, plan.workflowName);
    run.name = plan.expectedRunName;
    run.display_title = `wrong display title for ${plan.workflowName}`;
    run.event = "workflow_dispatch";
  }
  assert.throws(
    () => selectRequiredWorkflowRuns(runs, releaseSha, requiredReleaseWorkflows, qualifiers),
    /Benchmark: missing workflow_dispatch run/,
  );
  for (const plan of plans) {
    const run = findEvidenceRun(runs, plan.workflowName);
    run.name = plan.workflowName;
    run.display_title = plan.expectedRunName;
  }
  assert.doesNotThrow(() =>
    selectRequiredWorkflowRuns(runs, releaseSha, requiredReleaseWorkflows, qualifiers),
  );
});

test("release gate bootstrap reuses evidence that already exists at the SHA", async () => {
  const plans = releasePlans();
  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  for (const workflowName of ["Benchmark", "Fuzz"]) {
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
  // Check, Miri and Docs build are push-triggered on main, so a release
  // confirms the runs main already produced instead of waiting on new ones.
  for (const workflowName of ["Check", "Miri", "Docs build"]) {
    assert.equal(findEvidenceRun(runs, workflowName).event, "push");
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

test("release gate bootstrap attempts every missing dispatch before reporting failures", async () => {
  const attempts: string[] = [];
  await assert.rejects(
    bootstrapRequiredWorkflowRuns({
      sha: releaseSha,
      dispatchPlans: releasePlans(),
      listRuns: async () => [],
      dispatchWorkflow: async (plan) => {
        attempts.push(plan.workflowName);
        if (["Benchmark", "Fuzz"].includes(plan.workflowName)) {
          throw new Error(`dispatch denied for ${plan.workflowName}`);
        }
      },
    }),
    /Failed to dispatch release gates:[\s\S]*Benchmark: dispatch denied[\s\S]*Fuzz: dispatch denied/,
  );
  assert.deepEqual(
    attempts,
    releasePlans().map((plan) => plan.workflowName),
  );
});

test("release gate bootstrap never retries or hides a red latest run", async () => {
  const plans = releasePlans();
  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  for (const plan of plans) {
    const run = findEvidenceRun(runs, plan.workflowName);
    run.display_title = plan.expectedRunName;
    run.event = "workflow_dispatch";
  }
  const benchmark = findEvidenceRun(runs, "Benchmark");
  benchmark.conclusion = "failure";
  const dispatched: string[] = [];

  await assert.rejects(
    bootstrapRequiredWorkflowRuns({
      sha: releaseSha,
      dispatchPlans: plans,
      listRuns: async () => runs,
      dispatchWorkflow: async (plan) => dispatched.push(plan.workflowName),
    }),
    /Benchmark: completed\/failure/,
  );
  assert.deepEqual(dispatched, []);
});

test("release gate bootstrap reports a bounded wait timeout", async () => {
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
    /Timed out after 2ms[\s\S]*Check: missing/,
  );
  assert.equal(clock, 2);
});

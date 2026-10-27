import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

type Step = {
  name?: string;
  uses?: string;
  if?: string;
  run?: string;
  with?: Record<string, unknown>;
};
type ObservationWorkflow = {
  on: Record<string, unknown>;
  env: Record<string, string>;
  permissions: Record<string, string>;
  concurrency: { "cancel-in-progress": boolean };
  jobs: Record<string, { name: string; needs?: unknown; steps: Step[] }>;
};
const source = readFileSync(
  new URL("../../.github/workflows/pr-performance-observation.yml", import.meta.url),
  "utf8",
);
const workflow = parse(source) as ObservationWorkflow;

function requireIndependentObservation(candidate: ObservationWorkflow): void {
  assert.deepEqual(Object.keys(candidate.on).sort(), ["pull_request", "workflow_dispatch"]);
  assert.deepEqual(candidate.on.pull_request, { types: ["opened", "synchronize", "reopened"] });
  assert.deepEqual(candidate.permissions, { contents: "read" });
  assert.equal(candidate.concurrency["cancel-in-progress"], true);
  assert.deepEqual(Object.keys(candidate.jobs), ["observe"]);
  const job = candidate.jobs.observe;
  assert.equal(job.needs, undefined);
  assert.match(job.name, /optional/i);
  assert.ok(
    job.steps.every((step) => !/enforce-pr-budget|comment-pr|gh pr merge/.test(step.run ?? "")),
  );
  const upload = job.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.ok(upload);
  assert.equal(upload?.if, "always()");
  assert.equal(upload.with?.["retention-days"], 90);
}

test("every PR gets an independent read-only observation without a performance budget gate", () => {
  requireIndependentObservation(workflow);
  for (const file of [
    "check.yml",
    "pr-source-checks.yml",
    "queue-release-gates.yml",
    "release.yml",
  ]) {
    const existing = readFileSync(
      new URL(`../../.github/workflows/${file}`, import.meta.url),
      "utf8",
    );
    assert.doesNotMatch(
      existing,
      /pr-performance-observation|Performance observations \(optional\)/,
    );
  }
});

test("adding a privileged trigger, threshold enforcement, or a gate dependency is rejected", () => {
  for (const mutation of [
    (value: ObservationWorkflow) => {
      value.on.pull_request_target = value.on.pull_request;
    },
    (value: ObservationWorkflow) => {
      value.permissions["pull-requests"] = "write";
    },
    (value: ObservationWorkflow) => {
      value.jobs.observe.needs = "test-report";
    },
    (value: ObservationWorkflow) => {
      value.jobs.observe.steps.push({ run: "node tools/benchmarks/scripts/enforce-pr-budget.ts" });
    },
  ]) {
    const changed = structuredClone(workflow);
    mutation(changed);
    assert.throws(() => requireIndependentObservation(changed), assert.AssertionError);
  }
});

test("source roles, isolated fresh builds, and separate measurement artifacts stay explicit", () => {
  const steps = workflow.jobs.observe.steps;
  const checkouts = steps.filter((step) => step.uses?.startsWith("actions/checkout@"));
  assert.deepEqual(
    checkouts.map((step) => [step.with?.path, step.with?.ref, step.with?.["persist-credentials"]]),
    [
      ["driver", "${{ env.DRIVER_SHA }}", false],
      ["head", "${{ env.HEAD_SHA }}", false],
      ["base", "${{ env.BASE_SHA }}", false],
    ],
  );
  const build = steps.find(
    (step) => step.name === "Build normal source binaries in independent fresh targets",
  );
  assert.ok(build);
  assert.match(build.run ?? "", /for role in base head/);
  assert.match(build.run ?? "", /\$RUNNER_TEMP\/pr-performance-\$role-/);
  assert.match(build.run ?? "", /pr-performance-build\.ts/);
  assert.ok(
    steps.every((step) => !/actions\/cache@|setup-rust-sticky-cache/.test(step.uses ?? "")),
  );
  const observe = steps.find((step) => step.name?.startsWith("Observe alternating latency"));
  assert.match(observe?.run ?? "", /--base-source "\$BASE_SHA" --head-source "\$HEAD_SHA"/);
  assert.match(observe?.run ?? "", /--runs 5 --warmups 1/);
  const install = steps.findIndex(
    (step) => step.name === "Install one pinned physical dependency provider",
  );
  const controls = steps.findIndex(
    (step) => step.name === "Validate measurement and source custody controls",
  );
  assert.ok(install >= 0 && controls > install);
});

function requireWorkflowDriver(candidate: ObservationWorkflow): void {
  assert.equal(candidate.env.DRIVER_SHA, "${{ github.workflow_sha }}");
  const steps = candidate.jobs.observe.steps;
  const checkout = steps.find((step) => step.with?.path === "driver");
  assert.equal(checkout?.with?.ref, "${{ env.DRIVER_SHA }}");
  for (const name of [
    "Install one pinned physical dependency provider",
    "Validate measurement and source custody controls",
    "Generate identical public inputs and pin the native runtime",
  ]) {
    const step = steps.find((entry) => entry.name === name);
    assert.equal(
      (step as Step & { "working-directory"?: string })?.["working-directory"],
      "driver",
    );
  }
  const build = steps.find(
    (step) => step.name === "Build normal source binaries in independent fresh targets",
  );
  assert.match(
    build?.run ?? "",
    /node driver\/tools\/benchmarks\/scripts\/pr-performance-build\.ts/,
  );
  const observe = steps.find((step) => step.name?.startsWith("Observe alternating latency"));
  assert.match(
    observe?.run ?? "",
    /node driver\/tools\/benchmarks\/scripts\/pr-performance-observation\.ts/,
  );
}

test("older authored heads do not need to contain the measurement driver", () => {
  requireWorkflowDriver(workflow);
  for (const mutation of ["driver-is-head", "build-at-head", "observe-at-head"]) {
    const changed = structuredClone(workflow);
    if (mutation === "driver-is-head") changed.env.DRIVER_SHA = "${{ env.HEAD_SHA }}";
    else {
      const prefix =
        mutation === "build-at-head"
          ? "Build normal source binaries"
          : "Observe alternating latency";
      const step = changed.jobs.observe.steps.find((entry) => entry.name?.startsWith(prefix))!;
      step.run = step.run!.replaceAll("node driver/tools/", "node head/tools/");
    }
    assert.throws(() => requireWorkflowDriver(changed), assert.AssertionError);
  }
});

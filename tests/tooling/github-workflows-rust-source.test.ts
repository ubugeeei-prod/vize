import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

const SOURCE_PR_JOBS = [
  "pr-source-plan",
  "pr-rust-source",
  "pr-js-packages",
  "pr-tooling-scripts",
  "pr-playground-test",
];

type Job = {
  if?: string;
  needs?: string[] | string;
  steps?: Array<{
    name?: string;
    if?: string;
    run?: string;
    uses?: string;
    with?: Record<string, string>;
  }>;
  "timeout-minutes"?: number | string;
  uses?: string;
};
const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as {
  on?: Record<string, unknown>;
  jobs?: Record<string, Job>;
  concurrency?: { group?: string; "cancel-in-progress"?: boolean };
};
const sourceWorkflow = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
  on?: Record<string, unknown>;
  jobs?: Record<string, Job>;
};
const rustWorkflow = parse(readRepoFile(".github", "workflows", "pr-rust-checks.yml")) as {
  jobs?: Record<string, Job>;
};

test("PR and merge-group source checks are included in the required report", () => {
  assert.equal(
    workflow.jobs?.["pr-source-checks"]?.if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
  );
  assert.equal(
    workflow.jobs?.["pr-source-checks"]?.uses,
    "./.github/workflows/pr-source-checks.yml",
  );
  assert.ok(Object.hasOwn(sourceWorkflow.on ?? {}, "workflow_call"));
  for (const [job, minutes] of [
    ["pr-source-plan", 5],
    ["pr-js-packages", 35],
    ["pr-tooling-scripts", 35],
    ["pr-playground-test", 60],
    ["source-report", 5],
  ] as const) {
    assert.equal(sourceWorkflow.jobs?.[job]?.["timeout-minutes"], minutes);
  }
  for (const job of SOURCE_PR_JOBS) {
    assert.equal(
      sourceWorkflow.jobs?.[job]?.if,
      "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
    );
  }
  for (const job of SOURCE_PR_JOBS.filter(
    (name) => !["pr-source-plan", "pr-rust-source"].includes(name),
  )) {
    const steps = sourceWorkflow.jobs?.[job]?.steps ?? [];
    assert.ok(steps.length > 0, `${job} needs a skip explanation or validation steps`);
    assert.ok(
      steps.every((step) => step.if),
      `${job} must guard every expensive step`,
    );
  }
  assert.deepEqual(sourceWorkflow.jobs?.["pr-rust-source"]?.needs, "pr-source-plan");
  assert.deepEqual(sourceWorkflow.jobs?.["pr-js-packages"]?.needs, "pr-source-plan");
  assert.deepEqual(sourceWorkflow.jobs?.["pr-tooling-scripts"]?.needs, "pr-source-plan");
  assert.deepEqual(sourceWorkflow.jobs?.["pr-playground-test"]?.needs, "pr-source-plan");
  const commands = (job: string) =>
    (sourceWorkflow.jobs?.[job]?.steps ?? []).map((step) => step.run ?? "").join("\n");
  assert.equal(
    sourceWorkflow.jobs?.["pr-rust-source"]?.uses,
    "./.github/workflows/pr-rust-checks.yml",
  );
  const rustCommands = (job: string) =>
    (rustWorkflow.jobs?.[job]?.steps ?? []).map((step) => step.run ?? "").join("\n");
  assert.match(rustCommands("merge-rust-source"), /cargo clippy --workspace/);
  assert.match(rustCommands("merge-rust-source"), /cargo test --workspace/);
  assert.match(rustCommands("merge-rust-source"), /write-coverage-summary\.rs/);
  assert.match(rustCommands("pr-rust-build"), /cargo nextest archive @packages@/);
  assert.match(rustCommands("pr-rust-build"), /cargo test @packages@ --profile ci --doc/);
  assert.match(rustCommands("pr-rust-shard"), /cargo nextest run --archive-file/);
  const rustSteps = rustWorkflow.jobs?.["merge-rust-source"]?.steps ?? [];
  const pklIndex = rustSteps.findIndex((step) => step.name === "Install Pkl CLI");
  const buildIndex = rustSteps.findIndex((step) => step.name === "Build Rust workspace tests");
  const runIndex = rustSteps.findIndex((step) => step.name === "Run Rust workspace doctests");
  assert.ok(
    pklIndex >= 0 && buildIndex >= 0 && pklIndex < buildIndex,
    "Pkl fixtures need the CLI before workspace test compilation",
  );
  assert.ok(runIndex > buildIndex, "Workspace test compilation must precede execution");
  assert.match(
    rustSteps[buildIndex]?.run ?? "",
    /cargo nextest archive --workspace --cargo-profile ci --timings/,
  );
  assert.match(rustSteps[runIndex]?.run ?? "", /cargo test --workspace --profile ci --doc(?:;|$)/m);
  assert.match(commands("pr-js-packages"), /vp run --workspace-root test:js/);
  assert.match(commands("pr-js-packages"), /vp run --filter '\.\/npm\/ui' check/);
  assert.match(commands("pr-tooling-scripts"), /vp run --workspace-root test:scripts/);
  assert.match(commands("pr-playground-test"), /vp run --filter '\.\/playground' test:browser/);
  assert.equal(sourceWorkflow.jobs?.["source-report"]?.if, "${{ always() }}");
  assert.deepEqual(sourceWorkflow.jobs?.["source-report"]?.needs, SOURCE_PR_JOBS);
  assert.match(commands("source-report"), /require-needs-success\.mjs/);
});

test("untrusted source checks cannot write trusted sticky disks", () => {
  const action = parse(
    readRepoFile(".github", "actions", "setup-rust-sticky-cache", "action.yml"),
  ) as {
    inputs?: Record<string, { default?: string }>;
    runs?: { steps?: Array<{ uses?: string; with?: Record<string, string> }> };
  };
  assert.equal(action.inputs?.["cache-key-prefix"]?.default, "");
  const prefix =
    "${{ github.event_name == 'pull_request' && format('pr-{0}-', github.event.pull_request.number) || format('merge-{0}-', github.sha) }}";
  for (const job of SOURCE_PR_JOBS.filter(
    (name) => !["pr-source-plan", "pr-rust-source"].includes(name),
  )) {
    const cacheStep = sourceWorkflow.jobs?.[job]?.steps?.find(
      (step) => step.uses === "./.github/actions/setup-rust-sticky-cache",
    );
    assert.equal(cacheStep?.with?.["cache-key-prefix"], prefix, `${job} must isolate its cache`);
  }
  for (const job of ["merge-rust-source", "pr-rust-build"]) {
    const cacheStep = rustWorkflow.jobs?.[job]?.steps?.find(
      (step) => step.uses === "./.github/actions/setup-rust-sticky-cache",
    );
    assert.ok(
      cacheStep?.with?.["cache-key-prefix"]?.includes("pr-{0}-"),
      `${job} must isolate its cache`,
    );
  }
  const mounts = action.runs?.steps?.filter((step) =>
    step.uses?.startsWith("useblacksmith/stickydisk@"),
  );
  assert.equal(mounts?.length, 4, "registry, git, primary, and secondary disks need isolation");
  for (const mount of mounts ?? []) {
    assert.ok(
      mount.with?.key?.startsWith("${{ github.repository }}-${{ inputs.cache-key-prefix }}"),
      `shared cache key: ${mount.with?.key}`,
    );
  }
});

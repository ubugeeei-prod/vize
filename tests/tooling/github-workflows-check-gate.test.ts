import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";

import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.mjs";
import { readRepoFile, root } from "./support/github-workflows.ts";

const PR_JOBS = ["fmt-rust", "check-js", "security-audit", "node-engine-compat", "check-vize-apps"];
const FULL_SUITE_JOBS = [
  "nix-flake",
  "vue-parity",
  "test-scripts",
  "editor-extensions",
  "editor-host-smoke",
  "build-js-packages",
  "test-js-packages",
  "clippy-and-test",
  "coverage",
  "source-coverage",
  "branch-coverage",
  "playground-test",
];

type Job = {
  if?: string;
  needs?: string[];
  steps?: Array<{ name?: string; if?: string; run?: string; uses?: string }>;
};
const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as {
  on?: Record<string, unknown>;
  jobs?: Record<string, Job>;
  concurrency?: { group?: string; "cancel-in-progress"?: boolean };
};

function needs(results: Record<string, string> = {}): Record<string, { result: string }> {
  return Object.fromEntries(PR_JOBS.map((job) => [job, { result: results[job] ?? "success" }]));
}

test("obsolete main validation is cancelled when the branch advances", () => {
  for (const candidate of [
    workflow,
    parse(readRepoFile(".github", "workflows", "davinci-contracts.yml")) as typeof workflow,
  ]) {
    assert.equal(candidate.concurrency?.["cancel-in-progress"], true);
    assert.match(
      candidate.concurrency?.group ?? "",
      /github\.event\.pull_request\.number \|\| github\.ref/,
    );
    assert.doesNotMatch(candidate.concurrency?.group ?? "", /github\.sha/);
  }
});

test("PR and main push stay fast while full checks require schedule or dispatch", () => {
  assert.ok(workflow.on?.pull_request);
  assert.ok(workflow.on?.push);
  assert.ok(workflow.on?.schedule);
  assert.ok(Object.hasOwn(workflow.on ?? {}, "workflow_dispatch"));
  assert.deepEqual(workflow.jobs?.["test-report"]?.needs, PR_JOBS);
  for (const job of PR_JOBS) {
    assert.equal(workflow.jobs?.[job]?.if, undefined, `${job} must run on pull requests`);
  }
  for (const job of FULL_SUITE_JOBS) {
    assert.equal(
      workflow.jobs?.[job]?.if,
      "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
    );
  }
  assert.equal(
    workflow.jobs?.["nix-flake"]?.if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  assert.equal(
    workflow.jobs?.["semver-checks"]?.if,
    "${{ github.event_name == 'workflow_dispatch' && (startsWith(github.ref, 'refs/heads/release/v') || startsWith(github.ref, 'refs/tags/v')) }}",
  );
  assert.equal(
    workflow.jobs?.["test-report"]?.steps?.at(-1)?.run,
    "node tools/support/compat/github/require-needs-success.mjs",
  );
  const commands = (job: string) =>
    (workflow.jobs?.[job]?.steps ?? []).map((step) => step.run ?? "").join("\n");
  const checkSteps = workflow.jobs?.["check-js"]?.steps ?? [];
  assert.equal(
    checkSteps.find((step) => step.name === "Check fast JS/TS")?.if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'push' }}",
  );
  assert.equal(
    checkSteps.find((step) => step.name === "Check JS/TS")?.if,
    "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}",
  );
  const inventory = checkSteps.find((step) => step.name === "Check Davinci source inventories");
  assert.ok(inventory);
  assert.equal(
    inventory.if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'push' }}",
  );
  assert.deepEqual(
    inventory.run
      ?.trim()
      .split("\n")
      .map((line) => line.trim()),
    [
      "node tools/support/compat/davinci/croquis-consumers.mjs --check",
      "node tools/support/compat/davinci/consumer-migration-surfaces.mjs --check",
      "node --test tests/tooling/davinci-storage-policy.test.ts",
      "node tools/support/compat/davinci/storage-summary.mjs --check",
    ],
  );
  assert.ok(
    checkSteps.indexOf(inventory) <
      checkSteps.findIndex((step) => step.uses === "./.github/actions/setup-js-check-runtime"),
    "inventory checks must fail before the heavy JS check setup",
  );
  assert.match(commands("check-js"), /vp run --workspace-root check:repo/);
  assert.match(commands("check-js"), /vp run --workspace-root check:ci/);
  assert.match(commands("check-vize-apps"), /cargo build --profile ci -p vize/);
  assert.match(commands("test-scripts"), /vp run --workspace-root test:scripts/);
  assert.match(commands("test-js-packages"), /vp run --workspace-root test:js/);
  assert.match(commands("clippy-and-test"), /cargo clippy --workspace/);
  assert.match(commands("clippy-and-test"), /cargo test --workspace/);
  assert.match(commands("clippy-and-test"), /cargo bench -p vize_s1_to_s2/);
  assert.match(commands("branch-coverage"), /coverage:source:branch/);
  assert.match(commands("playground-test"), /test:browser/);
  assert.deepEqual(workflow.jobs?.["playground-test"]?.needs, ["build-js-packages"]);
});

test("report fails closed when any PR check fails or skips", () => {
  assert.equal(aggregateNeedsResults(needs()).exitCode, 0);
  for (const result of ["failure", "cancelled", "skipped"] as const) {
    const decision = aggregateNeedsResults(needs({ "check-js": result }));
    assert.equal(decision.exitCode, 1);
    assert.match(decision.message, new RegExp(`check-js: ${result}`));
  }
  assert.throws(() => aggregateNeedsResults({}), /needs context is empty/);
});

test("report command exits nonzero for a failed dependency", () => {
  const script = "tools/support/compat/github/require-needs-success.mjs";
  const result = spawnSync(process.execPath, [script], {
    cwd: root,
    encoding: "utf8",
    env: { ...process.env, NEEDS_JSON: JSON.stringify(needs({ "check-vize-apps": "failure" })) },
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /check-vize-apps: failure/);
});

test("slow suites use schedules or explicit dispatch without starting on PRs", () => {
  const pushOrDispatch = [
    "davinci-contracts.yml",
    "davinci-html-content-model.yml",
    "davinci-incremental.yml",
    "davinci-moonbit.yml",
  ];
  const scheduledOrDispatch = [
    "benchmark.yml",
    "check-bench.yml",
    "content-mapper-conformance.yml",
    "davinci-lean.yml",
    "davinci-resource-budgets.yml",
    "e2e.yml",
    "editor-conformance.yml",
    "fresco.yml",
    "fuzz.yml",
    "miri.yml",
    "tool-benchmark.yml",
    "vue-benchmarks-replay.yml",
  ];
  for (const name of [
    ...pushOrDispatch,
    ...scheduledOrDispatch,
    "criterion-bench.yml",
    "pkg-pr-new.yml",
  ]) {
    const events =
      (parse(readRepoFile(".github", "workflows", name)) as { on?: Record<string, unknown> }).on ??
      {};
    assert.equal(Object.hasOwn(events, "pull_request"), false, name);
    assert.equal(Object.hasOwn(events, "workflow_dispatch"), true, name);
    if (pushOrDispatch.includes(name)) assert.equal(Object.hasOwn(events, "push"), true, name);
    if (scheduledOrDispatch.includes(name))
      assert.equal(Object.hasOwn(events, "schedule"), true, name);
    if (scheduledOrDispatch.includes(name) || name === "pkg-pr-new.yml")
      assert.equal(Object.hasOwn(events, "push"), false, name);
  }
});

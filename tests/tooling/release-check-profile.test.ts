import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { runInNewContext } from "node:vm";
import { test } from "node:test";
import { parse } from "yaml";
import {
  assertRequiredWorkflowJobs,
  requiredWorkflowJobNames,
} from "../../tools/support/compat/github/release-preflight-evidence.mjs";
import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

type Step = { name?: string; uses?: string; run?: string; if?: string; env?: object };
type Job = {
  if?: string;
  needs?: string[];
  env?: object;
  steps: Step[];
  strategy?: { matrix?: { crate?: string[] } };
};
const workflow = parse(checkWorkflowJobSources()) as {
  jobs: Record<string, Job>;
};
const jobs = workflow.jobs;
const producers = [
  "rust-clippy-portability",
  "rust-davinci-allocations",
  "rust-native-differential",
];

function condition(expression: string | undefined, event: string, ref: string, profile: boolean) {
  if (!expression) return true;
  assert.ok(expression.startsWith("${{") && expression.endsWith("}}"));
  // These expressions use the common Boolean/property/function subset of Actions.
  // Hyphenated Actions property names need bracket syntax in JavaScript.
  const source = expression
    .slice(3, -2)
    .replaceAll("inputs.release-qualification", 'inputs["release-qualification"]');
  const result: unknown = runInNewContext(source, {
    github: { event_name: event, ref },
    inputs: { "release-qualification": profile },
    startsWith: (value: string, prefix: string) => value.startsWith(prefix),
    always: () => true,
    cancelled: () => false,
  });
  assert.equal(typeof result, "boolean");
  return result;
}

test("only explicit release qualification omits instrumented source measurement", () => {
  for (const [event, ref, profile, expected] of [
    ["schedule", "refs/heads/main", false, true],
    ["schedule", "refs/heads/main", true, true],
    ["workflow_dispatch", "refs/heads/main", false, true],
    ["workflow_dispatch", "refs/heads/main", true, true],
    ["workflow_dispatch", "refs/heads/fix/example", false, true],
    ["workflow_dispatch", "refs/heads/fix/example", true, true],
    ["workflow_dispatch", "refs/heads/release/v0.458.0", false, true],
    ["workflow_dispatch", "refs/tags/v0.458.0", false, true],
    ["workflow_dispatch", "refs/heads/release/v0.458.0", true, false],
    ["workflow_dispatch", "refs/tags/v0.458.0", true, false],
    ["workflow_dispatch", "refs/heads/release/v0.458.0-beta.1", true, false],
    ["pull_request", "refs/pull/1/merge", false, false],
    ["merge_group", "refs/heads/gh-readonly-queue/main/example", false, false],
    ["push", "refs/heads/main", false, false],
  ] as const) {
    assert.equal(
      condition(jobs["source-coverage"].if, event, ref, profile),
      expected,
      `${event} ${ref} release-qualification=${profile}`,
    );
  }
});

test("explicit profile refuses nonrelease and malformed release refs through its actual shell", () => {
  const step = jobs["iteration-plan"].steps.find(
    (candidate) => candidate.name === "Validate explicit release qualification",
  );
  assert.ok(step?.run);
  for (const ref of [
    "refs/heads/release/v0.458.0",
    "refs/tags/v0.458.0",
    "refs/heads/release/v0.458.0-beta.1",
  ]) {
    assert.equal(condition(step.if, "workflow_dispatch", ref, true), true);
    assert.equal(
      spawnSync("bash", ["-e", "-c", step.run], {
        env: { ...process.env, RELEASE_QUALIFICATION_REF: ref },
      }).status,
      0,
      ref,
    );
  }
  for (const ref of [
    "refs/heads/main",
    "refs/heads/fix/example",
    "refs/heads/release/v",
    "refs/heads/release/v0.458",
    "refs/heads/release/v0.458.0/extra",
    "refs/tags/v0.458.0;true",
  ]) {
    assert.equal(
      spawnSync("bash", ["-e", "-c", step.run], {
        env: { ...process.env, RELEASE_QUALIFICATION_REF: ref },
      }).status,
      1,
      ref,
    );
  }
  assert.equal(condition(step.if, "workflow_dispatch", "refs/heads/main", false), false);
});

test("release profile retains legacy nineteen roles and the real editor, tooling and coverage lanes", () => {
  const retained = [
    "test-scripts-gate",
    "test-js-packages",
    "build-js-packages",
    "full-js-report",
    "full-rust-workspace",
    "clippy-and-test",
    "full-rust-report",
    "semver-checks",
    "editor-extensions",
    "editor-host-smoke",
    "full-source-cli",
    "tooling-plan",
    "test-scripts",
    "coverage",
    "branch-coverage",
    ...producers,
  ];
  for (const ref of ["refs/heads/release/v0.458.0", "refs/tags/v0.458.0"]) {
    for (const id of retained) {
      assert.ok(jobs[id], id);
      assert.equal(condition(jobs[id].if, "workflow_dispatch", ref, true), true, id);
      assert.equal(condition(jobs[id].if, "workflow_dispatch", ref, false), true, id);
    }
    assert.equal(
      requiredWorkflowJobNames("Check", {
        event: "workflow_dispatch",
        head_branch: ref.replace(/^refs\/(?:heads|tags)\//u, ""),
      }).length,
      19,
    );
  }
  assert.equal(jobs["semver-checks"].strategy?.matrix?.crate?.length, 12);
  for (const id of producers) {
    assert.equal(jobs[id].needs, undefined, `${id} must run independently`);
    assert.equal(condition(jobs[id].if, "schedule", "refs/heads/main", false), true);
    assert.equal(condition(jobs[id].if, "pull_request", "refs/pull/1/merge", false), false);
  }
});

test("server-bound full workflow requires eight actual product jobs and the unchanged twelve API jobs", () => {
  const head = "a".repeat(40);
  const referenced = {
    path: `owner/repo/.github/workflows/full-qualification.yml@${head}`,
    sha: head,
    ref: "refs/heads/release/v0.500.0",
  };
  const run = {
    name: `Check full @ ${head}`,
    display_title: `Check full @ ${head}`,
    path: ".github/workflows/check.yml",
    event: "workflow_dispatch",
    head_sha: head,
    head_branch: "release/v0.500.0",
    repository: { full_name: "owner/repo" },
    referenced_workflows: [referenced],
  };
  const products = [
    "Full qualification / test-scripts",
    "Full qualification / test-js-packages",
    "Full qualification / build-js-packages",
    "full-js-report",
    "Full qualification / Full Rust workspace and doctests",
    "Full qualification / clippy-and-test",
    "Full qualification / full-rust-report",
    "Full qualification / Nuxt module qualification / Nuxt 3 build and SSR",
  ];
  const names = requiredWorkflowJobNames("Check", run);
  assert.deepEqual(names.slice(0, 8), products);
  assert.deepEqual(
    names.slice(8),
    requiredWorkflowJobNames("Check", {
      event: "workflow_dispatch",
      head_branch: run.head_branch,
    }).slice(7),
  );
  assert.equal(names.length, 20);
  assert.deepEqual(
    requiredWorkflowJobNames("Check", {
      ...run,
      event: "schedule",
      head_branch: "main",
      referenced_workflows: [{ ...referenced, ref: "refs/heads/main" }],
      name: `Check schedule @ ${head}`,
      display_title: `Check schedule @ ${head}`,
    }),
    products,
  );
  const successful = names.map((name) => ({ name, status: "completed", conclusion: "success" }));
  assert.doesNotThrow(() => assertRequiredWorkflowJobs("Check", successful, run));
  for (const name of names) {
    for (const state of ["missing", "duplicate", "skipped", "failure", "cancelled", "pending"]) {
      const changed = successful.flatMap((job) => {
        if (job.name !== name) return [job];
        if (state === "missing") return [];
        if (state === "duplicate") return [job, job];
        return [
          { ...job, ...(state === "pending" ? { status: "in_progress" } : { conclusion: state }) },
        ];
      });
      assert.throws(() => assertRequiredWorkflowJobs("Check", changed, run), `${name}/${state}`);
    }
  }
  for (const changed of [
    { ...run, name: "Other" },
    { ...run, name: "Check" },
    { ...run, name: `Check full @ ${"b".repeat(40)}` },
    { ...run, display_title: "Other" },
    { ...run, event: "pull_request" },
    { ...run, event: "schedule" },
    { ...run, path: ".github/workflows/other.yml" },
    { ...run, referenced_workflows: [referenced, referenced] },
    { ...run, referenced_workflows: [{ ...referenced, sha: "b".repeat(40) }] },
    { ...run, referenced_workflows: [{ ...referenced, ref: "refs/heads/main" }] },
    {
      ...run,
      referenced_workflows: [
        { ...referenced, path: `foreign/repo/.github/workflows/full-qualification.yml@${head}` },
      ],
    },
    { ...run, referenced_workflows: [] },
    { ...run, referenced_workflows: null },
  ])
    assert.throws(() => assertRequiredWorkflowJobs("Check", successful, changed));
});

test("parallel Rust jobs preserve every original step, command byte and runtime setting", () => {
  const original = (
    parse(
      readRepoFile("tests", "_fixtures", "ci", "check-clippy-and-test.original.yml.txt"),
    ) as Record<string, Job>
  )["clippy-and-test"];
  const firstCommand = original.steps.findIndex((step) => step.name === "Clippy");
  assert.ok(firstCommand > 0);
  const setup = original.steps.slice(0, firstCommand);
  const tasks: Step[] = [];
  for (const id of producers) {
    assert.deepEqual(jobs[id].env, original.env, `${id} changed runtime settings`);
    assert.deepEqual(jobs[id].steps.slice(0, firstCommand), setup, `${id} changed setup`);
    tasks.push(...jobs[id].steps.slice(firstCommand));
  }
  assert.deepEqual(tasks, original.steps.slice(firstCommand));
  const aggregate = jobs["clippy-and-test"];
  assert.deepEqual(aggregate.needs, producers);
  assert.equal(
    aggregate.if,
    "${{ always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  assert.deepEqual(jobs["full-rust-report"].needs, ["full-rust-workspace", "clippy-and-test"]);
});

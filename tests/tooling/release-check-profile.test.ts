import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { runInNewContext } from "node:vm";
import { test } from "node:test";
import { parse } from "yaml";
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

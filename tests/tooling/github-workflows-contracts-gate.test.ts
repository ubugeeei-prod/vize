import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";

import {
  aggregateCheckNeedsResults,
  aggregateNeedsResults,
} from "../../tools/support/compat/github/require-needs-success.mjs";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Workflow = {
  on: Record<string, unknown>;
  concurrency: { group: string; "cancel-in-progress": boolean };
  permissions: Record<string, string>;
  jobs: Record<
    string,
    { if?: string; uses?: string; needs?: string[]; steps?: { run?: string }[] }
  >;
};
const check = parse(readRepoFile(".github/workflows/check.yml")) as Workflow;
const source = parse(readRepoFile(".github/workflows/pr-source-checks.yml")) as Workflow;
const contracts = parse(readRepoFile(".github/workflows/davinci-contracts.yml")) as Workflow;
const topPrior = [
  "fmt-rust",
  "check-js",
  "security-audit",
  "node-engine-compat",
  "check-vize-apps",
  "pr-source-checks",
  "instruction-counts",
  "level-dependency-direction",
];
const prior = [
  "pr-source-plan",
  "pr-rust-source",
  "pr-js-packages",
  "pr-tooling-scripts",
  "pr-playground-test",
];
const needs = (wit = "success") => ({
  ...Object.fromEntries(prior.map((job) => [job, { result: "success" }])),
  "wit-contracts": { result: wit },
});

test("one parallel reusable WIT lane gates merge groups without adding a PR build", () => {
  assert.equal(Object.hasOwn(check.jobs, "wit-contracts"), false);
  assert.deepEqual(check.jobs["pr-source-checks"], {
    name: "PR source checks",
    permissions: { contents: "read", actions: "read" },
    if: "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
    uses: "./.github/workflows/pr-source-checks.yml",
  });
  assert.deepEqual(source.on, { workflow_call: null });
  assert.deepEqual(source.jobs["wit-contracts"], {
    name: "Guest contracts",
    if: "${{ github.event_name == 'merge_group' }}",
    uses: "./.github/workflows/davinci-contracts.yml",
  });
  assert.deepEqual(source.jobs["source-report"].needs, [
    ...prior,
    "wit-contracts",
    "canonical-corpus",
  ]);
  assert.equal(source.jobs["source-report"].if, "${{ always() }}");
  assert.equal(
    source.jobs["source-report"].steps?.at(-1)?.run,
    "node tools/support/compat/github/canonical-corpus-selection.mjs --check",
  );
  assert.deepEqual(check.jobs["test-report"].needs, topPrior);
  assert.equal(
    check.jobs["test-report"].if,
    "${{ always() && (github.event_name == 'pull_request' || github.event_name == 'merge_group') }}",
  );
  assert.equal(
    check.jobs["test-report"].steps?.at(-1)?.run,
    "node tools/support/compat/github/require-needs-success.mjs",
  );
  assert.equal(Object.hasOwn(contracts.on, "workflow_call"), true);
  assert.equal(Object.hasOwn(contracts.on, "push"), true);
  assert.equal(Object.hasOwn(contracts.on, "workflow_dispatch"), true);
  assert.equal(Object.hasOwn(contracts.on, "merge_group"), false);
  assert.equal(Object.hasOwn(contracts.on, "pull_request"), false);
  assert.deepEqual(contracts.permissions, { contents: "read" });
  assert.deepEqual(check.permissions, { contents: "read" });
  assert.deepEqual(source.permissions, { contents: "read" });
  assert.equal(contracts.jobs["wit-contract"].if, undefined);
  assert.equal(contracts.concurrency["cancel-in-progress"], true);
  assert.equal(check.concurrency["cancel-in-progress"], true);
  assert.equal(
    contracts.concurrency.group,
    "davinci-contracts-${{ github.workflow }}-${{ github.event.pull_request.number || github.ref }}",
  );
  assert.equal(
    check.concurrency.group,
    "check-v3-${{ github.workflow }}-${{ (github.event_name == 'workflow_dispatch' || github.event_name == 'schedule') && format('full-{0}', github.sha) || (github.event.pull_request.number || github.ref) }}",
  );
});

test("failed, skipped and absent nested WIT results poison the existing required report", () => {
  const outer = (sourceResult: string) =>
    aggregateNeedsResults({
      ...Object.fromEntries(topPrior.map((job) => [job, { result: "success" }])),
      "pr-source-checks": { result: sourceResult },
    });
  for (const result of ["failure", "cancelled", "skipped"]) {
    const inner = aggregateCheckNeedsResults(needs(result), "merge_group");
    assert.equal(inner.exitCode, 1);
    assert.equal(outer(inner.exitCode === 0 ? "success" : "failure").exitCode, 1);
  }
  for (const rows of [{}, { ...needs(), "wit-contracts": {} }]) {
    assert.throws(() => aggregateCheckNeedsResults(rows, "merge_group"));
    assert.equal(outer("failure").exitCode, 1);
  }
  assert.equal(outer("skipped").exitCode, 1);
  for (const [event, wit] of [
    ["pull_request", "skipped"],
    ["merge_group", "success"],
  ]) {
    const inner = aggregateCheckNeedsResults(needs(wit), event);
    assert.equal(inner.exitCode, 0);
    assert.equal(outer(inner.exitCode === 0 ? "success" : "failure").exitCode, 0);
  }
});

test("only the queue-only row may skip in the explicit supported nonqueue modes", () => {
  for (const event of ["pull_request", "push", "schedule", "workflow_dispatch"]) {
    assert.equal(aggregateCheckNeedsResults(needs("skipped"), event).exitCode, 0);
    for (const job of prior) {
      for (const result of ["skipped", "failure", "cancelled"]) {
        const rows = { ...needs("skipped"), [job]: { result } };
        assert.equal(aggregateCheckNeedsResults(rows, event).exitCode, 1);
      }
    }
    for (const result of ["failure", "cancelled"]) {
      assert.equal(aggregateCheckNeedsResults(needs(result), event).exitCode, 1);
    }
  }
  assert.equal(aggregateNeedsResults(needs("skipped")).exitCode, 1);
});

test("queue refusals, absent results and unknown events fail closed", () => {
  assert.equal(aggregateCheckNeedsResults(needs(), "merge_group").exitCode, 0);
  for (const result of ["skipped", "failure", "cancelled"]) {
    assert.equal(aggregateCheckNeedsResults(needs(result), "merge_group").exitCode, 1);
  }
  for (const event of ["merge_group", "pull_request", "push", "schedule", "workflow_dispatch"]) {
    assert.throws(() => aggregateCheckNeedsResults({}, event), /missing wit-contracts/);
    assert.throws(
      () => aggregateCheckNeedsResults({ ...needs(), "wit-contracts": {} }, event),
      /wit-contracts reported no result/,
    );
  }
  for (const event of [undefined, "", "workflow_call", "pull_request_target", "unknown"]) {
    assert.throws(
      () => aggregateCheckNeedsResults(needs("skipped"), event),
      /Unsupported Check event/,
    );
  }
});

test("the actual Check CLI fails queue omissions while the shared strict mode stays exact", () => {
  const script = "tools/support/compat/github/require-needs-success.mjs";
  const run = (args: string[], event: string, rows: Record<string, { result: string }>) =>
    spawnSync(process.execPath, [script, ...args], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, GITHUB_EVENT_NAME: event, NEEDS_JSON: JSON.stringify(rows) },
    });
  assert.equal(run(["--check"], "pull_request", needs("skipped")).status, 0);
  assert.equal(run(["--check"], "workflow_dispatch", needs("skipped")).status, 0);
  assert.equal(run(["--check"], "merge_group", needs()).status, 0);
  assert.equal(run(["--check"], "merge_group", needs("skipped")).status, 1);
  assert.equal(run(["--check"], "unknown", needs("skipped")).status, 1);
  assert.equal(run([], "pull_request", { old: { result: "success" } }).status, 0);
  assert.equal(run([], "pull_request", { old: { result: "skipped" } }).status, 1);
  assert.equal(run(["--unknown"], "pull_request", needs()).status, 1);
});

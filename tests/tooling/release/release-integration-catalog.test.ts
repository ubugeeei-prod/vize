import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { integrationEvent } from "../../../tools/support/compat/github/release-integration-event.mjs";
import { aggregateNeedsResults } from "../../../tools/support/compat/github/require-needs-success.mjs";
import { readRepoFile } from "../support/github-workflows.ts";

const base = "b".repeat(40);
const candidate = "c".repeat(40);
const repository = "owner/repo";
const pull = {
  number: 99,
  head: { ref: "release-integration/v1.2.3" },
  body: "<!-- vize-release-pin-source: 42 -->",
};
const event = {
  repository: { full_name: repository },
  number: 99,
  pull_request: pull,
  merge_group: {
    head_ref: `refs/heads/gh-readonly-queue/main/pr-99-${base}`,
    base_ref: "refs/heads/main",
    head_sha: candidate,
    base_sha: base,
  },
};

test("integration planning selects its own PR and queue root while ordinary PRs need no catalog", () => {
  assert.equal(integrationEvent("pull_request", event, repository, candidate), "99");
  assert.equal(
    integrationEvent("merge_group", event, repository, candidate, (number: number) => {
      assert.equal(number, 99);
      return pull;
    }),
    "99",
  );
  const ordinary = { ...pull, head: { ref: "fix/ordinary" }, body: "ordinary" };
  assert.equal(
    integrationEvent("pull_request", { ...event, pull_request: ordinary }, repository, candidate),
    "",
  );
  assert.equal(
    integrationEvent("merge_group", event, repository, candidate, () => ordinary),
    "",
  );
  const source = {
    ...pull,
    head: { ref: "release/v1.2.3" },
    body: "<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-head: abc -->",
  };
  assert.equal(
    integrationEvent("pull_request", { ...event, pull_request: source }, repository, candidate),
    "",
  );
});

test("ordinary batched queue roots succeed while official integrations reject ambiguous bases", () => {
  const batched = { ...event, merge_group: { ...event.merge_group, base_sha: "d".repeat(40) } };
  const ordinary = { ...pull, head: { ref: "fix/ordinary" }, body: "ordinary" };
  assert.equal(
    integrationEvent("merge_group", batched, repository, candidate, () => ordinary),
    "",
  );
  assert.throws(() => integrationEvent("merge_group", batched, repository, candidate, () => pull));
  assert.throws(() =>
    integrationEvent(
      "merge_group",
      { ...batched, merge_group: { ...batched.merge_group, base_sha: "short" } },
      repository,
      candidate,
      () => ordinary,
    ),
  );
});

test("integration planning rejects missing, foreign and ambiguous event identities", () => {
  for (const mutated of [
    { ...event, repository: { full_name: "foreign/repo" } },
    { ...event, number: 98 },
    { ...event, pull_request: { ...pull, head: { ref: "fix/ordinary" } } },
  ]) {
    assert.throws(() => integrationEvent("pull_request", mutated, repository, candidate));
  }
  for (const changes of [
    { head_ref: `refs/heads/gh-readonly-queue/main/pr-98-${base}` },
    { head_ref: `refs/heads/gh-readonly-queue/main/pr-99-${candidate}` },
    { head_ref: "refs/heads/other" },
    { head_sha: base },
    { base_sha: candidate },
    { base_sha: "short" },
    { base_ref: "refs/heads/other" },
  ]) {
    assert.throws(() =>
      integrationEvent(
        "merge_group",
        { ...event, merge_group: { ...event.merge_group, ...changes } },
        repository,
        candidate,
        () => pull,
      ),
    );
  }
  assert.throws(() => integrationEvent("push", event, repository, candidate));
  assert.throws(() => integrationEvent("pull_request", event, repository, "short"));
});

test("required Check report consumes the always-running catalog gate without allowing skips", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "check.yml"));
  const wrapper = workflow.jobs["release-integration-catalog"];
  const catalogWorkflow = parse(
    readRepoFile(".github", "workflows", "release-integration-catalog.yml"),
  );
  const gate = catalogWorkflow.jobs["release-integration-catalog"];
  assert.equal(wrapper.uses, "./.github/workflows/release-integration-catalog.yml");
  assert.equal(wrapper.if, gate.if);
  assert.deepEqual(wrapper.permissions, gate.permissions);
  assert.deepEqual(catalogWorkflow.on, { workflow_call: null });
  assert.equal(
    gate.if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
  );
  assert.deepEqual(gate.permissions, { contents: "read", "pull-requests": "read" });
  assert.ok(workflow.jobs["test-report"].needs.includes("release-integration-catalog"));
  assert.equal(gate.steps[1].run, "node tools/support/compat/github/release-integration-event.mjs");
  for (const step of gate.steps.slice(2)) {
    assert.equal(step.if, "${{ steps.candidate.outputs.integration != '' }}");
  }
  assert.equal(gate.steps[2].with["fetch-depth"], 0);
  assert.equal(
    gate.steps.at(-1).run,
    'rust-script tools/commands/release/pr.rs check-integration-candidate "$INTEGRATION_PR"',
  );
  for (const result of ["failure", "cancelled", "skipped"]) {
    assert.equal(aggregateNeedsResults({ "release-integration-catalog": { result } }).exitCode, 1);
  }
  assert.equal(
    aggregateNeedsResults({ "release-integration-catalog": { result: "success" } }).exitCode,
    0,
  );
});

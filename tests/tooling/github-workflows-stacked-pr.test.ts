import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

test("Check validates pull requests against every stacked base branch", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "check.yml"));
  assert.ok(Object.hasOwn(workflow.on, "pull_request"));
  assert.equal(workflow.on.pull_request?.branches, undefined);
  assert.equal(workflow.on.pull_request?.["branches-ignore"], undefined);
  assert.equal(workflow.on.pull_request?.paths, undefined);
  assert.equal(workflow.on.pull_request?.["paths-ignore"], undefined);
  assert.deepEqual(workflow.on.push.branches, ["main", "davinci"]);
  assert.deepEqual(workflow.on.merge_group.branches, ["main"]);
});

test("stacked pull requests retain the trusted title policy", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "title-policy.yml"));
  const trigger = workflow.on.pull_request_target;
  assert.ok(trigger);
  assert.equal(trigger.branches, undefined);
  assert.equal(trigger["branches-ignore"], undefined);
  assert.deepEqual(trigger.types, [
    "opened",
    "edited",
    "reopened",
    "synchronize",
    "ready_for_review",
  ]);
  assert.equal(Object.hasOwn(workflow.on, "pull_request"), false);
  const steps = workflow.jobs["issue-pr-title-policy"].steps;
  const checkout = steps.find((step: { uses?: string }) =>
    step.uses?.startsWith("actions/checkout@"),
  );
  assert.equal(checkout?.with?.ref, "${{ github.event.repository.default_branch }}");
  assert.match(checkout?.with?.["sparse-checkout"], /issue-pr-title-policy\.rs/);
  assert.ok(
    steps.some(
      (step: { run?: string }) =>
        step.run === "rust-script tools/commands/ci/github/issue-pr-title-policy.rs",
    ),
  );
});

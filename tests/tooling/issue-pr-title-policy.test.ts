import assert from "node:assert/strict";
import { test } from "node:test";

import { assignCall, patchTitleCall, runPolicy } from "./support/title-policy.ts";

test("issue title policy normalizes the scope and assigns new issues", () => {
  const { result, ghCalls } = runPolicy(
    {
      action: "opened",
      issue: {
        number: 12,
        title: "fix(check): checklist formatting is inconsistent",
        assignees: [],
      },
    },
    "issues",
  );

  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(ghCalls, [
    patchTitleCall(12, "fix(canon): checklist formatting is inconsistent"),
    assignCall(12),
  ]);
});

test("issue title policy assigns new issues without rewriting an already-correct title", () => {
  const { result, ghCalls } = runPolicy(
    {
      action: "opened",
      issue: { number: 23, title: "docs: check the formatting guide", assignees: [] },
    },
    "issues",
  );

  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(ghCalls, [assignCall(23)]);
});

test("PR title policy normalizes conventional titles before validation", () => {
  const { result, ghCalls } = runPolicy(
    {
      action: "opened",
      pull_request: {
        number: 34,
        title: "fix(check): update lint rules",
        assignees: [{ login: "ubugeeei" }],
      },
    },
    "pull_request_target",
  );

  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(ghCalls, [patchTitleCall(34, "fix(canon): update lint rules")]);
});

test("PR title policy fails non-conventional titles after normalization", () => {
  const { result, ghCalls } = runPolicy(
    {
      action: "opened",
      pull_request: { number: 56, title: "fix lint issue", assignees: [] },
    },
    "pull_request_target",
  );

  assert.equal(result.status, 1);
  assert.equal(
    result.stdout,
    "Assigned pull_request #56 to ubugeeei\n" +
      "::error title=Invalid PR title::Use Conventional Commits format: type(scope): summary\n",
  );
  assert.deepEqual(ghCalls, [assignCall(56)]);
});

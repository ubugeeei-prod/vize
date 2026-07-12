import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type WorkflowStep = {
  env?: Record<string, string>;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

type WorkflowJob = {
  permissions?: Record<string, string>;
  steps?: WorkflowStep[];
  "timeout-minutes"?: number;
};

test("reusable release preflight verifies evidence and crate plans without registry credentials", () => {
  const source = readRepoFile(".github", "workflows", "release-preflight.yml");
  const workflow = parse(source) as {
    env?: Record<string, unknown>;
    jobs?: Record<string, WorkflowJob>;
    on?: Record<string, unknown>;
    permissions?: Record<string, string>;
  };

  assert.ok(Object.hasOwn(workflow.on ?? {}, "workflow_call"));
  assert.deepEqual(workflow.permissions, {
    actions: "write",
    contents: "read",
    issues: "read",
  });
  assert.equal(workflow.env?.FORCE_JAVASCRIPT_ACTIONS_TO_NODE24, true);
  assert.deepEqual(Object.keys(workflow.jobs ?? {}).sort(), ["validate-crates", "verify"]);

  const verify = workflow.jobs?.verify;
  assert.ok(verify);
  assert.equal(verify["timeout-minutes"], 120);
  assert.deepEqual(verify.permissions, {
    actions: "write",
    contents: "read",
    issues: "read",
  });
  const verifyCheckout = verify.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.ok(verifyCheckout);
  assert.equal(verifyCheckout.with?.["fetch-depth"], 0);
  assert.equal(verifyCheckout.with?.["persist-credentials"], false);
  const verification = verify.steps?.find((step) => step.run != null);
  assert.equal(verification?.run, "node tools/github/release-preflight.mjs");
  assert.equal(verification?.env?.GITHUB_TOKEN, "${{ github.token }}");

  const crateValidation = workflow.jobs?.["validate-crates"];
  assert.ok(crateValidation);
  assert.equal(crateValidation["timeout-minutes"], 45);
  assert.deepEqual(crateValidation.permissions, { contents: "read" });
  assert.ok(crateValidation.steps?.some((step) => step.uses?.startsWith("wild-linker/action@")));
  assert.ok(
    crateValidation.steps?.some(
      (step) => step.run === "moon run --target native tools/moon/cmd/publish_crates -- --dry-run",
    ),
  );

  assert.doesNotMatch(source, /environment:|id-token:\s*write|secrets\.|crates-io-auth-action/);
});

import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { hostedOrBlacksmith, readRepoFile } from "./support/github-workflows.ts";

type WorkflowStep = {
  env?: Record<string, string>;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

type WorkflowJob = {
  permissions?: Record<string, string>;
  "runs-on"?: string;
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
  assert.match(verify["runs-on"] ?? "", new RegExp(`^${hostedOrBlacksmith("ubuntu-24.04")}$`));
  assert.equal(verify["timeout-minutes"], 360);
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
  assert.equal(verification?.run, "rust-script tools/commands/ci/github/release-preflight.rs");
  assert.equal(verification?.env?.GITHUB_TOKEN, "${{ github.token }}");

  const crateValidation = workflow.jobs?.["validate-crates"];
  assert.ok(crateValidation);
  assert.match(
    crateValidation["runs-on"] ?? "",
    new RegExp(`^${hostedOrBlacksmith("ubuntu-24.04")}$`),
  );
  assert.equal(crateValidation["timeout-minutes"], 45);
  assert.deepEqual(crateValidation.permissions, { contents: "read" });
  const crateCheckout = crateValidation.steps?.find((step) =>
    step.uses?.startsWith("actions/checkout@"),
  );
  assert.ok(crateCheckout);
  assert.equal(crateCheckout.with?.["persist-credentials"], false);
  const wildLinker = crateValidation.steps?.find((step) =>
    step.uses?.startsWith("wild-linker/action@"),
  );
  assert.ok(wildLinker);
  assert.equal(wildLinker.with?.["wild-version"], "0.9.0");
  assert.ok(
    crateValidation.steps?.some(
      (step) => step.run === "moon run --target native tools/moon/cmd/publish_crates -- --dry-run",
    ),
  );
  const stickyCache = crateValidation.steps?.find(
    (step) => step.uses === "./.github/actions/setup-rust-sticky-cache",
  );
  assert.ok(stickyCache);
  assert.equal(stickyCache.with?.key, "release-crates-dry-run");
  assert.equal(stickyCache.with?.["cache-key-suffix"], "${{ runner.os }}-${{ runner.arch }}");

  assert.doesNotMatch(source, /environment:|id-token:\s*write|secrets\.|crates-io-auth-action/);
});

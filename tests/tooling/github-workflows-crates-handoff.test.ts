import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type WorkflowStep = {
  env?: Record<string, string>;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

type WorkflowJob = {
  environment?: string;
  if?: string;
  permissions?: Record<string, string>;
  "runs-on"?: string;
  steps?: WorkflowStep[];
  "timeout-minutes"?: number;
};

test("crates.io handoff workflow recovers the JSX and Patina release set", () => {
  const source = readRepoFile(".github", "workflows", "release.yml");
  const workflow = parse(source) as {
    concurrency?: { group?: string };
    jobs?: Record<string, WorkflowJob>;
    on?: Record<string, unknown>;
    permissions?: Record<string, string>;
  };

  const dispatch = workflow.on?.workflow_dispatch as
    | { inputs?: Record<string, { options?: string[]; type?: string }> }
    | undefined;
  assert.ok(dispatch);
  assert.equal(dispatch.inputs?.tag_name?.type, "string");
  assert.deepEqual(dispatch.inputs?.crate_set?.options, ["jsx-patina"]);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.match(workflow.concurrency?.group ?? "", /release-crates-handoff/);

  for (const jobName of [
    "plan-release-platforms",
    "build-editor-extensions",
    "build-release-packages",
    "build-wasm-package",
    "release-preflight",
  ]) {
    assert.equal(workflow.jobs?.[jobName]?.if, "github.event_name == 'push'", jobName);
  }

  const job = workflow.jobs?.["release-crates-handoff"];
  assert.ok(job);
  assert.equal(job.if, "github.event_name == 'workflow_dispatch'");
  assert.equal(job.environment, "crates-io");
  assert.deepEqual(job.permissions, { contents: "read", "id-token": "write" });
  assert.match(job["runs-on"] ?? "", /^blacksmith-32vcpu-ubuntu-2404$/);
  assert.equal(job["timeout-minutes"], 30);

  const checkout = job.steps?.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.ok(checkout);
  assert.equal(checkout.with?.ref, "${{ steps.release.outputs.tag_name }}");
  assert.equal(checkout.with?.["persist-credentials"], false);
  assert.ok(job.steps?.some((step) => step.uses?.startsWith("rust-lang/crates-io-auth-action@")));
  assert.ok(
    job.steps?.some((step) =>
      /--crate vize_atelier_jsx[\s\S]*--crate vize_patina/.test(step.run ?? ""),
    ),
  );
  assert.doesNotMatch(source, /CARGO_REGISTRY_TOKEN:\s*\$\{\{\s*secrets\./);
});

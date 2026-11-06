import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

interface WorkflowJob {
  steps?: Array<{ run?: string; uses?: string; with?: Record<string, unknown> }>;
  strategy?: { matrix?: Record<string, unknown> };
}

test("check workflow runs declared Node engine compatibility matrix", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const parsed = parse(workflow) as { jobs?: Record<string, WorkflowJob> };
  const parsedJob = parsed.jobs?.["node-engine-compat"];
  const job = workflowJobBody(workflow, "node-engine-compat");
  const setupStep = parsedJob?.steps?.find((step) =>
    step.uses?.startsWith("voidzero-dev/setup-vp@"),
  );
  const testStep = parsedJob?.steps?.find(
    (step) => step.uses === "./.github/actions/test-small-node-cases",
  );

  assert.deepEqual(parsedJob?.strategy?.matrix?.["node-version"], ["22", "24"]);
  assert.doesNotMatch(job, /\.node-version\.ci/);
  assert.deepEqual(setupStep?.with, {
    version: "latest",
    "node-version": "${{ matrix.node-version }}",
    cache: true,
    "cache-save": false,
    "run-install": false,
  });
  assert.equal(testStep?.uses, "./.github/actions/test-small-node-cases");
  assert.deepEqual(testStep?.with, {
    suite: "node-engine",
    job: "node-engine-compat (${{ matrix.node-version }})",
    packet: "node-engine-${{ matrix.node-version }}",
  });
  const action = readRepoFile(".github", "actions", "test-small-node-cases", "action.yml");
  for (const file of [
    "tests/tooling/node-engine-matrix.test.ts",
    "tests/tooling/package-manifests.test.ts",
    "tests/tooling/bundler-plugin-manifests.test.ts",
  ])
    assert.ok(action.includes(file));
  assert.doesNotMatch(job, /vscode-typescript-vue-plugin\.test\.ts/);
});

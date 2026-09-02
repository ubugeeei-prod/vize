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
  const testStep = parsedJob?.steps?.find((step) =>
    step.run?.includes("tests/tooling/node-engine-matrix.test.ts"),
  );

  assert.deepEqual(parsedJob?.strategy?.matrix?.["node-version"], ["22", "24"]);
  assert.doesNotMatch(job, /\.node-version\.ci/);
  assert.deepEqual(setupStep?.with, {
    "node-version": "${{ matrix.node-version }}",
    cache: true,
    "run-install": false,
  });
  assert.match(
    testStep?.run ?? "",
    /node --test tests\/tooling\/node-engine-matrix\.test\.ts tests\/tooling\/package-manifests\.test\.ts/,
  );
  assert.doesNotMatch(job, /vscode-typescript-vue-plugin\.test\.ts/);
});

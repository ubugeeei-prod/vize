import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

interface ActionStep {
  if?: string;
  name?: string;
  run?: string;
  shell?: string;
  uses?: string;
  with?: Record<string, unknown>;
}

function actionStep(steps: ActionStep[], name: string): ActionStep {
  const step = steps.find((candidate) => candidate.name === name);
  assert.ok(step, `missing ${name}`);
  return step;
}

test("setup-runtime-package-managers accepts package.json and matrix Node sources", () => {
  const parsed = parse(
    readRepoFile(".github", "actions", "setup-runtime-package-managers", "action.yml"),
  ) as {
    inputs?: Record<string, { default?: string }>;
    runs?: { steps?: ActionStep[] };
  };
  const steps = parsed.runs?.steps ?? [];
  const corepack = actionStep(steps, "Enable package manager shims");
  const bun = actionStep(steps, "Install Bun package manager");

  assert.equal(parsed.inputs?.["node-version-file"]?.default, "package.json");
  assert.deepEqual(actionStep(steps, "Setup Vite+ and Node.js from version file"), {
    name: "Setup Vite+ and Node.js from version file",
    if: "inputs.node-version == ''",
    uses: "voidzero-dev/setup-vp@ca1c46663915d6c1042ae23bd39ab85718bfb0fa",
    with: {
      "node-version-file": "${{ inputs.node-version-file }}",
      cache: "${{ inputs.cache }}",
      "run-install": false,
    },
  });
  assert.deepEqual(actionStep(steps, "Setup Vite+ and Node.js from matrix version"), {
    name: "Setup Vite+ and Node.js from matrix version",
    if: "inputs.node-version != ''",
    uses: "voidzero-dev/setup-vp@ca1c46663915d6c1042ae23bd39ab85718bfb0fa",
    with: {
      "node-version": "${{ inputs.node-version }}",
      cache: "${{ inputs.cache }}",
      "run-install": false,
    },
  });
  assert.deepEqual(corepack, {
    name: "Enable package manager shims",
    shell: "bash",
    run: "corepack enable",
  });
  assert.deepEqual(bun, {
    name: "Install Bun package manager",
    uses: "oven-sh/setup-bun@0c5077e51419868618aeaa5fe8019c62421857d6",
    with: { "bun-version": "1.3.14" },
  });
  assert.ok(steps.indexOf(corepack) < steps.indexOf(bun));
});

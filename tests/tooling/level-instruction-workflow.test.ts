import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.mjs";
import { readRepoFile } from "./support/github-workflows.ts";

type Job = {
  env?: Record<string, string>;
  if?: string;
  needs?: string[];
  steps?: Array<{
    name?: string;
    if?: string;
    run?: string;
    uses?: string;
    with?: Record<string, unknown>;
    "continue-on-error"?: boolean | string;
  }>;
  uses?: string;
};
type Workflow = { name: string; on?: Record<string, unknown>; jobs?: Record<string, Job> };
const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as Workflow;
const instructionWorkflow = parse(
  readRepoFile(".github", "workflows", "level-instruction-counts.yml"),
) as Workflow;

test("merge queue instruction ceilings are enforced through the required aggregate", () => {
  const caller = workflow.jobs?.["instruction-counts"];
  assert.equal(caller?.uses, "./.github/workflows/level-instruction-counts.yml");
  assert.equal(
    caller?.if,
    "${{ github.event_name == 'pull_request' || github.event_name == 'merge_group' }}",
  );
  assert.ok(workflow.jobs?.["test-report"]?.needs?.includes("instruction-counts"));
  assert.ok(Object.hasOwn(instructionWorkflow.on ?? {}, "workflow_call"));
  const gate = instructionWorkflow.jobs?.["instruction-counts"];
  assert.equal(gate?.if, undefined);
  assert.equal(
    gate?.env?.MEASURE,
    "${{ github.event_name == 'merge_group' || github.event_name == 'workflow_dispatch' || github.workflow == 'Level instruction counts' }}",
  );
  assert.equal(instructionWorkflow.name, "Level instruction counts");
  const verify = gate?.steps?.find(
    (step) => step.name === "Verify pinned registry and immutable base ratchet",
  );
  assert.equal(verify?.if, undefined, "a missing or invalid registry must fail every call");
  assert.match(verify?.run ?? "", /--verify-budgets/);
  assert.match(verify?.run ?? "", /sha256sum --check --strict/);
  for (const name of [
    "Measure each stage three times without Criterion sampling",
    "Enforce pinned ceilings",
  ]) {
    const step = gate?.steps?.find((candidate) => candidate.name === name);
    assert.ok(step);
    assert.equal(step.if, "env.MEASURE == 'true'");
    assert.doesNotMatch(step.run ?? "", /hashFiles|continue-on-error/);
  }
});

test("required aggregate rejects failed, cancelled and skipped instruction checks", () => {
  for (const result of ["failure", "cancelled", "skipped"]) {
    const decision = aggregateNeedsResults({ "instruction-counts": { result } });
    assert.equal(decision.exitCode, 1);
    assert.match(decision.message, new RegExp(`instruction-counts: ${result}`));
  }
});

test("required report keeps inventory and final dependency verification in the same job", () => {
  const report = workflow.jobs?.["test-report"];
  assert.equal(report?.steps?.[1]?.uses, "./.github/actions/report-test-inventory");
  assert.equal(
    report?.steps?.at(-1)?.run,
    "node tools/support/compat/github/require-needs-success.mjs",
  );
  const action = parse(
    readRepoFile(".github", "actions", "report-test-inventory", "action.yml"),
  ) as { runs: { using: string; steps: NonNullable<Job["steps"]> } };
  assert.equal(action.runs.using, "composite");
  const names = [
    "Setup Vite+ and Node.js",
    "Install report dependencies",
    "Collect test inventory",
    "Upload test inventory",
  ];
  const positions = names.map((name) => action.runs.steps.findIndex((step) => step.name === name));
  for (const [index, position] of positions.entries()) {
    assert.notEqual(position, -1, names[index]);
    if (index > 0) {
      assert.ok(position > positions[index - 1], `${names[index]} must follow ${names[index - 1]}`);
    }
    const step = action.runs.steps[position];
    assert.equal(step.if, undefined, `${names[index]} must run unconditionally`);
    assert.ok(
      step["continue-on-error"] === undefined || step["continue-on-error"] === false,
      `${names[index]} must propagate failures`,
    );
  }
  const [setup, install, collect, upload] = positions.map(
    (position) => action.runs.steps[position],
  );
  assert.equal(setup.uses, "voidzero-dev/setup-vp@ca1c46663915d6c1042ae23bd39ab85718bfb0fa");
  assert.deepEqual(setup.with, {
    "node-version-file": "package.json",
    cache: true,
    "run-install": false,
  });
  assert.equal(
    install.run,
    "vp install --frozen-lockfile --prefer-offline --filter vize-workspace --ignore-scripts",
  );
  assert.match(collect.run ?? "", /test-inventory\.mjs --json test-inventory\.json/);
  assert.match(upload.uses ?? "", /^actions\/upload-artifact@/);
});

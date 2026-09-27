import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.mjs";
import { readRepoFile } from "./support/github-workflows.ts";

type Job = {
  env?: Record<string, string>;
  if?: string;
  needs?: string[];
  steps?: Array<{ name?: string; if?: string; run?: string; uses?: string }>;
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
  assert.match(action.runs.steps[0]?.run ?? "", /test-inventory\.mjs --json test-inventory\.json/);
  assert.match(action.runs.steps[1]?.uses ?? "", /^actions\/upload-artifact@/);
});

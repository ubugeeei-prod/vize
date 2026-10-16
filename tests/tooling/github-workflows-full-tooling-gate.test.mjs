import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import {
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import { toolingTestCommand } from "../../tools/support/compat/github/run-tooling-tests.mjs";
import {
  MAX_TOOLING_SHARDS,
  toolingShardMatrix,
} from "../../tools/support/compat/github/tooling-test-shards.ts";
import { aggregateNeedsResults } from "../../tools/support/compat/github/require-needs-success.ts";
import {
  assertRequiredWorkflowJobs,
  requiredWorkflowJobNames,
} from "../../tools/support/compat/github/release-preflight-evidence.mjs";

const root = new URL("../../", import.meta.url);
const read = (file) => readFileSync(new URL(file, root), "utf8");
const yaml = (file) => parse(read(file));
const fullCondition =
  "${{ github.event_name == 'schedule' || github.event_name == 'workflow_dispatch' }}";
const alwaysFullCondition = fullCondition
  .replace("${{ ", "${{ always() && (")
  .replace(" }}", ") }}");
const planCommand =
  'node tools/support/compat/github/plan-tooling-tests.mjs 0000000000000000000000000000000000000000 "$GITHUB_SHA" --tier merge --output "$RUNNER_TEMP/tooling-plan.json"';

void test("one planner owns the exhaustive matrix that every full worker regenerates", () => {
  const jobs = yaml(".github/workflows/check.yml").jobs;
  assert.equal(jobs["tooling-plan"].if, fullCondition);
  assert.equal(jobs["tooling-plan"].uses, "./.github/workflows/full-tooling-plan.yml");
  const workflow = yaml(".github/workflows/full-tooling-plan.yml");
  assert.deepEqual(workflow.on.workflow_call.outputs, {
    matrix: { value: "${{ jobs.plan.outputs.matrix }}" },
  });
  const plan = workflow.jobs.plan;
  assert.equal(plan["timeout-minutes"], 5);
  assert.deepEqual(plan.outputs, { matrix: "${{ steps.plan.outputs.tooling-matrix }}" });
  assert.equal(plan.steps.at(-1).id, "plan");
  assert.equal(plan.steps.at(-1).run, `${planCommand} --github-output`);
  const worker = yaml(".github/actions/test-full-tooling/action.yml").runs.steps.find((step) =>
    step.run?.includes("plan-tooling-tests.mjs"),
  );
  assert.equal(worker.run, planCommand);
});

void test("the planned matrix partitions the complete inventory and refuses any other shape", () => {
  const files = toolingTestFiles();
  const full = planToolingTests([], { tier: "merge" });
  const { include } = toolingShardMatrix(full);
  const total = include.length;
  assert.ok(total > 1 && total <= MAX_TOOLING_SHARDS);
  assert.deepEqual(
    include.map(({ index, total }) => ({ index, total })),
    Array.from({ length: total }, (_, index) => ({ index: index + 1, total })),
  );
  const union = include.flatMap(({ index }) =>
    toolingTestCommand(full, files, `${index}/${total}`).slice(2),
  );
  assert.deepEqual([...union].sort(), [...files].sort());
  for (const shard of [`1/${total + 1}`, `${total + 1}/${total}`, `0/${total}`, String(total)])
    assert.throws(() => toolingTestCommand(full, files, shard));
});

void test("release evidence keeps exactly one test-scripts job that requires every planned worker", () => {
  const jobs = yaml(".github/workflows/check.yml").jobs;
  const completed = (name) => ({ name, status: "completed", conclusion: "success" });
  const workerNames = toolingShardMatrix(planToolingTests([], { tier: "merge" })).include.map(
    ({ name }) => `Tooling (${name})`,
  );
  const evidence = [...requiredWorkflowJobNames("Check"), ...workerNames].map(completed);
  assert.doesNotThrow(() => assertRequiredWorkflowJobs("Check", evidence));
  assert.throws(
    () =>
      assertRequiredWorkflowJobs(
        "Check",
        evidence.filter(({ name }) => name !== "test-scripts"),
      ),
    /exactly one successful test-scripts job; found 0/,
  );
  assert.throws(
    () => assertRequiredWorkflowJobs("Check", [...evidence, completed("test-scripts")]),
    /exactly one successful test-scripts job; found 2/,
  );
  const named = Object.entries(jobs).filter(
    ([id, job]) => (job.name ?? id) === "test-scripts" && job.strategy === undefined,
  );
  assert.deepEqual(
    named.map(([id]) => id),
    ["test-scripts-gate"],
  );
  assert.notEqual(
    jobs["test-scripts"].strategy,
    undefined,
    "matrix workers report per-shard names",
  );
  assert.equal(jobs["test-scripts"].name, "Tooling (${{ matrix.name }})");
  const gate = jobs["test-scripts-gate"];
  assert.equal(gate.if, alwaysFullCondition);
  assert.equal(gate["timeout-minutes"], 5);
  assert.deepEqual(gate.needs, ["full-source-cli", "tooling-plan", "test-scripts"]);
  assert.equal(gate.steps.at(-1).uses, "./.github/actions/report-full-js-qualification");
  assert.deepEqual(gate.steps.at(-1).with, { "needs-json": "${{ toJSON(needs) }}" });
  assert.equal(gate.steps.at(-1)["continue-on-error"], undefined);
  const green = Object.fromEntries(gate.needs.map((name) => [name, { result: "success" }]));
  assert.equal(aggregateNeedsResults(green).exitCode, 0);
  for (const name of gate.needs)
    for (const result of ["failure", "cancelled", "skipped"])
      assert.equal(aggregateNeedsResults({ ...green, [name]: { result } }).exitCode, 1);
});

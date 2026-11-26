import { readAutomationPrograms } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

const workflow = parse(
  readAutomationPrograms(
    new URL("../../.github/workflows/04-typechecker-native-phases.yml", import.meta.url),
  ),
);
const job = workflow.jobs["native-phases"];
const historicalProjectionCondition = job.steps.find(
  (step: { name?: string }) =>
    step.name ===
    "Qualify declared template emits and config-scoped Vue helpers in the source CLI and editor",
)?.if;

const events = [
  ["scheduled", "schedule", {}],
  ["manual with receipt", "workflow_dispatch", { historical_projection_receipt: true }],
  ["manual without receipt", "workflow_dispatch", { historical_projection_receipt: false }],
  ["manual default", "workflow_dispatch", {}],
  ["pull request", "pull_request", { historical_projection_receipt: true }],
  ["merge group", "merge_group", { historical_projection_receipt: true }],
  ["push", "push", { historical_projection_receipt: true }],
] as const;

function enabledEvents(condition: unknown): string[] {
  assert.ok(typeof condition === "string", "the workflow step declares an event condition");
  return events
    .filter(([, event_name, inputs]) =>
      runInNewContext(condition, { github: { event_name }, inputs }),
    )
    .map(([name]) => name);
}

function evaluate(value: string, github: object, inputs: object = {}) {
  const expression = /^\$\{\{\s*(.*?)\s*\}\}$/.exec(value)?.[1];
  assert.ok(expression, `expected an Actions expression: ${value}`);
  return runInNewContext(expression, { github, inputs });
}

void test("historical native phase campaigns run only on schedule or explicit immutable source dispatch", () => {
  assert.deepEqual(Object.keys(workflow.on).sort(), ["schedule", "workflow_dispatch"]);
  assert.equal(workflow.on.schedule.length, 1);
  assert.equal(workflow.on.workflow_dispatch.inputs.source_sha.required, true);
  assert.deepEqual(workflow.on.workflow_dispatch.inputs.historical_projection_receipt, {
    description:
      "Run the schedule-only template projection history qualification at the exact driver",
    required: false,
    default: false,
    type: "boolean",
  });
  assert.equal(job.if, undefined);
  assert.deepEqual(enabledEvents(historicalProjectionCondition), [
    "scheduled",
    "manual with receipt",
  ]);
  for (const name of [
    "Qualify original outside-import types with the source CLI and official native oracle",
    "Verify the original reference-path project through native, batch and actual CLI checks",
    "Qualify optional callable defaults in the source CLI and editor",
  ])
    assert.deepEqual(
      enabledEvents(job.steps.find((step: { name?: string }) => step.name === name)?.if),
      ["scheduled"],
      name,
    );
});

void test("scheduled source identity follows the immutable event and manual source remains explicit", () => {
  const main = "1".repeat(40);
  const historical = "2".repeat(40);
  const github = { event_name: "schedule", sha: main, workflow_sha: main };
  assert.equal(evaluate(workflow.env.SOURCE_SHA, github), main);
  assert.equal(evaluate(workflow.env.DRIVER_SHA, github), main);
  assert.equal(
    evaluate(
      workflow.env.SOURCE_SHA,
      { ...github, event_name: "workflow_dispatch" },
      { source_sha: historical },
    ),
    historical,
  );
  const custody = job.steps.find(
    (step: { name?: string }) =>
      step.name === "Pin immutable driver, source and freshly fetched main baseline",
  ).run;
  assert.match(custody, /git merge-base --is-ancestor "\$DRIVER_SHA" "\$main_head_sha"/);
  assert.match(custody, /git merge-base --is-ancestor "\$SOURCE_SHA" "\$main_head_sha"/);
  assert.match(custody, /"\$SOURCE_SHA" == "\$DRIVER_SHA" && "\$SOURCE_SHA" == "\$GITHUB_SHA"/);
  assert.match(custody, /"\$HISTORICAL_PROJECTION_RECEIPT" == true/);
  const source = job.steps.find(
    (step: { with?: { path?: string } }) => step.with?.path === ".native-phase-source",
  );
  assert.deepEqual(enabledEvents(source.if), [
    "manual with receipt",
    "manual without receipt",
    "manual default",
  ]);
  assert.equal(source.with.ref, "${{ env.SOURCE_SHA }}");
  assert.equal(source.with["persist-credentials"], false);
});

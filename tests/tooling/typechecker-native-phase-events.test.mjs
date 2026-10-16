import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

const workflow = parse(
  readFileSync(
    new URL("../../.github/workflows/typechecker-native-phases.yml", import.meta.url),
    "utf8",
  ),
);
const job = workflow.jobs["native-phases"];

function evaluate(value, github, inputs = {}) {
  const expression = /^\$\{\{\s*(.*?)\s*\}\}$/.exec(value)?.[1];
  assert.ok(expression, `expected an Actions expression: ${value}`);
  return runInNewContext(expression, { github, inputs });
}

void test("historical native phase campaigns run only on schedule or explicit immutable source dispatch", () => {
  assert.deepEqual(Object.keys(workflow.on).sort(), ["schedule", "workflow_dispatch"]);
  assert.equal(workflow.on.schedule.length, 1);
  assert.equal(workflow.on.workflow_dispatch.inputs.source_sha.required, true);
  assert.equal(job.if, undefined);
  for (const name of [
    "Qualify declared template emits and config-scoped Vue helpers in the source CLI and editor",
    "Qualify original outside-import types with the source CLI and official native oracle",
    "Verify the original reference-path project through native, batch and actual CLI checks",
    "Qualify optional callable defaults in the source CLI and editor",
  ])
    assert.equal(
      job.steps.find((step) => step.name === name)?.if,
      "github.event_name == 'schedule'",
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
    (step) => step.name === "Pin immutable driver, source and freshly fetched main baseline",
  ).run;
  assert.match(custody, /git merge-base --is-ancestor "\$DRIVER_SHA" "\$main_head_sha"/);
  assert.match(custody, /git merge-base --is-ancestor "\$SOURCE_SHA" "\$main_head_sha"/);
  assert.match(custody, /"\$SOURCE_SHA" == "\$DRIVER_SHA" && "\$SOURCE_SHA" == "\$GITHUB_SHA"/);
  const source = job.steps.find((step) => step.with?.path === ".native-phase-source");
  assert.equal(source.if, "github.event_name == 'workflow_dispatch'");
  assert.equal(source.with.ref, "${{ env.SOURCE_SHA }}");
  assert.equal(source.with["persist-credentials"], false);
});

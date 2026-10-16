import assert from "node:assert/strict";
import test from "node:test";
import { parse } from "yaml";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("JS check runtime reserves native prerequisites for full qualification", () => {
  const checkJsJob = workflowJobBody(readRepoFile(".github", "workflows", "check.yml"), "check-js");
  const action = readRepoFile(".github", "actions", "setup-js-check-runtime", "action.yml");
  const runtime = parse(action);
  const actionSteps =
    (runtime as { runs?: { steps?: Array<{ uses?: string; if?: string }> } }).runs?.steps ?? [];
  const actionUses = actionSteps.map((step) => step.uses).filter((uses) => uses != null);

  assert.match(
    checkJsJob,
    /vp run --no-cache --filter '\.\/npm\/native' build:debug && vp run --workspace-root check:ci/,
  );
  assert.match(action, /setup-moonbit/);
  assert.match(action, /dtolnay\/rust-toolchain/);
  assert.match(action, /wild-linker\/action/);
  assert.ok(actionUses.includes("./.github/actions/setup-rust-script"));
  assert.match(action, /key:\s*check-js/);
  assert.equal(runtime.inputs.full.default, "false");
  for (const step of actionSteps.slice(1)) assert.equal(step.if, "inputs.full == 'true'");
});

test("workflow context validation belongs to the unconditional required JS gate", () => {
  type Step = { uses?: string; if?: unknown; "continue-on-error"?: unknown };
  const workflow = parse(readRepoFile(".github", "workflows", "check.yml"));
  const job = workflow.jobs["check-js"] as { if?: unknown; steps: Step[] };
  assert.equal(job.if, undefined, "missing optional workflow jobs must not bypass this gate");
  const runtimes = job.steps.filter(
    (step) => step.uses === "./.github/actions/setup-js-check-runtime",
  );
  assert.equal(runtimes.length, 1);
  assert.equal(runtimes[0].if, undefined);
  assert.equal(runtimes[0]["continue-on-error"], undefined);
  const runtime = parse(readRepoFile(".github", "actions", "setup-js-check-runtime", "action.yml"));
  const steps = runtime.runs.steps as Step[];
  assert.equal(steps[0].uses, "./.github/actions/check-workflow-syntax");
  assert.equal(steps[0].if, undefined);
  assert.equal(steps[0]["continue-on-error"], undefined);
  assert.equal(steps.filter((step) => step.uses === steps[0].uses).length, 1);
});

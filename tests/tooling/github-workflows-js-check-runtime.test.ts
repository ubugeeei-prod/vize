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
  assert.doesNotMatch(action, /setup-moonbit/);
  assert.match(action, /dtolnay\/rust-toolchain/);
  assert.ok(actionUses.includes("./.github/actions/setup-wild-http-recovery"));
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

test("workflow checker release recovery stays bounded and version locked", () => {
  const action = parse(readRepoFile(".github", "actions", "check-workflow-syntax", "action.yml"));
  const install = action.runs.steps.find(
    (step: { name?: string }) => step.name === "Install the pinned workflow checker",
  ).run as string;
  const normalized = install.replace(/\\\s+/gu, " ").replace(/\s+/gu, " ");
  assert.ok(
    normalized.includes(
      "--connect-timeout 10 --max-time 30 --speed-limit 1 --speed-time 5 --retry 2 --retry-all-errors --retry-delay 1 --retry-max-time 30",
    ),
  );
  assert.ok(
    normalized.includes(
      "7:000|18:200|28:000|28:200|28:302|52:000|55:000|55:200|56:000|56:200| 22:500|22:502|22:503|22:504",
    ),
  );
  assert.ok(
    normalized.includes(
      "--connect-timeout 10 --max-time 30 --retry 5 --retry-all-errors --retry-delay 1 --retry-max-time 30",
    ),
  );
  assert.match(
    install,
    /GOBIN="\$directory" go install github\.com\/rhysd\/actionlint\/cmd\/actionlint@v1\.7\.12/u,
  );
  assert.match(install, /8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8/u);
});

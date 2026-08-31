import assert from "node:assert/strict";
import test from "node:test";
import { parse } from "yaml";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("JS check runtime action installs the native build prerequisites", () => {
  const checkJsJob = workflowJobBody(readRepoFile(".github", "workflows", "check.yml"), "check-js");
  const action = readRepoFile(".github", "actions", "setup-js-check-runtime", "action.yml");
  const actionSteps =
    (parse(action) as { runs?: { steps?: Array<{ uses?: string }> } }).runs?.steps ?? [];
  const actionUses = actionSteps.map((step) => step.uses).filter((uses) => uses != null);

  assert.match(
    checkJsJob,
    /vp run --filter '\.\/npm\/native' build:debug && vp run --workspace-root check:ci/,
  );
  assert.match(action, /setup-moonbit/);
  assert.match(action, /dtolnay\/rust-toolchain/);
  assert.match(action, /wild-linker\/action/);
  assert.ok(actionUses.includes("./.github/actions/setup-rust-script"));
  assert.match(action, /key:\s*check-js/);
});

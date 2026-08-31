import assert from "node:assert/strict";
import test from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("JS check runtime action installs the native build prerequisites", () => {
  const checkJsJob = workflowJobBody(readRepoFile(".github", "workflows", "check.yml"), "check-js");
  const action = readRepoFile(".github", "actions", "setup-js-check-runtime", "action.yml");

  assert.match(
    checkJsJob,
    /vp run --filter '\.\/npm\/native' build:debug && vp run --workspace-root check:ci/,
  );
  assert.match(action, /setup-moonbit/);
  assert.match(action, /dtolnay\/rust-toolchain/);
  assert.match(action, /wild-linker\/action/);
  assert.match(action, /key:\s*check-js/);
});

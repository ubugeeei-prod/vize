import assert from "node:assert/strict";
export function mainManualContext(env: NodeJS.ProcessEnv = process.env): void {
  assert.equal(env.GITHUB_ACTIONS, "true");
  assert.equal(env.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(env.GITHUB_REF, "refs/heads/main");
  assert.equal(env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
}

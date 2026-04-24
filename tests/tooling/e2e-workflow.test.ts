import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("app e2e workflow is manually selectable and uploads failure artifacts", () => {
  const workflow = readRepoFile(".github", "workflows", "e2e.yml");

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /type:\s*choice/);
  for (const suite of ["dev", "preview", "check", "lint", "build", "check-fixtures"]) {
    assert.match(workflow, new RegExp(`- ${suite}`));
    assert.match(workflow, new RegExp(`${suite}\\)\\n\\s+`));
  }

  assert.match(workflow, /Build native package/);
  assert.match(workflow, /Cache Playwright browsers/);
  assert.match(workflow, /pnpm --dir tests exec playwright install --with-deps chromium/);
  assert.match(workflow, /RUN_BUILD_TESTS=1 pnpm --dir tests run test:preview/);
  assert.match(workflow, /- name: Upload app e2e artifacts\s+if: failure\(\)/);
  assert.match(workflow, /tests\/app\/results\//);
  assert.match(workflow, /tests\/app\/screenshots\//);
  assert.match(workflow, /tests\/app\/playwright-report\//);
  assert.match(workflow, /tests\/playwright-report\//);
  assert.match(workflow, /if-no-files-found:\s*ignore/);
});

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

  assert.match(workflow, /pull_request:\n\s+branches: \[main\]/);
  assert.match(workflow, /name: app-e2e \(\$\{\{ matrix\.suite \}\}\)/);
  assert.match(workflow, /fail-fast:\s*false/);
  assert.match(
    workflow,
    /fromJSON\('\["dev","preview","build","check","lint","check-fixtures"\]'\)/,
  );
  assert.doesNotMatch(workflow, /fromJSON\('\["dev","vrt","preview"/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /type:\s*choice/);
  for (const suite of ["dev", "vrt", "preview", "check", "lint", "build", "check-fixtures"]) {
    assert.match(workflow, new RegExp(`- ${suite}`));
    assert.match(workflow, new RegExp(`${suite}\\)\\n\\s+`));
  }

  assert.match(workflow, /--filter '\.\/tests\.\.\.'/);
  assert.match(workflow, /--filter '\.\/npm\/vize-native\.\.\.'/);
  assert.match(workflow, /--filter '\.\/npm\/vite-plugin-vize\.\.\.'/);
  assert.match(workflow, /Build native package/);
  assert.match(workflow, /Build vize CLI/);
  assert.match(workflow, /cargo build --profile ci -p vize/);
  assert.match(workflow, /uses: \.\/\.github\/actions\/setup-moonbit/);
  assert.match(workflow, /Cache Playwright browsers/);
  assert.match(workflow, /contains\(fromJSON\('\["dev","vrt"\]'\), matrix\.suite\)/);
  assert.match(workflow, /vp exec --filter '\.\/tests' -- playwright install --with-deps chromium/);
  assert.match(workflow, /RUN_BUILD_TESTS=1 vp run --filter '\.\/tests' test:preview/);
  assert.match(workflow, /vp run --filter '\.\/tests' test:dev:ci/);
  assert.match(workflow, /vp run --filter '\.\/tests' test:check/);
  assert.match(workflow, /vp run --filter '\.\/tests' test:lint/);
  assert.match(workflow, /vp run --filter '\.\/tests' test:check:fixtures/);
  assert.doesNotMatch(workflow, /pnpm --dir tests/);
  assert.match(workflow, /- name: Upload app e2e artifacts\s+if: failure\(\)/);
  assert.match(workflow, /name: app-e2e-artifacts-\$\{\{ matrix\.suite \}\}/);
  assert.match(workflow, /tests\/app\/results\//);
  assert.match(workflow, /tests\/app\/screenshots\//);
  assert.match(workflow, /tests\/app\/playwright-report\//);
  assert.match(workflow, /tests\/playwright-report\//);
  assert.match(workflow, /if-no-files-found:\s*ignore/);
});

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("app e2e workflow runs on schedule + workflow_dispatch and uploads failure artifacts", () => {
  const workflow = readRepoFile(".github", "workflows", "e2e.yml");

  // App E2E and VRT are slow and now run nightly on schedule plus on demand
  // via workflow_dispatch. They no longer block PR merges (faster gates take
  // over there). Regressions still gate release via the readiness pipeline.
  assert.match(workflow, /schedule:[\s\S]*?- cron:\s*"/);
  assert.doesNotMatch(workflow, /pull_request:/);
  assert.match(workflow, /name: app-e2e \(\$\{\{ matrix\.suite \}\}\)/);
  assert.match(workflow, /fail-fast:\s*false/);
  // Scheduled runs exercise every suite, including vrt.
  assert.match(workflow, /fromJSON\('\["dev","vrt","preview","build","check","lint"\]'\)/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /type:\s*choice/);
  for (const suite of ["dev", "vrt", "preview", "check", "lint", "build"]) {
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
  assert.doesNotMatch(workflow, /pnpm --dir tests/);
  assert.match(workflow, /- name: Upload app e2e artifacts\s+if: failure\(\)/);
  assert.match(workflow, /name: app-e2e-artifacts-\$\{\{ matrix\.suite \}\}/);
  assert.match(workflow, /tests\/app\/results\//);
  assert.match(workflow, /tests\/app\/screenshots\//);
  assert.match(workflow, /tests\/app\/playwright-report\//);
  assert.match(workflow, /tests\/playwright-report\//);
  assert.match(workflow, /if-no-files-found:\s*ignore/);
});

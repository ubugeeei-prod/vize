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
  assert.match(workflow, /- all/);
  assert.match(
    workflow,
    /github\.event_name == 'workflow_dispatch' && inputs\.suite != 'all' && fromJSON\(format\('\["\{0\}"\]', inputs\.suite\)\) \|\| fromJSON\('\["dev","vrt","preview","build","check","lint"\]'\)/,
  );
  for (const suite of ["dev", "vrt", "preview", "check", "lint", "build"]) {
    assert.match(workflow, new RegExp(`- ${suite}`));
    assert.match(workflow, new RegExp(`${suite}\\)\\n\\s+`));
  }

  assert.match(workflow, /--filter '\.\/tests\.\.\.'/);
  assert.match(workflow, /--filter '\.\/npm\/native\.\.\.'/);
  assert.match(workflow, /--filter '\.\/npm\/builder\/vite\.\.\.'/);
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

test("app e2e dispatch pins validated exact-SHA checkouts and run evidence", () => {
  const workflow = readRepoFile(".github", "workflows", "e2e.yml");

  assert.match(
    workflow,
    /target_sha:\n\s+description:\s*Full lowercase 40-character target SHA; --ref must be a branch or tag at this commit\n\s+required:\s*false\n\s+type:\s*string/,
  );
  assert.match(
    workflow,
    /run-name:[^\n]*inputs\.testbox_id \|\| inputs\.suite[^\n]*'nightly'[^\n]*inputs\.target_sha \|\| github\.sha/,
  );
  assert.match(
    workflow,
    /group:[^\n]*inputs\.testbox_id \|\| inputs\.suite[^\n]*github\.ref[^\n]*inputs\.target_sha \|\| github\.sha[^\n]*github\.event_name/,
  );
  assert.match(workflow, /head_sha comes from --ref/);
  assert.match(workflow, /branch or tag whose tip is target_sha/);
  assert.match(workflow, /E2E_TARGET_SHA:\s*\$\{\{\s*inputs\.target_sha \|\| github\.sha\s*\}\}/);

  const checkouts = workflow.match(
    /ref:\s*\$\{\{\s*env\.E2E_TARGET_SHA\s*\}\}\n\s+submodules:\s*recursive/g,
  );
  assert.equal(checkouts?.length, 2, "both Testbox and app-e2e must pin checkout");
  assert.match(workflow, /name:\s*Validate optional target SHA/);
  assert.match(
    workflow,
    /name:\s*Validate optional target SHA\n\s+if:\s*\$\{\{\s*inputs\.target_sha != ''\s*\}\}/,
  );
  assert.match(workflow, /name:\s*Validate target SHA/);
  assert.match(workflow, /REQUESTED_SUITE:\s*\$\{\{\s*inputs\.suite\s*\}\}/);
  assert.match(workflow, /"\$REQUESTED_SUITE" == "all"/);
  assert.match(workflow, /target_sha is required when suite=all/);
  assert.match(
    workflow,
    /if \[\[ -z "\$REQUESTED_TARGET_SHA" \]\]; then[\s\S]*if \[\[ "\$REQUESTED_SUITE" == "all" \]\]; then[\s\S]*exit 1[\s\S]*fi\n\s+exit 0/,
  );
  assert.equal(
    workflow.match(/\^\[0-9a-f\]\{40\}\$/g)?.length,
    2,
    "both dispatch paths validate a full SHA",
  );
  assert.equal(
    workflow.match(/"\$RUN_HEAD_SHA" != "\$REQUESTED_TARGET_SHA"/g)?.length,
    2,
    "both dispatch paths bind requested checkout to workflow run head_sha",
  );
  assert.equal(
    workflow.match(/run head_sha comes from --ref/g)?.length,
    2,
    "both dispatch paths explain the ref constraint",
  );
});

import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const yamlStepBody = (document: string, selector: { id: string } | { name: string }): string => {
  const lines = document.split(/\r?\n/);
  const marker = "id" in selector ? `id: ${selector.id}` : `- name: ${selector.name}`;
  const markerIndex = lines.findIndex((line) => line.trim() === marker);
  assert.notEqual(markerIndex, -1, `YAML step ${marker}`);

  let start = markerIndex;
  while (start >= 0 && !lines[start].trimStart().startsWith("- ")) start -= 1;
  assert.ok(start >= 0, `YAML list item for ${marker}`);
  const indentation = lines[start].search(/\S/);
  let end = start + 1;
  while (end < lines.length) {
    const line = lines[end];
    if (
      line.trim() !== "" &&
      line.search(/\S/) === indentation &&
      line.trimStart().startsWith("- ")
    ) {
      break;
    }
    end += 1;
  }
  return lines.slice(start, end).join("\n");
};

const shellWords = (command: string, label: string): string[] => {
  const logicalCommand = command.trim().replace(/\\\r?\n[ \t]*/g, " ");
  assert.doesNotMatch(logicalCommand, /\r?\n/, `${label} must be one logical shell command`);
  return [...logicalCommand.matchAll(/'([^']*)'|"([^"]*)"|(\S+)/g)].map(
    (match) => match[1] ?? match[2] ?? match[3],
  );
};

test("full app e2e workflow remains nightly/on-demand and uploads failure artifacts", () => {
  const workflow = readRepoFile(".github", "workflows", "e2e.yml");
  const appJob = workflowJobBody(workflow, "app-e2e");

  // Expensive real-world matrices remain nightly/on-demand. Pull requests use
  // the focused readiness job asserted below.
  assert.match(workflow, /schedule:[\s\S]*?- cron:\s*"/);
  assert.match(appJob, /name: app-e2e \(\$\{\{ matrix\.suite \}\}\)/);
  assert.match(
    appJob,
    /if:\s*\$\{\{\s*github\.event_name != 'pull_request' && \(github\.event_name != 'workflow_dispatch' \|\| inputs\.testbox_id == ''\)\s*\}\}/,
  );
  assert.match(appJob, /fail-fast:\s*false/);
  // Scheduled runs exercise every suite, including vrt.
  assert.match(appJob, /fromJSON\('\["dev","vrt","preview","build","check","lint"\]'\)/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /type:\s*choice/);
  assert.match(workflow, /- all/);
  assert.match(
    appJob,
    /github\.event_name == 'workflow_dispatch' && inputs\.suite != 'all' && fromJSON\(format\('\["\{0\}"\]', inputs\.suite\)\) \|\| fromJSON\('\["dev","vrt","preview","build","check","lint"\]'\)/,
  );
  for (const suite of ["dev", "vrt", "preview", "check", "lint", "build"]) {
    assert.match(workflow, new RegExp(`- ${suite}`));
    assert.match(appJob, new RegExp(`${suite}\\)\\n\\s+`));
  }

  assert.match(appJob, /--filter '\.\/tests\.\.\.'/);
  assert.match(appJob, /--filter '\.\/npm\/native\.\.\.'/);
  assert.match(appJob, /--filter '\.\/npm\/builder\/vite\.\.\.'/);
  assert.match(appJob, /Build native package/);
  assert.match(appJob, /Build vize CLI/);
  assert.match(appJob, /cargo build --profile ci -p vize/);
  assert.match(appJob, /uses: \.\/\.github\/actions\/setup-moonbit/);
  assert.match(appJob, /Cache Playwright browsers/);
  assert.match(appJob, /contains\(fromJSON\('\["dev","vrt"\]'\), matrix\.suite\)/);
  assert.match(appJob, /vp exec --filter '\.\/tests' -- playwright install --with-deps chromium/);
  assert.match(appJob, /RUN_BUILD_TESTS=1 vp run --filter '\.\/tests' test:preview/);
  assert.match(appJob, /vp run --filter '\.\/tests' test:dev:ci/);
  assert.match(appJob, /vp run --filter '\.\/tests' test:check/);
  assert.match(appJob, /vp run --filter '\.\/tests' test:lint/);
  assert.doesNotMatch(appJob, /pnpm --dir tests/);
  assert.match(appJob, /- name: Upload app e2e artifacts\s+if: failure\(\)/);
  assert.match(appJob, /name: app-e2e-artifacts-\$\{\{ matrix\.suite \}\}/);
  assert.match(appJob, /tests\/app\/results\//);
  assert.match(appJob, /tests\/app\/screenshots\//);
  assert.match(appJob, /tests\/app\/playwright-report\//);
  assert.match(appJob, /tests\/playwright-report\//);
  for (const fixture of ["elk", "frontend-phpcon", "misskey", "npmx", "vuefes"]) {
    assert.match(appJob, new RegExp(`\\.vize/artifacts/${fixture}-vrt/artifacts/`));
  }
  assert.match(appJob, /if-no-files-found:\s*ignore/);
});

test("pull requests run one path-filtered fast app readiness job", () => {
  const workflow = readRepoFile(".github", "workflows", "e2e.yml");
  const job = workflowJobBody(workflow, "app-readiness");
  const pullRequestBlock = workflow.slice(
    workflow.indexOf("\n  pull_request:\n"),
    workflow.indexOf("\n  schedule:\n"),
  );

  assert.match(pullRequestBlock, /branches:\s*\[main\]/);
  assert.doesNotMatch(pullRequestBlock, /paths:/);
  assert.match(
    workflow,
    /run-name:[^\n]*format\('readiness-pr-\{0\}', github\.event\.pull_request\.number\)/,
  );

  for (const path of [
    "'.github/actions/app-readiness/**'",
    "'.github/workflows/e2e.yml'",
    "'.gitmodules'",
    "'Cargo.lock'",
    "'crates/**'",
    "'npm/native/**'",
    "'npm/cli/**'",
    "'npm/builder/vite/**'",
    "'npm/framework/nuxt/**'",
    "'npm/framework/nuxt-lint-config/**'",
    "'pnpm-lock.yaml'",
    "'tests/package.json'",
    "'tests/tsconfig.json'",
    "'tests/_fixtures/_git/{elk,misskey,npmx.dev,nuxt-ui,reka-ui}'",
    "'tests/_fixtures/_projects/compiler-macros/**'",
    "'tests/_helpers/**'",
    "'tests/app/dev/misskey.spec.ts'",
    "'tests/snapshots/check/compiler-macros.ts'",
    "'tests/snapshots/check/{elk,misskey,npmx,nuxt-ui,reka-ui}.ts'",
    "'tests/snapshots/check/__snapshots__/{elk,misskey,npmx.dev,nuxt-ui,reka-ui}-check.snap'",
    "'tests/snapshots/lint/{elk,misskey,npmx,nuxt-ui,reka-ui}.ts'",
    "'tests/snapshots/lint/__snapshots__/{elk,misskey,npmx.dev,nuxt-ui,reka-ui}-lint.snap'",
    "'tests/snapshots/build/elk.ts'",
    "'tests/tooling/cli-lint-contract.test.ts'",
    "'tools/moon/**'",
    "'tools/vite-plus/**'",
    "'tsconfig.json'",
    "'vite.config.ts'",
  ]) {
    assert.match(job, new RegExp(`- ${path.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`));
  }
  assert.doesNotMatch(job, /- ['"]docs\/\*\*/);
  for (const broadPath of ["npm", "tests", "tools"]) {
    assert.doesNotMatch(job, new RegExp(`- ['"]${broadPath}/\\*\\*['"]`));
  }

  assert.match(job, /if:\s*\$\{\{\s*github\.event_name == 'pull_request'\s*\}\}/);
  assert.match(job, /runs-on:\s*blacksmith-32vcpu-ubuntu-2404/);
  assert.match(job, /timeout-minutes:\s*30\b/);
  assert.match(job, /contents:\s*read\s*# required by actions\/checkout/);
  assert.match(job, /pull-requests:\s*read\s*# required by dorny\/paths-filter/);
  assert.match(
    job,
    /uses:\s*dorny\/paths-filter@7b450fff21473bca461d4b92ce414b9d0420d706\s*# v4\.0\.2/,
  );
  assert.match(job, /id:\s*changes/);
  assert.match(job, /filters:\s*\|\n\s+readiness:/);
  const checkout = yamlStepBody(job, { name: "Checkout readiness source" });
  assert.match(checkout, /uses:\s*actions\/checkout@[0-9a-f]{40}\s*# v6/);
  assert.match(checkout, /if:\s*steps\.changes\.outputs\.readiness == 'true'/);
  assert.match(checkout, /persist-credentials:\s*false/);
  const readiness = yamlStepBody(job, { name: "Run app readiness" });
  assert.match(readiness, /if:\s*steps\.changes\.outputs\.readiness == 'true'/);
  assert.match(readiness, /uses:\s*\.\/\.github\/actions\/app-readiness/);
  assert.doesNotMatch(job, /cargo build|vp run|app-readiness-artifacts/);
});

test("local app readiness action keeps setup, diagnostics, and aggregation bounded", () => {
  const action = readRepoFile(".github", "actions", "app-readiness", "action.yml");

  assert.match(action, /runs:\n\s+using:\s*composite/);
  const hydration = yamlStepBody(action, { name: "Hydrate pinned app fixtures" });
  const runMarker = "run: |";
  const runStart = hydration.indexOf(runMarker);
  assert.notEqual(runStart, -1, "hydration step run script");
  assert.deepEqual(
    shellWords(hydration.slice(runStart + runMarker.length), "hydration run script"),
    [
      "git",
      "submodule",
      "update",
      "--init",
      "--recursive",
      "--depth",
      "1",
      "tests/_fixtures/_git/elk",
      "tests/_fixtures/_git/misskey",
      "tests/_fixtures/_git/npmx.dev",
      "tests/_fixtures/_git/nuxt-ui",
      "tests/_fixtures/_git/reka-ui",
    ],
    "readiness must hydrate exactly the pinned readiness fixtures",
  );
  const setupRust = yamlStepBody(action, { name: "Setup Rust" });
  assert.match(setupRust, /id:\s*rust-toolchain/);
  assert.match(setupRust, /uses:\s*dtolnay\/rust-toolchain@[0-9a-f]{40}\s*# stable/);
  assert.match(action, /uses:\s*wild-linker\/action@[0-9a-f]{40}\s*# v0\.9\.0/);
  assert.match(action, /uses:\s*\.\/\.github\/actions\/setup-rust-sticky-cache/);
  assert.match(action, /cache-key-suffix:[^\n]*steps\.rust-toolchain\.outputs\.cachekey/);
  assert.doesNotMatch(action, /cache-key-suffix:[^\n]*hashFiles/);
  assert.match(action, /uses:\s*\.\/\.github\/actions\/setup-moonbit/);
  assert.match(action, /cargo build --profile ci -p vize/);

  const browserCache = yamlStepBody(action, { name: "Cache Playwright browsers" });
  assert.match(browserCache, /id:\s*cache-playwright/);
  const installDependencies = yamlStepBody(action, {
    name: "Install Playwright OS dependencies",
  });
  assert.match(installDependencies, /shell:\s*bash/);
  assert.match(
    installDependencies,
    /run:\s*vp exec --filter '\.\/tests' -- playwright install-deps chromium/,
  );
  const installBrowser = yamlStepBody(action, { name: "Install Playwright browser" });
  assert.match(installBrowser, /if:\s*steps\.cache-playwright\.outputs\.cache-hit != 'true'/);
  assert.match(installBrowser, /playwright install chromium/);
  assert.doesNotMatch(action, /playwright install --with-deps chromium/);

  for (const [id, script, log, timeout] of [
    ["check", "test:readiness:check", "check.log", "3m"],
    ["lint", "test:readiness:lint", "lint.log", "2m"],
    ["build", "test:readiness:build", "build.log", "3m"],
    ["dev", "test:readiness:dev", "dev.log", "3m"],
  ] as const) {
    const step = yamlStepBody(action, { id });
    assert.match(step, /continue-on-error:\s*true/);
    assert.match(step, /shell:\s*bash/);
    assert.ok(step.includes("set -o pipefail"));
    assert.ok(step.includes(`timeout --signal=TERM --kill-after=15s ${timeout}`));
    assert.ok(step.includes(script));
    assert.ok(step.includes(`tee target/app-readiness-logs/${log}`));
  }

  const upload = yamlStepBody(action, { name: "Upload app readiness artifacts" });
  assert.match(upload, /if:\s*\$\{\{\s*cancelled\(\)\s*\|\|\s*failure\(\)/);
  for (const id of ["check", "lint", "build", "dev"]) {
    assert.match(upload, new RegExp(`steps\\.${id}\\.outcome\\s*==\\s*['"]failure['"]`));
  }
  assert.match(upload, /name:\s*app-readiness-artifacts/);
  assert.match(upload, /target\/app-readiness-logs\//);
  assert.match(upload, /if-no-files-found:\s*ignore/);
  assert.match(action, /## Fast app readiness/);
  for (const [outcome, step] of [
    ["CHECK_OUTCOME", "check"],
    ["LINT_OUTCOME", "lint"],
    ["BUILD_OUTCOME", "build"],
    ["DEV_OUTCOME", "dev"],
  ]) {
    assert.ok(action.includes(`${outcome}: \${{ steps.${step}.outcome }}`));
  }
  assert.match(action, /failed Actions step/);
  assert.match(action, /captured suite logs in app-readiness-artifacts/);
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

  const checkouts = workflow.match(/ref:\s*\$\{\{\s*env\.E2E_TARGET_SHA\s*\}\}/g);
  assert.equal(checkouts?.length, 2, "both Testbox and app-e2e must pin checkout");
  assert.doesNotMatch(workflow, /submodules:\s*recursive/);
  assert.equal(
    workflow.match(/uses:\s*\.\/\.github\/actions\/hydrate-app-fixtures/g)?.length,
    2,
    "both full E2E jobs must hydrate only their exercised fixtures",
  );
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

test("full app e2e hydrates only fixtures referenced by its scripts", () => {
  const action = readRepoFile(".github", "actions", "hydrate-app-fixtures", "action.yml");
  const fixtures = [
    "ant-design-vue",
    "directus",
    "element-plus",
    "elk",
    "frontend-phpcon-do-website",
    "hoppscotch",
    "misskey",
    "naive-ui",
    "npmx.dev",
    "nuxt-ui",
    "primevue",
    "reka-ui",
    "voicevox",
    "vue-vben-admin",
    "vuefes-2025",
    "vuetify",
  ];

  assert.match(action, /git submodule update --init --recursive --depth 1/);
  for (const fixture of fixtures) {
    assert.match(action, new RegExp(`tests/_fixtures/_git/${fixture.replace(".", "\\.")}`));
  }
  assert.equal(action.match(/tests\/_fixtures\/_git\//g)?.length, fixtures.length);
});

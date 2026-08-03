import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

type CompositeActionStep = {
  env?: Record<string, string>;
  if?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, number | string>;
};

test("the typecheck divergence ratchet runs on every pull request", () => {
  // Dimension 1 of the vue-tsc parity scorecard (#3222) is a *per-PR* ratchet:
  // tooling/compat-ratchet.test.ts may only hold or improve the divergence
  // ledger in tests/_fixtures/compat-baseline.json. It reaches the PR lane only
  // because the vue-parity job carries no event guard — three sibling jobs in
  // this same workflow (nix-flake, source-coverage, branch-coverage) opt out of
  // pull requests with exactly such a guard for runtime reasons. Adding one to
  // vue-parity would silently un-gate the ledger with every other assertion in
  // this file still passing, so pin the trigger and the absence of the guard.
  const workflow = readRepoFile(".github", "workflows", "check.yml");

  assert.match(workflow, /\n  pull_request:\n    branches: \[main\]\n/);
  assert.doesNotMatch(
    workflowJobBody(workflow, "vue-parity"),
    /^ {4}if:/m,
    "vue-parity must stay unconditional: it is the only pre-merge gate on the divergence ledger",
  );
});

test("Vue parity structurally gates compiler fixtures and incremental LSP behavior", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const job = workflowJobBody(workflow, "vue-parity");
  const action = parse(readRepoFile(".github", "actions", "check-vue-parity", "action.yml")) as {
    runs?: { steps?: CompositeActionStep[]; using?: string };
  };
  const testsPackage = JSON.parse(readRepoFile("tests", "package.json"));

  assert.match(job, /uses:\s*\.\/\.github\/actions\/check-vue-parity/);
  assert.equal(action.runs?.using, "composite");
  const steps = action.runs?.steps ?? [];
  const hydration = steps.find(
    (step) => step.name === "Hydrate fixtures and install JS dependencies",
  );
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/create-vue/);
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/element-plus/);
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/misskey/);
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/nuxt-ui/);
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/pinia/);
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/vue-router/);
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/vue-element-admin/);
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/vue-vben-admin/);
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/vitepress/);
  assert.match(hydration?.run ?? "", /vp install --frozen-lockfile --prefer-offline/);
  assert.equal(
    steps.find((step) => step.name === "Build vize CLI")?.run,
    "cargo build --profile ci -p vize --features legacy",
  );
  const parity = steps.find((step) => step.name === "Check Vue compiler and typecheck parity");
  assert.deepEqual(parity?.env, { VIZE_TEST_BIN: "target/ci/vize" });
  assert.equal(parity?.run, "vp run --filter './tests' test:check:fixtures");
  // The per-PR drop-in compatibility ratchet must ride the same lane: it is
  // the only pre-merge gate holding the vize/vue-tsc divergence ledger.
  assert.match(
    testsPackage.scripts["test:check:fixtures"],
    /tooling\/compat-ratchet\.test\.ts/,
    "test:check:fixtures must run the per-PR compat ratchet",
  );
  assert.match(
    testsPackage.scripts["test:check:fixtures"],
    /^VIZE_TEST_REQUIRE_TSGO=1 /,
    "typecheck parity must fail closed when tsgo is unavailable",
  );
  assert.match(
    testsPackage.scripts["test:check:fixtures"],
    /snapshots\/check\/vue-benchmarks-correctness-plants\.ts/,
    "typecheck parity must run the upstream correctness plants",
  );
  assert.match(
    testsPackage.scripts["test:check:fixtures"],
    /snapshots\/check\/zz-intentional-errors-fixtures\.ts/,
    "typecheck parity must run the batch broken-to-repaired oracle",
  );
  // A bare text match would also pass on a mention in a comment or a leftover
  // import, so pin the two facts that make the Tier-L project actually run:
  // vueVbenAdminApp is the real app config from _helpers/apps.ts, and it is a
  // member of the array the suite iterates.
  const batchOracle = readRepoFile(
    "tests",
    "snapshots",
    "check",
    "zz-intentional-errors-fixtures.ts",
  ).replace(/\/\/[^\n]*/g, "");
  const identifiers = (list: string | undefined) =>
    (list ?? "").split(",").map((entry) => entry.trim());
  const appsImport = /import \{([\s\S]*?)\} from "\.\.\/\.\.\/_helpers\/apps\.ts";/.exec(
    batchOracle,
  );
  assert.ok(
    identifiers(appsImport?.[1]).includes("vueVbenAdminApp"),
    "the per-PR batch repair oracle must import the Tier-L app config from _helpers/apps.ts",
  );
  const fixtureAppList = /const fixtureApps = \[([\s\S]*?)\]/.exec(batchOracle);
  assert.ok(
    identifiers(fixtureAppList?.[1]).includes("vueVbenAdminApp"),
    "the per-PR batch repair oracle must retain a Tier-L project",
  );
  assert.match(
    batchOracle,
    /for \(const app of fixtureApps\)/,
    "the per-PR batch repair oracle must run every registered app",
  );

  const glyphProperties = steps.find(
    (step) => step.name === "Check glyph formatter corpus properties",
  );
  assert.deepEqual(glyphProperties?.env, { VIZE_TEST_BIN: "target/ci/vize" });
  assert.match(glyphProperties?.run ?? "", /--test-concurrency=1/);
  for (const property of ["idempotence", "parse-preservation", "lint-agreement"]) {
    assert.match(
      glyphProperties?.run ?? "",
      new RegExp(`tests/tooling/glyph-corpus-${property}\\.test\\.ts`),
    );
  }

  const incremental = steps.find(
    (step) => step.name === "Check incremental LSP against Misskey and Vue Vben Admin",
  );
  assert.deepEqual(incremental?.env, { VIZE_LSP_BIN: "target/ci/vize" });
  assert.equal(incremental?.run, "vp run --filter './tests' test:performance:lsp-incremental");

  const churn = steps.find((step) => step.name === "Stress LSP edit churn against Misskey");
  assert.deepEqual(churn?.env, { VIZE_LSP_BIN: "target/ci/vize" });
  assert.equal(churn?.run, "vp run --filter './tests' test:performance:lsp-churn");

  const summary = steps.find((step) => step.name === "Publish incremental LSP summaries");
  assert.equal(summary?.if, "${{ always() }}");
  assert.match(
    summary?.run ?? "",
    /misskey-lsp-incremental vben-lsp-incremental misskey-lsp-churn/,
  );
  assert.match(summary?.run ?? "", /summary\.md/);
  assert.match(summary?.run ?? "", /GITHUB_STEP_SUMMARY/);

  const uploads: Array<[stepName: string, suiteDir: string]> = [
    ["Upload Misskey incremental LSP metrics", "misskey-lsp-incremental"],
    ["Upload Vue Vben Admin incremental LSP metrics", "vben-lsp-incremental"],
    ["Upload Misskey churn LSP metrics", "misskey-lsp-churn"],
  ];
  for (const [stepName, suiteDir] of uploads) {
    const upload = steps.find((step) => step.name === stepName);
    assert.equal(upload?.if, "${{ always() }}");
    assert.match(upload?.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
    assert.deepEqual(upload?.with, {
      name: `${suiteDir}-metrics`,
      path: `target/vize-tests/metrics/${suiteDir}/`,
      "if-no-files-found": "warn",
      "retention-days": 14,
    });
  }
  assert.equal(
    testsPackage.scripts["test:performance:lsp-incremental"],
    "node --test --test-concurrency=1 performance/misskey-lsp-incremental.test.ts performance/vben-lsp-incremental.test.ts",
  );
  assert.equal(
    testsPackage.scripts["test:performance:lsp-churn"],
    "node --test --test-concurrency=1 performance/lsp-churn-stress.test.ts",
  );
});

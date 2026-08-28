import assert from "node:assert/strict";
import { test } from "node:test";

import {
  CHECK_FIXTURE_ENV,
  CHECK_FIXTURE_NODE_ARGS,
  checkFixturePhases,
} from "./support/check-fixtures/manifest.ts";
import { vueParityAction } from "./support/check-vue-parity-action.ts";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("the typecheck divergence ratchet runs on every pull request", () => {
  // Dimension 1 of the vue-tsc parity scorecard (#3222) is a *per-PR* ratchet:
  // tooling/compat-ratchet.test.ts may only hold or improve the divergence
  // ledger in tests/_fixtures/compat-baseline.json. It reaches the PR lane only
  // because the vue-parity job carries no event guard — three sibling jobs in
  // this same workflow (nix-flake, source-coverage, branch-coverage) opt out of
  // pull requests with exactly such a guard for runtime reasons. Adding one to
  // vue-parity would silently un-gate the ledger with every other assertion in
  // this file still passing, so pin the trigger and the absence of the guard.
  // The davinci integration branch carries the same required gates, so its
  // PRs ride the identical trigger (davinci-road/roadmap.md standing gates).
  const workflow = readRepoFile(".github", "workflows", "check.yml");

  assert.match(workflow, /\n  pull_request:\n    branches: \[main, davinci\]\n/);
  assert.doesNotMatch(
    workflowJobBody(workflow, "vue-parity"),
    /^ {4}if:/m,
    "vue-parity must stay unconditional: it is the only pre-merge gate on the divergence ledger",
  );
});

test("Vue parity structurally gates compiler fixtures and incremental LSP behavior", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const job = workflowJobBody(workflow, "vue-parity");
  const action = vueParityAction();
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
  const build = steps.find((step) => step.name === "Build vize CLI");
  assert.deepEqual(build?.env, { RUNNER_ENVIRONMENT: "${{ runner.environment }}" });
  assert.match(
    build?.run ?? "",
    /cargo_args=\(build --profile ci -p vize --features legacy\)/,
    "the hosted-runner fallback must keep the same vize build command shape",
  );
  assert.match(
    build?.run ?? "",
    /\[ "\$RUNNER_ENVIRONMENT" = "github-hosted" \]/,
    "the hosted-runner fallback must make build pressure runner-aware",
  );
  assert.match(
    build?.run ?? "",
    /cargo_args\+=\(--jobs 2\)/,
    "GitHub-hosted vue-parity builds must avoid saturating the smaller runner before Corsa starts",
  );
  assert.match(build?.run ?? "", /cargo "\$\{cargo_args\[@\]\}"/);
  // The first step of the action writes the process-budget baseline, so the
  // always-uploaded artifact exists even when a later step is killed by the
  // spawn exhaustion this lane is guarding against (#4126).
  assert.equal(steps[0]?.name, "Record runner process budget baseline");
  // The runner profile only reaches the script through this mapping; what the
  // script then emits for each profile is executed below.
  assert.deepEqual(steps[0]?.env, { RUNNER_ENVIRONMENT: "${{ runner.environment }}" });

  const buildIndex = steps.findIndex((step) => step.name === "Build vize CLI");
  const settle = steps[buildIndex + 1];
  assert.equal(settle?.name, "Settle hosted runner task budget");
  assert.equal(settle?.if, "${{ runner.environment == 'github-hosted' }}");
  assert.match(
    settle?.run ?? "",
    /VIZE_RUNNER_BASELINE_THREADS/,
    "hosted-runner settling must compare against the captured baseline",
  );
  assert.match(
    settle?.run ?? "",
    /budget_threads=\$\(\(baseline_threads \+ 96\)\)/,
    "hosted-runner settling must reserve the fixture cycle's bounded process budget",
  );
  assert.match(
    settle?.run ?? "",
    /post-build-settle\.txt/,
    "hosted-runner settling must publish its samples with the existing topology artifact",
  );

  const parity = steps.find((step) => step.name === "Check Vue compiler and typecheck parity");
  assert.deepEqual(parity?.env, { VIZE_TEST_BIN: "target/ci/vize" });
  assert.equal(parity?.run, "vp run --filter './tests' test:check:fixtures");
  const phaseFiles = checkFixturePhases.map((phase) => phase.file);
  // The per-PR drop-in compatibility ratchet must ride the same lane: it is
  // the only pre-merge gate holding the vize/vue-tsc divergence ledger.
  assert.ok(
    phaseFiles.includes("tooling/compat-ratchet.test.ts"),
    "test:check:fixtures must run the per-PR compat ratchet",
  );
  assert.deepEqual(
    { ...CHECK_FIXTURE_ENV },
    { VIZE_TEST_REQUIRE_TSGO: "1" },
    "typecheck parity must fail closed when tsgo is unavailable",
  );
  assert.ok(
    CHECK_FIXTURE_NODE_ARGS.includes("--test-concurrency=1"),
    "typecheck parity phases must stay serial",
  );
  assert.ok(
    phaseFiles.includes("snapshots/check/vue-benchmarks-correctness-plants.ts"),
    "typecheck parity must run the upstream correctness plants",
  );
  assert.ok(
    phaseFiles.includes("snapshots/check/zz-intentional-errors-fixtures.ts"),
    "typecheck parity must run the batch broken-to-repaired oracle",
  );

  const cycles = steps.find(
    (step) => step.name === "Bound fixture process topology under a constrained PID budget",
  );
  assert.deepEqual(cycles?.env, {
    VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR: "${{ env.VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR }}",
    VIZE_TEST_BIN: "target/ci/vize",
  });
  // `--no-cache` carries the step's environment, so it belongs to the contract
  // rather than to cache hygiene: Vite+ hands a cached script only its own
  // passthrough set plus the variables the task config declares, and a
  // `package.json` script has no task config to declare them in, so a cached
  // invocation reaches the harness without the floor above, without
  // `GOMAXPROCS`, and without `RAYON_NUM_THREADS`. It also keeps a
  // process-budget measurement from being replayed instead of measured.
  assert.equal(cycles?.run, "vp run --no-cache --filter './tests' test:check:fixtures:cycles");
  assert.equal(
    testsPackage.scripts["test:check:fixtures:cycles"],
    "node tooling/support/check-fixtures/cycles.ts",
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

  const batchIncremental = steps.find(
    (step) => step.name === "Check Tier-L batch incremental session reuse",
  );
  assert.deepEqual(batchIncremental?.env, {
    VIZE_TIER_L_FIXTURE: "tests/_fixtures/_git/vue-vben-admin",
    VIZE_TIER_L_CORSA_BIN: "node_modules/@typescript/typescript-linux-x64/lib/tsc",
    VIZE_TIER_L_METRICS_DIR: "target/vize-tests/metrics/vben-batch-incremental",
    VIZE_TIER_L_BUDGET_SCALE: "${{ env.VIZE_TIER_L_BUDGET_SCALE }}",
    VIZE_RUNTIME_NODE_MODULES: "node_modules",
  });
  assert.equal(
    batchIncremental?.run,
    "cargo test --locked --profile ci -p vize_canon --test tier_l_incremental -- --ignored --nocapture",
  );

  const vue2Build = steps.find((step) => step.name === "Check Vue 2 build snapshots");
  assert.deepEqual(vue2Build?.env, { VIZE_TEST_BIN: "target/ci/vize" });
  assert.equal(
    vue2Build?.run,
    "node --test --test-concurrency=1 tests/snapshots/build/vue2-filters.ts",
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
  for (const oracle of [
    "sfc-baseline-routes",
    "sfc-baselines",
    "vue2-render-signature",
    "glyph-sfc-evidence",
    "sfc-equivalence",
  ]) {
    assert.match(glyphProperties?.run ?? "", new RegExp(`tests/tooling/${oracle}\\.test\\.ts`));
  }

  const incremental = steps.find(
    (step) => step.name === "Check incremental LSP against Misskey and Vue Vben Admin",
  );
  assert.deepEqual(incremental?.env, {
    VIZE_LSP_BIN: "target/ci/vize",
    VIZE_PERF_BUDGET_SCALE: "${{ env.VIZE_PERF_BUDGET_SCALE }}",
  });
  assert.equal(
    incremental?.run,
    "vp run --no-cache --filter './tests' test:performance:lsp-incremental",
  );

  const churn = steps.find((step) => step.name === "Stress LSP edit churn against Misskey");
  assert.deepEqual(churn?.env, {
    VIZE_LSP_BIN: "target/ci/vize",
    VIZE_PERF_BUDGET_SCALE: "${{ env.VIZE_PERF_BUDGET_SCALE }}",
  });
  assert.equal(churn?.run, "vp run --no-cache --filter './tests' test:performance:lsp-churn");

  const summary = steps.find((step) => step.name === "Publish incremental LSP summaries");
  assert.equal(summary?.if, "${{ always() }}");
  // Asserted one directory at a time: a single ordered pattern fails without
  // saying which suite went missing, and it breaks on a harmless reordering.
  const summarisedDirectories = [
    "check-fixtures-topology",
    "check-fixtures-cycles",
    "vben-batch-incremental",
    "misskey-lsp-incremental",
    "vben-lsp-incremental",
    "misskey-lsp-churn",
  ];
  for (const directory of summarisedDirectories) {
    assert.ok(
      (summary?.run ?? "").includes(directory),
      `the summary step must publish the ${directory} metrics directory`,
    );
  }
  assert.match(summary?.run ?? "", /summary\.md/);
  assert.match(summary?.run ?? "", /GITHUB_STEP_SUMMARY/);

  const uploads: Array<[stepName: string, suiteDir: string]> = [
    ["Upload Vue Vben Admin batch incremental metrics", "vben-batch-incremental"],
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

  // The topology artifact is the one upload that may not be missing: its first
  // rows are written before anything can fail, so an empty directory means the
  // evidence was lost rather than never produced.
  const topology = steps.find((step) => step.name === "Upload fixture process topology");
  assert.equal(topology?.if, "${{ always() }}");
  assert.match(topology?.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(topology?.with, {
    name: "check-fixtures-topology",
    path: "target/vize-tests/metrics/check-fixtures-topology/",
    "if-no-files-found": "error",
    "retention-days": 14,
  });
  const cycleUpload = steps.find((step) => step.name === "Upload fixture process budget cycles");
  assert.equal(cycleUpload?.if, "${{ always() }}");
  assert.match(cycleUpload?.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(cycleUpload?.with, {
    name: "check-fixtures-cycles",
    path: "target/vize-tests/metrics/check-fixtures-cycles/",
    "if-no-files-found": "warn",
    "retention-days": 14,
  });
  assert.equal(
    testsPackage.scripts["test:performance:lsp-incremental"],
    "node --test --test-concurrency=1 performance/misskey-lsp-incremental.test.ts performance/vben-lsp-incremental.test.ts",
  );
  assert.equal(
    testsPackage.scripts["test:performance:lsp-churn"],
    "node --test --test-concurrency=1 performance/lsp-churn-stress.test.ts",
  );
});

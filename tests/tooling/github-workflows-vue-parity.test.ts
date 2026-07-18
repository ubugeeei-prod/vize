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
  assert.match(hydration?.run ?? "", /tests\/_fixtures\/_git\/vitepress/);
  assert.match(hydration?.run ?? "", /vp install --frozen-lockfile --prefer-offline/);
  assert.equal(
    steps.find((step) => step.name === "Build vize CLI")?.run,
    "cargo build --profile ci -p vize --features legacy",
  );
  const parity = steps.find((step) => step.name === "Check Vue compiler and typecheck parity");
  assert.deepEqual(parity?.env, { VIZE_TEST_BIN: "target/ci/vize" });
  assert.equal(parity?.run, "vp run --filter './tests' test:check:fixtures");

  const incremental = steps.find((step) => step.name === "Check incremental LSP against Misskey");
  assert.deepEqual(incremental?.env, { VIZE_LSP_BIN: "target/ci/vize" });
  assert.equal(incremental?.run, "vp run --filter './tests' test:performance:lsp-incremental");

  const summary = steps.find((step) => step.name === "Publish incremental LSP summary");
  assert.equal(summary?.if, "${{ always() }}");
  assert.match(summary?.run ?? "", /misskey-lsp-incremental\/summary\.md/);
  assert.match(summary?.run ?? "", /GITHUB_STEP_SUMMARY/);

  const upload = steps.find((step) => step.name === "Upload incremental LSP metrics");
  assert.equal(upload?.if, "${{ always() }}");
  assert.match(upload?.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(upload?.with, {
    name: "misskey-lsp-incremental-metrics",
    path: "target/vize-tests/metrics/misskey-lsp-incremental/",
    "if-no-files-found": "warn",
    "retention-days": 14,
  });
  assert.equal(
    testsPackage.scripts["test:performance:lsp-incremental"],
    "node --test --test-concurrency=1 performance/misskey-lsp-incremental.test.ts",
  );
});

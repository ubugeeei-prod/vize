import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type WorkflowStep = {
  env?: Record<string, string>;
  if?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

type WorkflowJob = {
  env?: Record<string, string>;
  "runs-on"?: string;
  steps?: WorkflowStep[];
  strategy?: { "fail-fast"?: boolean; matrix?: { shard?: number[] } };
  "timeout-minutes"?: number;
};

test("real-project workflow schedules every balanced fixture shard", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "real-project-matrix.yml")) as {
    concurrency?: { "cancel-in-progress"?: boolean; group?: string };
    jobs?: Record<string, WorkflowJob>;
    on?: {
      schedule?: Array<{ cron?: string }>;
      workflow_dispatch?: { inputs?: Record<string, unknown> };
    };
    permissions?: Record<string, string>;
  };
  const job = workflow.jobs?.["real-project-matrix"];

  assert.ok(job);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.equal(workflow.on?.schedule?.[0]?.cron, "37 5 * * 0");
  const dispatch = workflow.on?.workflow_dispatch;
  assert.ok(dispatch, "Missing workflow_dispatch trigger");
  assert.equal(dispatch.inputs, undefined, "release dispatch must expose no record-only mode");
  assert.deepEqual(workflow.concurrency, {
    group: "real-project-matrix-${{ github.ref }}",
    "cancel-in-progress": true,
  });
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 120);
  assert.equal(job.strategy?.["fail-fast"], false);
  assert.deepEqual(job.strategy?.matrix?.shard, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
  assert.deepEqual(job.env, {
    FIXTURE_SHARD_COUNT: "11",
    FIXTURE_SHARD_INDEX: "${{ matrix.shard }}",
    FIXTURE_REPORT_DIR: "real-project-results/shard-${{ matrix.shard }}",
  });
});

test("real-project workflow hydrates only its shard and runs every core tool", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "real-project-matrix.yml")) as {
    jobs?: Record<string, WorkflowJob>;
  };
  const steps = workflow.jobs?.["real-project-matrix"]?.steps ?? [];
  const nodeSetupIndex = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  const shims = steps.find((step) => step.name === "Enable package manager shims");
  assert.ok(shims, "Missing 'Enable package manager shims' step");
  const shimsIndex = steps.indexOf(shims);
  const hydrationIndex = steps.findIndex(
    (step) => step.name === "Select and hydrate fixture shard",
  );
  const hydration = steps.find((step) => step.name === "Select and hydrate fixture shard");
  const dependency = steps.find(
    (step) => step.name === "Install pinned typecheck baseline dependencies",
  );
  assert.ok(dependency, "Missing 'Install pinned typecheck baseline dependencies' step");
  const dependencyIndex = steps.indexOf(dependency);
  const run = steps.find((step) => step.name === "Exercise real projects with every core tool");
  const runIndex = steps.indexOf(run!);
  const lsp = steps.find((step) => step.name === "Check real-project LSP lifecycle");
  assert.ok(lsp, "Missing 'Check real-project LSP lifecycle' step");
  const lspIndex = steps.indexOf(lsp);
  const syntaxHighlighter = steps.find(
    (step) => step.name === "Check real-project syntax highlighting",
  );
  assert.ok(syntaxHighlighter, "Missing 'Check real-project syntax highlighting' step");
  const syntaxHighlighterIndex = steps.indexOf(syntaxHighlighter);
  const glyphProperties = steps.find(
    (step) => step.name === "Check glyph formatter corpus properties",
  );
  const glyphPropertiesIndex = steps.indexOf(glyphProperties!);
  const divergence = steps.find((step) => step.name === "Enforce typechecker baseline divergence");
  const divergenceIndex = steps.indexOf(divergence!);
  const summary = steps.find((step) => step.name === "Publish shard summary");
  const summaryIndex = steps.indexOf(summary!);
  const upload = steps.find((step) => step.name === "Upload shard report");

  assert.notEqual(nodeSetupIndex, -1);
  assert.notEqual(hydrationIndex, -1);
  assert.ok(
    nodeSetupIndex < shimsIndex && shimsIndex < hydrationIndex,
    "Node 24 and package manager shims must be active before loading matrix scripts",
  );
  assert.deepEqual(steps[nodeSetupIndex].with, {
    "node-version-file": ".node-version",
    cache: true,
    "run-install": false,
  });
  assert.equal(shims.run, "corepack enable");
  assert.match(hydration?.run ?? "", /--list-fixture-paths/);
  assert.match(hydration?.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(hydration?.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(hydration?.run ?? "", /mkdir -p "\$FIXTURE_REPORT_DIR"/);
  assert.match(hydration?.run ?? "", /selected-fixtures\.txt/);
  assert.match(hydration?.run ?? "", /git submodule update --init --depth 1/);
  assert.doesNotMatch(hydration?.run ?? "", /--recursive/);
  assert.match(hydration?.run ?? "", /"\$\{fixture_paths\[@\]\}"/);
  assert.ok(hydrationIndex < dependencyIndex && dependencyIndex < runIndex);
  assert.match(dependency.run ?? "", /tools\/fixtures\/typecheck-dependency-prepare\.mjs/);
  assert.match(dependency.run ?? "", /--output-dir "\$FIXTURE_REPORT_DIR"/);
  assert.match(dependency.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(dependency.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(dependency.run ?? "", /--timeout-ms 600000/);
  assert.match(run?.run ?? "", /tools\/fixtures\/tool-matrix-report\.mjs/);
  assert.match(run?.run ?? "", /--vize-bin target\/ci\/vize/);
  assert.match(run?.run ?? "", /--timeout-ms 600000/);
  assert.match(run?.run ?? "", /--output-dir "\$FIXTURE_REPORT_DIR"/);
  assert.ok(
    runIndex < lspIndex && lspIndex < syntaxHighlighterIndex,
    "the hydrated fixture corpus must run through the production LSP before syntax audit",
  );
  assert.deepEqual(lsp.env, {
    CORSA_PATH: "${{ github.workspace }}/node_modules/.bin/tsgo",
    REAL_PROJECT_LSP_TIMEOUT_MS: "600000",
    VIZE_LSP_BIN: "target/ci/vize",
  });
  assert.match(lsp.run ?? "", /if \[ ! -x "\$CORSA_PATH" \]; then/);
  assert.match(lsp.run ?? "", /::error title=Missing typecheck runtime::/);
  assert.match(lsp.run ?? "", /tests\/tooling\/real-project-lsp\.test\.ts/);
  assert.match(lsp.run ?? "", /--test-concurrency=1/);
  assert.match(lsp.run ?? "", /test -s "\$FIXTURE_REPORT_DIR\/lsp-lifecycle-summary\.json"/);
  assert.ok(
    syntaxHighlighterIndex < glyphPropertiesIndex,
    "the hydrated fixture corpus must run through the shipped syntax highlighter",
  );
  assert.match(
    syntaxHighlighter?.run ?? "",
    /tests\/tooling\/real-project-syntax-highlighting\.test\.ts/,
  );
  assert.equal(syntaxHighlighter?.env?.SYNTAX_HIGHLIGHTER_ORACLE_TIMEOUT_MS, "600000");
  assert.match(syntaxHighlighter?.run ?? "", /--test-concurrency=1/);
  assert.match(
    syntaxHighlighter?.run ?? "",
    /test -s "\$FIXTURE_REPORT_DIR\/syntax-highlighter-divergence\.json"/,
  );
  assert.match(
    syntaxHighlighter?.run ?? "",
    /test -s "\$FIXTURE_REPORT_DIR\/syntax-highlighter-divergence\.md"/,
  );
  assert.ok(glyphPropertiesIndex < divergenceIndex);
  assert.equal(glyphProperties?.env?.VIZE_TEST_BIN, "target/ci/vize");
  assert.match(glyphProperties?.run ?? "", /--test-concurrency=1/);
  for (const property of ["idempotence", "parse-preservation", "lint-agreement"]) {
    assert.match(
      glyphProperties?.run ?? "",
      new RegExp(`tests/tooling/glyph-corpus-${property}\\.test\\.ts`),
    );
  }
  assert.ok(runIndex < divergenceIndex && divergenceIndex < summaryIndex);
  assert.match(divergence?.run ?? "", /tools\/fixtures\/typecheck-divergence-report\.mjs/);
  assert.match(divergence?.run ?? "", /--report-dir "\$FIXTURE_REPORT_DIR"/);
  assert.match(divergence?.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(divergence?.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(divergence?.run ?? "", /--budget-mode "\$BUDGET_MODE"/);
  assert.match(divergence?.run ?? "", /--vue-tsc-bin tests\/node_modules\/\.bin\/vue-tsc/);
  assert.equal(divergence?.env?.BUDGET_MODE, "enforce");
  assert.equal(summary?.if, "${{ always() }}");
  assert.match(summary?.run ?? "", /summary\.md/);
  assert.match(summary?.run ?? "", /lsp-lifecycle-summary\.json/);
  assert.match(summary?.run ?? "", /authoredFeatureProjectCount/);
  assert.match(summary?.run ?? "", /missingAuthoredFeatureProjectIds/);
  assert.match(summary?.run ?? "", /actualFileCount/);
  assert.match(summary?.run ?? "", /No LSP lifecycle report was produced/);
  assert.match(summary?.run ?? "", /syntax-highlighter-summary\.json/);
  assert.match(summary?.run ?? "", /failedProjectCount/);
  assert.match(
    summary?.run ?? "",
    /syntax_divergence="\$FIXTURE_REPORT_DIR\/syntax-highlighter-divergence\.md"/,
  );
  assert.match(summary?.run ?? "", /if \[\[ -s "\$syntax_divergence" \]\]/);
  assert.match(summary?.run ?? "", /cat "\$syntax_divergence" >> "\$GITHUB_STEP_SUMMARY"/);
  assert.match(summary?.run ?? "", /No syntax-highlighter divergence report was produced/);
  assert.match(summary?.run ?? "", /\*-typecheck-divergence\.md/);
  assert.match(summary?.run ?? "", /divergence_reports\[@\]/);
  assert.equal(upload?.if, "${{ always() }}");
  assert.match(upload?.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(upload?.with, {
    name: "real-project-matrix-${{ matrix.shard }}",
    path: "${{ env.FIXTURE_REPORT_DIR }}",
    "if-no-files-found": "error",
    "retention-days": 30,
  });
});

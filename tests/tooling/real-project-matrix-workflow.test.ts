import assert from "node:assert/strict";
import { test } from "node:test";

import { requiredRealProjectMatrixShardCount } from "../../legacy-tools/github/release-preflight-matrix-evidence.mjs";
import {
  findStep,
  readRealProjectMatrixWorkflow,
  realProjectMatrixSteps,
} from "./support/real-project-matrix-workflow.ts";

test("real-project workflow schedules every balanced fixture shard", () => {
  const workflow = readRealProjectMatrixWorkflow();
  const job = workflow.jobs?.["real-project-matrix"];
  const expectedShards = Array.from(
    { length: requiredRealProjectMatrixShardCount },
    (_, shard) => shard,
  );

  assert.ok(job);
  assert.deepEqual(workflow.permissions, { contents: "read", issues: "read" });
  assert.equal(workflow.on?.schedule?.[0]?.cron, "37 5 * * 0");
  const dispatch = workflow.on?.workflow_dispatch;
  assert.ok(dispatch, "Missing workflow_dispatch trigger");
  assert.deepEqual(Object.keys(dispatch.inputs ?? {}), [
    "core_tools_mode",
    "core_tools_timeout_ms",
    "typecheck_dependencies_mode",
    "lint_divergence_mode",
    "lsp_mode",
    "typecheck_divergence_mode",
    "davinci_dom_corpus_mode",
  ]);
  assert.deepEqual(dispatch.inputs?.core_tools_mode, {
    description: "Core tool surface handling",
    required: false,
    default: "enforce",
    type: "choice",
    options: ["enforce", "record-only"],
  });
  assert.deepEqual(dispatch.inputs?.core_tools_timeout_ms, {
    description: "Per-project core tool timeout in milliseconds",
    required: false,
    default: "2400000",
    type: "string",
  });
  assert.deepEqual(dispatch.inputs?.typecheck_dependencies_mode, {
    description: "Typecheck baseline dependency preparation handling",
    required: false,
    default: "enforce",
    type: "choice",
    options: ["enforce", "record-only"],
  });
  assert.deepEqual(dispatch.inputs?.lint_divergence_mode, {
    description: "Patina lint divergence gate handling",
    required: false,
    default: "enforce",
    type: "choice",
    options: ["enforce", "record-only"],
  });
  assert.deepEqual(dispatch.inputs?.lsp_mode, {
    description: "LSP lifecycle gate handling",
    required: false,
    default: "enforce",
    type: "choice",
    options: ["enforce", "record-only"],
  });
  assert.deepEqual(dispatch.inputs?.typecheck_divergence_mode, {
    description: "Typechecker baseline divergence gate handling",
    required: false,
    default: "enforce",
    type: "choice",
    options: ["enforce", "record-only"],
  });
  assert.deepEqual(dispatch.inputs?.davinci_dom_corpus_mode, {
    description: "Davinci S2 DOM corpus gate handling",
    required: false,
    default: "enforce",
    type: "choice",
    options: ["enforce", "record-only"],
  });
  assert.deepEqual(workflow.concurrency, {
    group: "real-project-matrix-${{ github.ref }}",
    "cancel-in-progress": true,
  });
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 120);
  assert.equal(
    job.name,
    `real projects (\${{ matrix.shard }}/${requiredRealProjectMatrixShardCount})`,
  );
  assert.equal(job.strategy?.["fail-fast"], false);
  assert.equal(job.strategy?.["max-parallel"], 6);
  assert.deepEqual(job.strategy?.matrix?.shard, expectedShards);
  assert.deepEqual(job.env, {
    FIXTURE_SHARD_COUNT: String(requiredRealProjectMatrixShardCount),
    FIXTURE_SHARD_INDEX: "${{ matrix.shard }}",
    FIXTURE_REPORT_DIR: "real-project-results/shard-${{ matrix.shard }}",
  });
});

test("real-project workflow hydrates only its shard and runs every core tool", () => {
  const steps = realProjectMatrixSteps();
  const nodeSetupIndex = steps.findIndex((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  const shims = findStep(steps, "Enable package manager shims");
  const shimsIndex = steps.indexOf(shims);
  const hydration = findStep(steps, "Select and hydrate fixture shard");
  const hydrationIndex = steps.indexOf(hydration);
  const dependency = findStep(steps, "Install pinned typecheck baseline dependencies");
  const dependencyIndex = steps.indexOf(dependency);
  const waiverAudit = findStep(steps, "Audit formatter waiver owners");
  const waiverAuditIndex = steps.indexOf(waiverAudit);
  const run = findStep(steps, "Exercise real projects with every core tool");
  const runIndex = steps.indexOf(run);
  const lsp = findStep(steps, "Check real-project LSP lifecycle");
  const lspIndex = steps.indexOf(lsp);
  const lintDivergence = findStep(steps, "Measure real-project lint divergence");
  const lintDivergenceIndex = steps.indexOf(lintDivergence);
  const syntaxHighlighter = findStep(steps, "Check real-project syntax highlighting");
  const syntaxHighlighterIndex = steps.indexOf(syntaxHighlighter);
  const glyphProperties = findStep(steps, "Check glyph formatter corpus properties");
  const glyphPropertiesIndex = steps.indexOf(glyphProperties);
  const divergence = findStep(steps, "Enforce typechecker baseline divergence");
  const divergenceIndex = steps.indexOf(divergence);

  assert.notEqual(nodeSetupIndex, -1);
  assert.notEqual(hydrationIndex, -1);
  assert.ok(
    nodeSetupIndex < shimsIndex && shimsIndex < hydrationIndex,
    "Node 24 and package manager shims must be active before loading matrix scripts",
  );
  assert.deepEqual(steps[nodeSetupIndex].with, {
    "node-version-file": "package.json",
    cache: true,
    "run-install": false,
  });
  assert.equal(shims.run, "corepack enable");
  assert.match(hydration.run ?? "", /--list-fixture-paths/);
  assert.match(hydration.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(hydration.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(hydration.run ?? "", /mkdir -p "\$FIXTURE_REPORT_DIR"/);
  assert.match(hydration.run ?? "", /selected-fixtures\.txt/);
  assert.match(hydration.run ?? "", /git submodule update --init --depth 1/);
  assert.doesNotMatch(hydration.run ?? "", /--recursive/);
  assert.match(hydration.run ?? "", /"\$\{fixture_paths\[@\]\}"/);
  assert.ok(
    hydrationIndex < dependencyIndex &&
      dependencyIndex < waiverAuditIndex &&
      waiverAuditIndex < runIndex,
  );
  assert.match(dependency.run ?? "", /tools\/commands\/fixtures\/typecheck-dependency-prepare\.rs/);
  assert.match(dependency.run ?? "", /--output-dir "\$FIXTURE_REPORT_DIR"/);
  assert.match(dependency.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(dependency.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(dependency.run ?? "", /--timeout-ms 600000/);
  assert.deepEqual(
    {
      id: dependency.id,
      if: dependency.if,
      continueOnError: dependency["continue-on-error"],
    },
    {
      id: "typecheck_dependencies",
      if: "${{ !cancelled() }}",
      continueOnError: true,
    },
  );
  assert.equal(waiverAudit.id, "waiver_audit");
  assert.equal(waiverAudit.if, "${{ !cancelled() }}");
  assert.equal(waiverAudit["continue-on-error"], true);
  assert.deepEqual(waiverAudit.env, { GITHUB_TOKEN: "${{ github.token }}" });
  assert.match(waiverAudit.run ?? "", /glyph-corpus-waiver-audit\.rs/);
  assert.match(waiverAudit.run ?? "", /glyph-waiver-issues\.json/);
  assert.match(run.run ?? "", /tools\/commands\/fixtures\/tool-matrix-report\.rs/);
  assert.match(run.run ?? "", /--vize-bin target\/ci\/vize/);
  assert.match(run.run ?? "", /--timeout-ms "\$CORE_TOOLS_TIMEOUT_MS"/);
  assert.match(run.run ?? "", /--output-dir "\$FIXTURE_REPORT_DIR"/);
  assert.equal(run.env?.CORE_TOOLS_TIMEOUT_MS, "${{ inputs.core_tools_timeout_ms || '2400000' }}");
  for (const [step, id] of [
    [run, "core_tools"],
    [lsp, "lsp"],
    [lintDivergence, "lint_divergence"],
    [syntaxHighlighter, "syntax_highlighter"],
    [glyphProperties, "glyph"],
    [divergence, "typecheck_divergence"],
  ] as const) {
    assert.equal(step.id, id);
    assert.equal(step.if, "${{ !cancelled() }}");
    assert.equal(step["continue-on-error"], true);
  }
  assert.ok(
    runIndex < lspIndex && lspIndex < syntaxHighlighterIndex,
    "the hydrated fixture corpus must run through the production LSP before syntax audit",
  );
  assert.deepEqual(lsp.env, {
    CORSA_PATH: "${{ github.workspace }}/node_modules/@typescript/typescript-linux-x64/lib/tsc",
    REAL_PROJECT_LSP_TIMEOUT_MS: "600000",
    VIZE_LSP_BIN: "target/ci/vize",
  });
  assert.match(lsp.run ?? "", /if \[ ! -x "\$CORSA_PATH" \]; then/);
  assert.match(lsp.run ?? "", /::error title=Missing typecheck runtime::/);
  assert.match(lsp.run ?? "", /tests\/tooling\/real-project-lsp\.test\.ts/);
  assert.match(lsp.run ?? "", /--test-concurrency=1/);
  assert.match(lsp.run ?? "", /test -s "\$FIXTURE_REPORT_DIR\/lsp-lifecycle-summary\.json"/);
  assert.ok(
    lspIndex < lintDivergenceIndex && lintDivergenceIndex < syntaxHighlighterIndex,
    "the hydrated fixture corpus must be measured against the lint baseline",
  );
  for (const pattern of [
    /tools\/commands\/fixtures\/lint-divergence-report\.rs/,
    /--shard-index "\$FIXTURE_SHARD_INDEX"/,
    /--shard-count "\$FIXTURE_SHARD_COUNT"/,
    /--measure-coverage-gap/,
    /--budget-mode "\$LINT_DIVERGENCE_MODE"/,
    /--vize-bin target\/ci\/vize/,
    /--timeout-ms 600000/,
    /--output-dir "\$FIXTURE_REPORT_DIR"/,
    /test -s "\$FIXTURE_REPORT_DIR\/lint-divergence-summary\.json"/,
  ]) {
    assert.match(lintDivergence.run ?? "", pattern);
  }
  assert.equal(
    lintDivergence.env?.LINT_DIVERGENCE_MODE,
    "${{ inputs.lint_divergence_mode || 'enforce' }}",
  );
  assert.ok(
    syntaxHighlighterIndex < glyphPropertiesIndex,
    "the hydrated fixture corpus must run through the shipped syntax highlighter",
  );
  for (const pattern of [
    /tests\/tooling\/real-project-syntax-highlighting\.test\.ts/,
    /--test-concurrency=1/,
    /test -s "\$FIXTURE_REPORT_DIR\/syntax-highlighter-divergence\.json"/,
    /test -s "\$FIXTURE_REPORT_DIR\/syntax-highlighter-divergence\.md"/,
  ]) {
    assert.match(syntaxHighlighter.run ?? "", pattern);
  }
  assert.equal(syntaxHighlighter.env?.SYNTAX_HIGHLIGHTER_ORACLE_TIMEOUT_MS, "600000");
  assert.ok(glyphPropertiesIndex < divergenceIndex);
  assert.equal(glyphProperties.env?.VIZE_TEST_BIN, "target/ci/vize");
  assert.match(glyphProperties.run ?? "", /--test-concurrency=1/);
  for (const property of ["idempotence", "parse-preservation", "lint-agreement"]) {
    assert.match(
      glyphProperties.run ?? "",
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
    assert.match(glyphProperties.run ?? "", new RegExp(`tests/tooling/${oracle}\\.test\\.ts`));
  }
  assert.match(glyphProperties.run ?? "", /glyph-\$property\.json/);
  // A missing or empty Pug oracle artifact must fail the job: assert the guard,
  // not just the filename, so an inverted test or a dropped exit code is caught.
  assert.match(
    glyphProperties.run ?? "",
    /\[\[ ! -s "\$FIXTURE_REPORT_DIR\/glyph-pug-semantics\.json" \]\][\s\S]*?glyph_exit_code=1/,
  );
  assert.ok(runIndex < divergenceIndex);
  assert.match(divergence.run ?? "", /tools\/commands\/fixtures\/typecheck-divergence-report\.rs/);
  assert.match(divergence.run ?? "", /--report-dir "\$FIXTURE_REPORT_DIR"/);
  assert.match(divergence.run ?? "", /--shard-index "\$FIXTURE_SHARD_INDEX"/);
  assert.match(divergence.run ?? "", /--shard-count "\$FIXTURE_SHARD_COUNT"/);
  assert.match(divergence.run ?? "", /--budget-mode "\$BUDGET_MODE"/);
  assert.match(divergence.run ?? "", /--vue-tsc-bin tests\/node_modules\/\.bin\/vue-tsc/);
  assert.equal(divergence["timeout-minutes"], 100);
  assert.equal(divergence.env?.BUDGET_MODE, "${{ inputs.typecheck_divergence_mode || 'enforce' }}");

  const surfaceVerdict = findStep(steps, "Enforce all real-project surface verdicts");
  assert.match(surfaceVerdict.run ?? "", /real-project-surface-verdict\.rs/);
  assert.match(surfaceVerdict.run ?? "", /--from-workflow-env/);
  assert.equal(
    surfaceVerdict.env?.TYPECHECK_DEPENDENCIES_MODE,
    "${{ inputs.typecheck_dependencies_mode || 'enforce' }}",
  );
  assert.equal(
    surfaceVerdict.env?.LINT_DIVERGENCE_MODE,
    "${{ inputs.lint_divergence_mode || 'enforce' }}",
  );
  assert.equal(
    surfaceVerdict.env?.TYPECHECK_DIVERGENCE_MODE,
    "${{ inputs.typecheck_divergence_mode || 'enforce' }}",
  );
  assert.equal(
    surfaceVerdict.env?.VIZE_LINT_DIVERGENCE_OUTCOME,
    "${{ steps.lint_divergence.outcome }}",
  );
});

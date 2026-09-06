import assert from "node:assert/strict";
import { test } from "node:test";

import {
  findStep,
  readShardSummaryScript,
  realProjectMatrixSteps,
  shardSummaryCommandPath,
} from "./support/real-project-matrix-workflow.ts";

test("real-project workflow gates every measured surface on one verdict", () => {
  const steps = realProjectMatrixSteps();
  const divergenceIndex = steps.indexOf(findStep(steps, "Enforce typechecker baseline divergence"));
  const divergence = steps[divergenceIndex];
  const verdict = findStep(steps, "Enforce all real-project surface verdicts");
  const verdictIndex = steps.indexOf(verdict);
  const summaryIndex = steps.indexOf(findStep(steps, "Publish shard summary"));

  assert.ok(divergenceIndex < verdictIndex && verdictIndex < summaryIndex);
  assert.equal(divergence.env.VIZE_TYPECHECK_DIVERGENCE_PROGRESS, "1");
  assert.equal(verdict.if, "${{ always() }}");
  assert.equal(verdict.shell, "bash");
  assert.deepEqual(verdict.env, {
    CORE_TOOLS_MODE: "${{ inputs.core_tools_mode || 'enforce' }}",
    TYPECHECK_DEPENDENCIES_MODE: "${{ inputs.typecheck_dependencies_mode || 'enforce' }}",
    LINT_DIVERGENCE_MODE: "${{ inputs.lint_divergence_mode || 'enforce' }}",
    LSP_MODE: "${{ inputs.lsp_mode || 'enforce' }}",
    TYPECHECK_DIVERGENCE_MODE: "${{ inputs.typecheck_divergence_mode || 'enforce' }}",
    VIZE_WAIVER_AUDIT_OUTCOME: "${{ steps.waiver_audit.outcome }}",
    VIZE_TYPECHECK_DEPENDENCIES_OUTCOME: "${{ steps.typecheck_dependencies.outcome }}",
    VIZE_CORE_TOOLS_OUTCOME: "${{ steps.core_tools.outcome }}",
    VIZE_LSP_OUTCOME: "${{ steps.lsp.outcome }}",
    VIZE_LINT_DIVERGENCE_OUTCOME: "${{ steps.lint_divergence.outcome }}",
    VIZE_SYNTAX_HIGHLIGHTER_OUTCOME: "${{ steps.syntax_highlighter.outcome }}",
    VIZE_GLYPH_OUTCOME: "${{ steps.glyph.outcome }}",
    VIZE_TYPECHECK_DIVERGENCE_OUTCOME: "${{ steps.typecheck_divergence.outcome }}",
  });
  for (const pattern of [
    /real-project-surface-verdict\.rs/,
    /--from-workflow-env/,
    /--output "\$FIXTURE_REPORT_DIR\/surface-verdict\.json"/,
  ]) {
    assert.match(verdict.run ?? "", pattern);
  }
});

test("real-project workflow publishes and uploads the shard evidence it produced", () => {
  const steps = realProjectMatrixSteps();
  const summary = findStep(steps, "Publish shard summary");
  const upload = findStep(steps, "Upload shard report");

  assert.ok(steps.indexOf(summary) < steps.indexOf(upload));
  assert.equal(summary.if, "${{ always() }}");
  assert.equal(summary.shell, "bash");
  assert.equal(summary.run, `rust-script ${shardSummaryCommandPath}`);
  assert.equal(upload.if, "${{ always() }}");
  assert.match(upload.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(upload.with, {
    name: "real-project-matrix-${{ matrix.shard }}",
    path: "${{ env.FIXTURE_REPORT_DIR }}",
    "if-no-files-found": "error",
    "retention-days": 30,
  });
});

test("shard summary script records every surface, present or missing", () => {
  const script = readShardSummaryScript();

  for (const pattern of [
    /summary\.md/,
    /lsp-lifecycle-summary\.json/,
    /authoredFeatureProjectCount/,
    /authoredAnchorCount/,
    /missingAuthoredFeatureProjectIds/,
    /actualFileCount/,
    /No LSP lifecycle report was produced/,
    /syntax-highlighter-summary\.json/,
    /failedProjectCount/,
    /lint-divergence-summary\.json/,
    /patinaOnlyRuleFindingCount/,
    /-lint-divergence\.md/,
    /No lint divergence report was produced/,
    /syntax-highlighter-divergence\.md/,
    /append_file_or_line/,
    /No syntax-highlighter divergence report was produced/,
    /-typecheck-divergence\.md/,
    /append_unique_typecheck_divergence/,
    /glyph-waiver-issues\.json/,
    /surface-verdict\.json/,
    /dehydrate_selected_fixture_shard/,
    /selected-fixtures\.txt/,
    /"submodule", "deinit", "--force", "--"/,
  ]) {
    assert.match(script, pattern);
  }
  assert.doesNotMatch(script, /\bjq\b/);
  assert.match(script, /serde_json/);
});

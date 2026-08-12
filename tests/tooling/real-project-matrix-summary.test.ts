import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  findStep,
  realProjectMatrixSteps,
  shardSummaryScriptPath,
} from "./support/real-project-matrix-workflow.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/**
 * Runs the publish step against a throwaway report directory and returns the
 * step summary it rendered, so the assertions read the emitted metrics rather
 * than the shell source that produced them.
 */
function renderShardSummary(artifacts: Record<string, string>): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-shard-summary-"));
  try {
    for (const [name, contents] of Object.entries(artifacts)) {
      fs.writeFileSync(path.join(dir, name), contents);
    }
    const stepSummary = path.join(dir, "step-summary.md");
    fs.writeFileSync(stepSummary, "");
    const run = spawnSync("bash", [shardSummaryScriptPath], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, FIXTURE_REPORT_DIR: dir, GITHUB_STEP_SUMMARY: stepSummary },
    });
    assert.equal(run.status, 0, `shard summary script failed: ${run.stderr}`);
    return fs.readFileSync(stepSummary, "utf8");
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

test("real-project workflow gates every measured surface on one verdict", () => {
  const steps = realProjectMatrixSteps();
  const divergenceIndex = steps.indexOf(findStep(steps, "Enforce typechecker baseline divergence"));
  const verdict = findStep(steps, "Enforce all real-project surface verdicts");
  const verdictIndex = steps.indexOf(verdict);
  const summaryIndex = steps.indexOf(findStep(steps, "Publish shard summary"));

  assert.ok(divergenceIndex < verdictIndex && verdictIndex < summaryIndex);
  assert.equal(verdict.if, "${{ always() }}");
  assert.equal(verdict.shell, "bash");
  assert.deepEqual(verdict.env, {
    BUDGET_MODE: "${{ inputs.budget_mode || 'enforce' }}",
    CORE_TOOLS_MODE: "${{ inputs.core_tools_mode || 'enforce' }}",
    LSP_MODE: "${{ inputs.lsp_mode || 'enforce' }}",
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
    /real-project-surface-verdict\.mjs/,
    /surface-verdict\.json/,
    /core_tools_verdict="\$VIZE_CORE_TOOLS_OUTCOME"/,
    /\[\[ "\$CORE_TOOLS_MODE" == "record-only"/,
    /--surface "core-tools=\$core_tools_verdict"/,
    /lsp_verdict="\$VIZE_LSP_OUTCOME"/,
    /\[\[ "\$LSP_MODE" == "record-only"/,
    /--surface "lsp=\$lsp_verdict"/,
    /typecheck_divergence_verdict="\$VIZE_TYPECHECK_DIVERGENCE_OUTCOME"/,
    /\[\[ "\$BUDGET_MODE" == "record-only"/,
    /--surface "typecheck-divergence=\$typecheck_divergence_verdict"/,
  ]) {
    assert.match(verdict.run ?? "", pattern);
  }
  for (const [surface, variable] of [
    ["waiver-audit", "VIZE_WAIVER_AUDIT_OUTCOME"],
    ["typecheck-dependencies", "VIZE_TYPECHECK_DEPENDENCIES_OUTCOME"],
    ["lint-divergence", "VIZE_LINT_DIVERGENCE_OUTCOME"],
    ["syntax-highlighter", "VIZE_SYNTAX_HIGHLIGHTER_OUTCOME"],
    ["glyph", "VIZE_GLYPH_OUTCOME"],
  ]) {
    assert.match(verdict.run ?? "", new RegExp(`--surface "${surface}=\\$${variable}"`));
  }
});

test("real-project workflow publishes and uploads the shard evidence it produced", () => {
  const steps = realProjectMatrixSteps();
  const summary = findStep(steps, "Publish shard summary");
  const upload = findStep(steps, "Upload shard report");

  assert.ok(steps.indexOf(summary) < steps.indexOf(upload));
  assert.equal(summary.if, "${{ always() }}");
  assert.equal(summary.shell, "bash");
  assert.equal(summary.run, `bash ${shardSummaryScriptPath}`);
  assert.equal(upload.if, "${{ always() }}");
  assert.match(upload.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(upload.with, {
    name: "real-project-matrix-${{ matrix.shard }}",
    path: "${{ env.FIXTURE_REPORT_DIR }}",
    "if-no-files-found": "error",
    "retention-days": 30,
  });
});

test("shard summary script reads every artifact and reports its metrics", () => {
  const summary = renderShardSummary({
    "summary.md": "# fixture tool report\n",
    "lsp-lifecycle-summary.json": JSON.stringify({
      summary: {
        projectCount: 3,
        actualFileCount: 41,
        authoredFeatureProjectCount: 2,
        vueFileCount: 27,
        failedProjectCount: 1,
        missingAuthoredFeatureProjectIds: ["nuxt-app", "vitepress"],
      },
    }),
    "syntax-highlighter-summary.json": JSON.stringify({
      summary: { projectCount: 4, fileCount: 52, lineCount: 900, failedProjectCount: 0 },
    }),
    "lint-divergence-summary.json": JSON.stringify({
      projectCount: 5,
      totals: {
        sharedCount: 12,
        falsePositiveCount: 3,
        falseNegativeCount: 4,
        patinaOnlyRuleFindingCount: 7,
      },
    }),
    "nuxt-app-lint-divergence.md": "## lint divergence detail\n",
    "syntax-highlighter-divergence.md": "## syntax divergence detail\n",
    "nuxt-app-typecheck-divergence.md": "## typecheck divergence detail\n",
    "glyph-waiver-issues.json": JSON.stringify({ waiverCount: 2, issues: [{ number: 1 }] }),
    "surface-verdict.json": JSON.stringify({ status: "failed", failedSurfaceNames: ["lsp"] }),
  });

  assert.match(summary, /# fixture tool report/);
  assert.match(
    summary,
    /LSP lifecycle: 3 project\(s\), 41 actual file\(s\), 2 authored feature oracle\(s\), 27 Vue file\(s\), 1 failed project\(s\); missing authored oracles: nuxt-app, vitepress/,
  );
  assert.match(
    summary,
    /Syntax highlighter: 4 project\(s\), 52 file\(s\), 900 line\(s\), 0 failed project\(s\)/,
  );
  assert.match(
    summary,
    /Lint divergence: 5 project\(s\), 12 shared, 3 false positive\(s\), 4 false negative\(s\), 7 patina-only finding\(s\)/,
  );
  assert.match(summary, /## lint divergence detail/);
  assert.match(summary, /## syntax divergence detail/);
  assert.match(summary, /## typecheck divergence detail/);
  assert.match(summary, /Formatter waivers: 2 precise waiver\(s\), 1 open owner Issue\(s\)/);
  assert.match(summary, /Surface verdict: failed; failed: lsp/);
});

test("shard summary script records surfaces that produced no artifact", () => {
  const summary = renderShardSummary({});

  for (const missing of [
    "No fixture tool report was produced.",
    "No LSP lifecycle report was produced.",
    "No syntax-highlighter report was produced.",
    "No lint divergence report was produced.",
    "No syntax-highlighter divergence report was produced.",
    "No unique typecheck divergence report was produced.",
    "No formatter waiver owner report was produced.",
    "No real-project surface verdict was produced.",
  ]) {
    assert.ok(summary.includes(missing), `missing surface line: ${missing}`);
  }
});

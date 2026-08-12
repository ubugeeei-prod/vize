#!/usr/bin/env bash
set -euo pipefail

# Renders the shard's step summary from whatever evidence the matrix job
# produced. Every surface is optional on purpose: a failed or skipped step
# still has to leave a legible trace in the job summary.

if [[ -f "$FIXTURE_REPORT_DIR/summary.md" ]]; then
  cat "$FIXTURE_REPORT_DIR/summary.md" >> "$GITHUB_STEP_SUMMARY"
else
  echo "No fixture tool report was produced." >> "$GITHUB_STEP_SUMMARY"
fi
lsp_report="$FIXTURE_REPORT_DIR/lsp-lifecycle-summary.json"
if [[ -s "$lsp_report" ]]; then
  jq -r '"LSP lifecycle: \(.summary.projectCount) project(s), \(.summary.actualFileCount) actual file(s), \(.summary.authoredFeatureProjectCount) authored feature oracle(s), \(.summary.vueFileCount) Vue file(s), \(.summary.failedProjectCount) failed project(s); missing authored oracles: \(.summary.missingAuthoredFeatureProjectIds | join(", "))"' \
    "$lsp_report" >> "$GITHUB_STEP_SUMMARY"
else
  echo "No LSP lifecycle report was produced." >> "$GITHUB_STEP_SUMMARY"
fi
syntax_report="$FIXTURE_REPORT_DIR/syntax-highlighter-summary.json"
if [[ -f "$syntax_report" ]]; then
  jq -r '"Syntax highlighter: \(.summary.projectCount) project(s), \(.summary.fileCount) file(s), \(.summary.lineCount) line(s), \(.summary.failedProjectCount) failed project(s)"' \
    "$syntax_report" >> "$GITHUB_STEP_SUMMARY"
else
  echo "No syntax-highlighter report was produced." >> "$GITHUB_STEP_SUMMARY"
fi
lint_divergence="$FIXTURE_REPORT_DIR/lint-divergence-summary.json"
if [[ -s "$lint_divergence" ]]; then
  jq -r '"Lint divergence: \(.projectCount) project(s), \(.totals.sharedCount) shared, \(.totals.falsePositiveCount) false positive(s), \(.totals.falseNegativeCount) false negative(s), \(.totals.patinaOnlyRuleFindingCount) patina-only finding(s)"' \
    "$lint_divergence" >> "$GITHUB_STEP_SUMMARY"
  for report in "$FIXTURE_REPORT_DIR"/*-lint-divergence.md; do
    [[ -s "$report" ]] && cat "$report" >> "$GITHUB_STEP_SUMMARY"
  done
else
  echo "No lint divergence report was produced." >> "$GITHUB_STEP_SUMMARY"
fi
syntax_divergence="$FIXTURE_REPORT_DIR/syntax-highlighter-divergence.md"
if [[ -s "$syntax_divergence" ]]; then
  cat "$syntax_divergence" >> "$GITHUB_STEP_SUMMARY"
else
  echo "No syntax-highlighter divergence report was produced." >> "$GITHUB_STEP_SUMMARY"
fi
mapfile -t divergence_reports < <(
  find "$FIXTURE_REPORT_DIR" -maxdepth 1 -type f \
    -name '*-typecheck-divergence.md' -print | sort
)
if (( ${#divergence_reports[@]} == 1 )); then
  cat "${divergence_reports[0]}" >> "$GITHUB_STEP_SUMMARY"
else
  echo "No unique typecheck divergence report was produced." >> "$GITHUB_STEP_SUMMARY"
fi
waiver_report="$FIXTURE_REPORT_DIR/glyph-waiver-issues.json"
if [[ -s "$waiver_report" ]]; then
  jq -r '"Formatter waivers: \(.waiverCount) precise waiver(s), \(.issues | length) open owner Issue(s)"' \
    "$waiver_report" >> "$GITHUB_STEP_SUMMARY"
else
  echo "No formatter waiver owner report was produced." >> "$GITHUB_STEP_SUMMARY"
fi
surface_verdict="$FIXTURE_REPORT_DIR/surface-verdict.json"
if [[ -s "$surface_verdict" ]]; then
  jq -r '"Surface verdict: \(.status); failed: \(.failedSurfaceNames | join(", "))"' \
    "$surface_verdict" >> "$GITHUB_STEP_SUMMARY"
else
  echo "No real-project surface verdict was produced." >> "$GITHUB_STEP_SUMMARY"
fi

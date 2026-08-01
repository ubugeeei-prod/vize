/**
 * The human-readable half of a divergence run, split out of
 * `typecheck-divergence-report.mjs` so the report script stays inside the
 * per-file line budget.
 *
 * This is what a reviewer reads in the shard's step summary, so every field the
 * verdict depends on is printed: the diagnostic counts, both exclusion lenses,
 * the Vue-file coverage, and — since #3513 — how many configuration diagnostics
 * the baseline emitted. A shard that measured nothing has to say so here, not
 * only in the JSON nobody opens.
 */
export function renderMarkdown(artifact) {
  const summary = artifact.divergence.summary;
  const coverage = artifact.baseline.coverage;
  return [
    `## ${artifact.project} typecheck divergence`,
    "",
    `Commit: ${artifact.evidence.commitSha}`,
    `Vize diagnostics: ${summary.vizeDiagnosticCount}`,
    `vue-tsc diagnostics: ${summary.baselineDiagnosticCount}`,
    `Shared: ${summary.sharedCount}`,
    `Message mismatches: ${summary.messageMismatchCount}`,
    `Documented differences: ${summary.documentedDifferenceCount}`,
    `False positives: ${summary.falsePositiveCount} (${summary.falsePositiveRatio})`,
    `False negatives: ${summary.falseNegativeCount} (${summary.falseNegativeRatio})`,
    `Vize excluded non-Vue: ${summary.vizeExcludedNonVueCount}`,
    `vue-tsc excluded non-Vue: ${summary.baselineExcludedNonVueCount}`,
    `vue-tsc excluded project-level: ${summary.baselineExcludedProjectCount}`,
    `vue-tsc excluded external: ${summary.baselineExcludedExternalCount}`,
    `vue-tsc configuration errors: ${artifact.baseline.configuration.errorCount}`,
    `Vize Vue files: ${coverage.vizeVueFileCount}`,
    `vue-tsc Vue files: ${coverage.baselineVueFileCount}`,
    `Shared Vue files: ${coverage.sharedVueFileCount}`,
    `Missing Vue files: ${coverage.missingVueFiles.length}`,
    `Unexpected Vue files: ${coverage.unexpectedVueFiles.length}`,
    `Ignored dependency Vue files: ${coverage.ignoredDependencyVueFileCount}`,
    `Budget verdict: ${describeVerdict(artifact.budget)}`,
    `Budget passed: ${artifact.budget.passed}`,
    `Digest: ${artifact.divergence.sha256}`,
    "",
  ].join("\n");
}

function describeVerdict(budget) {
  return budget.unusableReason == null ? budget.verdict : `unusable (${budget.unusableReason})`;
}

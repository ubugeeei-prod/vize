/**
 * The human-readable half of a lint divergence run, split out of
 * `lint-divergence-report.mjs` so the report script stays inside the per-file
 * line budget.
 *
 * This is what a reviewer reads in the shard's step summary, so it has to carry
 * enough to tell a real divergence from a measurement artifact: the comparable
 * rule surface (a run that compared nothing must say so here, not only in the
 * JSON), the parse-error count (files the reference parser could not read are
 * excluded, and a corpus the baseline mostly failed to parse is not evidence),
 * and the per-rule breakdown, because an aggregate ratio never says which rule
 * to fix.
 */
export function renderMarkdown(artifact) {
  const summary = artifact.divergence.summary;
  const lines = [
    `## ${artifact.project} lint divergence`,
    "",
    `Commit: ${artifact.evidence.commitSha}`,
    `Revision: ${artifact.revision}`,
    `Preset: ${artifact.preset}`,
    `Baseline: ${artifact.baseline.package} ${artifact.baseline.version}`,
    `Compared rules: ${artifact.baseline.comparedRuleCount} of ${artifact.baseline.mappedRuleCount} mapped`,
    `Files: ${artifact.files.comparedCount}`,
    `Baseline messages dropped as foreign-rule directives: ${artifact.baseline.droppedConfigMessageCount}`,
    "",
    `Patina findings: ${summary.patinaFindingCount}`,
    `Baseline findings: ${summary.baselineFindingCount}`,
    `Comparable baseline findings: ${summary.comparableBaselineCount}`,
    `Shared: ${summary.sharedCount}`,
    `Message differences: ${summary.messageDifferenceCount}`,
    `Documented divergences: ${summary.documentedDivergenceCount}`,
    `Rule location divergences: ${summary.ruleLocationDivergenceCount ?? 0}`,
    `False positives: ${summary.falsePositiveCount} (${summary.falsePositiveRatio})`,
    `False negatives: ${summary.falseNegativeCount} (${summary.falseNegativeRatio})`,
    `Unimplemented upstream findings: ${summary.unimplementedCount}`,
    `Intentional divergences: ${summary.intentionalDivergenceCount}`,
    `Patina-only rule findings: ${summary.patinaOnlyRuleFindingCount}`,
    `Baseline parse errors: ${summary.baselineParseErrorCount}`,
    `Baseline invalid ranges: ${summary.baselineInvalidRangeCount}`,
    `Budget verdict: ${artifact.budget?.verdict ?? "not-evaluated"}`,
    `Budget passed: ${artifact.budget?.passed ?? false}`,
    "",
  ];
  if (artifact.baseline.comparedRuleCount === 0) {
    lines.push("> No mapped rule was comparable under this preset: nothing was measured.", "");
  }
  lines.push(...ruleTable("False positives", artifact.divergence.falsePositives));
  lines.push(...ruleTable("False negatives", artifact.divergence.falseNegatives));
  lines.push(
    ...ruleTable("Rule location divergences", artifact.divergence.ruleLocationDivergences ?? []),
  );
  lines.push(...ruleTable("Unimplemented upstream rules", artifact.divergence.unimplemented));
  return `${lines.join("\n")}\n`;
}

function ruleTable(title, findings) {
  if (findings.length === 0) return [`### ${title}: none`, ""];
  const counts = new Map();
  for (const finding of findings) {
    const key = finding.upstreamRuleId ?? finding.ruleId;
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  const rows = [...counts].sort(
    (left, right) => right[1] - left[1] || left[0].localeCompare(right[0]),
  );
  return [
    `### ${title}: ${findings.length}`,
    "",
    "| Rule | Findings |",
    "| --- | ---: |",
    ...rows.map(([rule, count]) => `| \`${rule}\` | ${count} |`),
    "",
  ];
}

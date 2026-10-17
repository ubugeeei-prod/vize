import path from "node:path";
import {
  byteOrder,
  get,
  list,
  number,
  object,
  text,
  unsigned,
  writeJson,
  relativePath,
  type Json,
  type JsonObject,
} from "./value.ts";

function unusableReason(artifact: Json): string | null {
  const summary = get(artifact, "divergence", "summary");
  const reference = number(get(summary, "referenceCoverageIntegrityErrorCount"));
  if (reference > 0)
    return `expected exactly one native finding for ${reference} reviewed reference-coverage case(s)`;
  const compared = unsigned(get(artifact, "files", "comparedCount")),
    expected = unsigned(get(artifact, "files", "expectedCount"));
  if (expected === 0 && compared !== 0)
    return "the project expected zero Vue files but selected a nonempty corpus";
  if (compared === 0 && expected !== 0) return "the project selected no Vue files";
  if (unsigned(get(artifact, "baseline", "comparedRuleCount")) === 0)
    return "no mapped eslint-plugin-vue rule was comparable under the selected preset";
  const parse = number(get(summary, "baselineParseErrorCount"));
  if (parse > 0) return `eslint-plugin-vue could not parse ${parse} compared file(s)`;
  const invalid = number(get(summary, "baselineInvalidRangeCount"));
  return invalid > 0
    ? `eslint-plugin-vue reported ${invalid} finding(s) with invalid source ranges`
    : null;
}
export function evaluateBudget(artifact: Json): JsonObject {
  const summary = get(artifact, "divergence", "summary"),
    reason = unusableReason(artifact);
  const falsePositivePassed = number(get(summary, "falsePositiveCount")) === 0,
    falseNegativePassed = number(get(summary, "falseNegativeCount")) === 0;
  const verdict =
    reason !== null
      ? "unusable"
      : falsePositivePassed && falseNegativePassed
        ? "passed"
        : "breached";
  return {
    maxFalsePositiveCount: 0,
    maxFalseNegativeCount: 0,
    falsePositivePassed,
    falseNegativePassed,
    unusableReason: reason,
    verdict,
    passed: verdict === "passed",
  };
}
function describeFailure(artifact: Json): string {
  const project = text(get(artifact, "project"), "?"),
    budget = get(artifact, "budget"),
    summary = get(artifact, "divergence", "summary");
  if (get(budget, "verdict") === "unusable")
    return `Lint divergence baseline is unusable for ${project}: ${text(get(budget, "unusableReason"), "unknown")}`;
  const breaches: string[] = [];
  if (get(budget, "falsePositivePassed") !== true)
    breaches.push(
      `${JSON.stringify(get(summary, "falsePositiveCount"))} false positives exceed maxFalsePositiveCount 0`,
    );
  if (get(budget, "falseNegativePassed") !== true)
    breaches.push(
      `${JSON.stringify(get(summary, "falseNegativeCount"))} false negatives exceed maxFalseNegativeCount 0`,
    );
  return `Lint divergence budget breached for ${project}: ${breaches.join("; ")}`;
}
export function parseBudgetMode(value: string): string {
  if (value !== "enforce" && value !== "record-only")
    throw new Error("--budget-mode must be one of: enforce, record-only");
  return value;
}
export function assertBudgetsPassed(artifacts: Json[], mode: string): void {
  parseBudgetMode(mode);
  const warning = (detail: string) =>
    console.log(`::warning title=Lint divergence budget not enforced::${detail}`);
  if (!artifacts.length) {
    const detail = "Lint divergence budget has no measured projects";
    if (mode === "enforce") throw new Error(detail);
    warning(detail);
    return;
  }
  const failures = artifacts.filter((artifact) => get(artifact, "budget", "passed") !== true);
  if (!failures.length) return;
  const details = failures.map(describeFailure);
  if (mode === "enforce")
    throw new Error(
      `Lint divergence budget failed for ${failures.length} project(s):\n${details.join("\n")}`,
    );
  for (const detail of details) warning(detail);
}
export function summarizeBudgets(artifacts: Json[]): Json {
  const failed = artifacts.filter((artifact) => get(artifact, "budget", "passed") !== true),
    passed = artifacts.length > 0 && failed.length === 0;
  return {
    status: passed ? "success" : "failure",
    passed,
    projectCount: artifacts.length,
    passedCount: artifacts.length - failed.length,
    failedCount: failed.length,
    unusableCount: artifacts.filter((a) => get(a, "budget", "verdict") === "unusable").length,
    breachedCount: artifacts.filter((a) => get(a, "budget", "verdict") === "breached").length,
    failedProjects: failed.map((a) => get(a, "project")),
  };
}
export function writeIndex(
  root: string,
  outputDir: string,
  preset: string | null,
  evidence: Json,
  artifacts: Json[],
): void {
  const keys = [
    "patinaFindingCount",
    "baselineFindingCount",
    "comparableBaselineCount",
    "sharedCount",
    "messageDifferenceCount",
    "documentedDivergenceCount",
    "ruleLocationDivergenceCount",
    "falsePositiveCount",
    "falseNegativeCount",
    "unimplementedCount",
    "intentionalDivergenceCount",
    "patinaOnlyRuleFindingCount",
    "baselineParseErrorCount",
    "baselineInvalidRangeCount",
  ];
  const totals = Object.fromEntries(
    keys.map((key) => [
      key,
      artifacts.reduce<number>((sum, a) => sum + number(get(a, "divergence", "summary", key)), 0),
    ]),
  );
  const summary: Json = {
    schema: "vize.fixtureLintDivergenceIndex",
    version: 1,
    evidence,
    preset: preset ?? "all-mapped",
    projectCount: artifacts.length,
    budget: summarizeBudgets(artifacts),
    totals,
    projects: artifacts.map((a) => ({
      project: get(a, "project"),
      ...object(get(a, "divergence", "summary")),
    })),
  };
  const filename = path.join(outputDir, "lint-divergence-summary.json");
  writeJson(filename, summary);
  console.log(`Wrote ${relativePath(root, filename)}`);
}
function ruleTable(title: string, findings: Json): string[] {
  const records = list(findings);
  if (!records.length) return [`### ${title}: none`, ""];
  const counts = new Map<string, number>();
  for (const finding of records) {
    // Rust's get/or_else does not fall back on an explicitly null upstream id.
    const record = object(finding),
      value = record?.upstreamRuleId === undefined ? record?.ruleId : record.upstreamRuleId;
    const key = text(value, "-");
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  const rows = [...counts].sort(([a, n], [b, m]) => m - n || byteOrder(a, b));
  return [
    `### ${title}: ${records.length}`,
    "",
    "| Rule | Findings |",
    "| --- | ---: |",
    ...rows.map(([rule, count]) => `| \`${rule}\` | ${count} |`),
    "",
  ];
}
export function renderMarkdown(artifact: Json): string {
  const summary = get(artifact, "divergence", "summary"),
    baseline = get(artifact, "baseline");
  const count = (key: string) => JSON.stringify(get(summary, key));
  const ratio = (key: string) => {
    const value = get(summary, key);
    return typeof value === "number" && Number.isInteger(value)
      ? `${value}.0`
      : JSON.stringify(value);
  };
  const lines = [
    `## ${text(get(artifact, "project"), "-")} lint divergence`,
    "",
    `Commit: ${text(get(artifact, "evidence", "commitSha"), "unknown")}`,
    `Revision: ${text(get(artifact, "revision"), "unknown")}`,
    `Preset: ${text(get(artifact, "preset"), "unknown")}`,
    `Baseline: ${text(get(baseline, "package"), "unknown")} ${text(get(baseline, "version"), "unknown")}`,
    `Compared rules: ${number(get(baseline, "comparedRuleCount"))} of ${number(get(baseline, "mappedRuleCount"))} mapped`,
    `Files: ${number(get(artifact, "files", "comparedCount"))}`,
    `Baseline messages dropped as foreign-rule directives: ${number(get(baseline, "droppedConfigMessageCount"))}`,
    "",
    `Patina findings: ${count("patinaFindingCount")}`,
    `Baseline findings: ${count("baselineFindingCount")}`,
    `Comparable baseline findings: ${count("comparableBaselineCount")}`,
    `Shared: ${count("sharedCount")}`,
    `Message differences: ${count("messageDifferenceCount")}`,
    `Documented divergences: ${count("documentedDivergenceCount")}`,
    `Rule location divergences: ${count("ruleLocationDivergenceCount")}`,
    `False positives: ${count("falsePositiveCount")} (${ratio("falsePositiveRatio")})`,
    `False negatives: ${count("falseNegativeCount")} (${ratio("falseNegativeRatio")})`,
    `Unimplemented upstream findings: ${count("unimplementedCount")}`,
    `Intentional divergences: ${count("intentionalDivergenceCount")}`,
    `Patina-only rule findings: ${count("patinaOnlyRuleFindingCount")}`,
    `Patina findings outside reference coverage: ${count("patinaOutsideBaselineSurfaceCount")}`,
    `Reference coverage integrity errors: ${count("referenceCoverageIntegrityErrorCount")}`,
    `Baseline parse errors: ${count("baselineParseErrorCount")}`,
    `Baseline invalid ranges: ${count("baselineInvalidRangeCount")}`,
    `Budget verdict: ${text(get(artifact, "budget", "verdict"), "not-evaluated")}`,
    `Budget passed: ${get(artifact, "budget", "passed") === true}`,
    "",
  ];
  if (unsigned(get(baseline, "comparedRuleCount")) === 0)
    lines.push("> No mapped rule was comparable under this preset: nothing was measured.", "");
  for (const [title, key] of [
    ["False positives", "falsePositives"],
    ["False negatives", "falseNegatives"],
    ["Rule location divergences", "ruleLocationDivergences"],
    ["Unimplemented upstream rules", "unimplemented"],
    ["Patina findings outside reference coverage", "patinaOutsideBaselineSurface"],
  ])
    lines.push(...ruleTable(title, get(artifact, "divergence", key)));
  return lines.join("\n") + "\n";
}

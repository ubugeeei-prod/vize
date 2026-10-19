/**
 * Fail-closed verdicts for the Patina versus eslint-plugin-vue corpus scorecard.
 *
 * The rule map and ledger are evidence only if a release lane actually reads the
 * measured divergence and refuses unexplained drift. The budget is intentionally
 * strict: documented divergences are already removed by the comparator, so any
 * remaining false positive or false negative is an unreviewed parity gap.
 */
const budgetModes = ["enforce", "record-only"] as const;
export type BudgetMode = (typeof budgetModes)[number];

export interface LintBudget {
  readonly maxFalsePositiveCount: number;
  readonly maxFalseNegativeCount: number;
  readonly falsePositivePassed: boolean;
  readonly falseNegativePassed: boolean;
  readonly unusableReason: string | null;
  readonly verdict: "unusable" | "passed" | "breached";
  readonly passed: boolean;
}

export interface BudgetArtifact {
  readonly project: string;
  readonly files: { readonly expectedCount?: number; readonly comparedCount: number };
  readonly baseline: { readonly comparedRuleCount: number };
  readonly divergence: {
    readonly summary: {
      readonly falsePositiveCount: number;
      readonly falseNegativeCount: number;
      readonly baselineParseErrorCount: number;
      readonly baselineInvalidRangeCount: number;
    };
  };
  readonly budget?: LintBudget;
}

export function parseBudgetMode(value: unknown): BudgetMode {
  const mode = budgetModes.find((candidate) => candidate === value);
  if (mode === undefined) {
    throw new Error(`--budget-mode must be one of: ${budgetModes.join(", ")}`);
  }
  return mode;
}

export function evaluateBudget(artifact: BudgetArtifact): LintBudget {
  const summary = artifact.divergence.summary;
  const unusableReason = unusableLintReason(artifact);
  const falsePositivePassed = summary.falsePositiveCount === 0;
  const falseNegativePassed = summary.falseNegativeCount === 0;
  const verdict =
    unusableReason != null
      ? "unusable"
      : falsePositivePassed && falseNegativePassed
        ? "passed"
        : "breached";
  return {
    maxFalsePositiveCount: 0,
    maxFalseNegativeCount: 0,
    falsePositivePassed,
    falseNegativePassed,
    unusableReason,
    verdict,
    passed: verdict === "passed",
  };
}

export function attachBudget<T extends BudgetArtifact>(artifact: T): T & { budget: LintBudget } {
  return { ...artifact, budget: evaluateBudget(artifact) };
}

export function assertBudgetsPassed(
  artifacts: readonly BudgetArtifact[],
  budgetMode: unknown = "enforce",
): void {
  const mode = parseBudgetMode(budgetMode);
  if (artifacts.length === 0) {
    const detail = "Lint divergence budget has no measured projects";
    if (mode === "enforce") throw new Error(detail);
    process.stdout.write(`::warning title=Lint divergence budget not enforced::${detail}\n`);
    return;
  }
  const failures = artifacts.filter((artifact) => artifact.budget?.passed !== true);
  if (failures.length === 0) return;
  const details = failures.map(describeFailure);
  if (mode === "enforce") {
    throw new Error(
      `Lint divergence budget failed for ${failures.length} project(s):\n${details.join("\n")}`,
    );
  }
  for (const detail of details) {
    process.stdout.write(`::warning title=Lint divergence budget not enforced::${detail}\n`);
  }
}

export function summarizeBudgets(artifacts: readonly BudgetArtifact[]) {
  const failures = artifacts.filter((artifact) => artifact.budget?.passed !== true);
  const unusable = artifacts.filter((artifact) => artifact.budget?.verdict === "unusable");
  const breached = artifacts.filter((artifact) => artifact.budget?.verdict === "breached");
  return {
    status: artifacts.length > 0 && failures.length === 0 ? "success" : "failure",
    passed: artifacts.length > 0 && failures.length === 0,
    projectCount: artifacts.length,
    passedCount: artifacts.length - failures.length,
    failedCount: failures.length,
    unusableCount: unusable.length,
    breachedCount: breached.length,
    failedProjects: failures.map((artifact) => artifact.project),
  };
}

function unusableLintReason(artifact: BudgetArtifact): string | null {
  if (artifact.files.expectedCount === 0 && artifact.files.comparedCount !== 0) {
    return "the project expected zero Vue files but selected a nonempty corpus";
  }
  if (artifact.files.comparedCount === 0 && artifact.files.expectedCount !== 0) {
    return "the project selected no Vue files";
  }
  if (artifact.baseline.comparedRuleCount === 0) {
    return "no mapped eslint-plugin-vue rule was comparable under the selected preset";
  }
  const parseErrors = artifact.divergence.summary.baselineParseErrorCount;
  if (parseErrors > 0) {
    return `eslint-plugin-vue could not parse ${parseErrors} compared file(s)`;
  }
  const invalidRanges = artifact.divergence.summary.baselineInvalidRangeCount;
  if (invalidRanges > 0) {
    return `eslint-plugin-vue reported ${invalidRanges} finding(s) with invalid source ranges`;
  }
  return null;
}

function describeFailure(artifact: BudgetArtifact): string {
  const budget = artifact.budget;
  if (budget === undefined)
    return `Lint divergence budget was not evaluated for ${artifact.project}`;
  const summary = artifact.divergence.summary;
  if (budget.verdict === "unusable") {
    return `Lint divergence baseline is unusable for ${artifact.project}: ${budget.unusableReason}`;
  }
  const breaches: string[] = [];
  if (!budget.falsePositivePassed) {
    breaches.push(`${summary.falsePositiveCount} false positives exceed maxFalsePositiveCount 0`);
  }
  if (!budget.falseNegativePassed) {
    breaches.push(`${summary.falseNegativeCount} false negatives exceed maxFalseNegativeCount 0`);
  }
  return `Lint divergence budget breached for ${artifact.project}: ${breaches.join("; ")}`;
}

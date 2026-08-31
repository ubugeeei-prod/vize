/**
 * Fail-closed verdicts for the Patina versus eslint-plugin-vue corpus scorecard.
 *
 * The rule map and ledger are evidence only if a release lane actually reads the
 * measured divergence and refuses unexplained drift. The budget is intentionally
 * strict: documented divergences are already removed by the comparator, so any
 * remaining false positive or false negative is an unreviewed parity gap.
 */
const budgetModes = ["enforce", "record-only"];

export function parseBudgetMode(value) {
  if (!budgetModes.includes(value)) {
    throw new Error(`--budget-mode must be one of: ${budgetModes.join(", ")}`);
  }
  return value;
}

export function evaluateBudget(artifact) {
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

export function attachBudget(artifact) {
  return { ...artifact, budget: evaluateBudget(artifact) };
}

export function assertBudgetsPassed(artifacts, budgetMode = "enforce") {
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

export function summarizeBudgets(artifacts) {
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

function unusableLintReason(artifact) {
  if (artifact.files.comparedCount === 0) return "the project selected no Vue files";
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

function describeFailure(artifact) {
  const budget = artifact.budget;
  const summary = artifact.divergence.summary;
  if (budget.verdict === "unusable") {
    return `Lint divergence baseline is unusable for ${artifact.project}: ${budget.unusableReason}`;
  }
  const breaches = [];
  if (!budget.falsePositivePassed) {
    breaches.push(`${summary.falsePositiveCount} false positives exceed maxFalsePositiveCount 0`);
  }
  if (!budget.falseNegativePassed) {
    breaches.push(`${summary.falseNegativeCount} false negatives exceed maxFalseNegativeCount 0`);
  }
  return `Lint divergence budget breached for ${artifact.project}: ${breaches.join("; ")}`;
}

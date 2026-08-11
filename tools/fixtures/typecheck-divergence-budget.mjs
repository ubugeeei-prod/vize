/**
 * Verdict and enforcement for the vize-versus-vue-tsc false-positive /
 * false-negative budget, split out of `typecheck-divergence-report.mjs` so the
 * report script stays inside the per-file line budget.
 *
 * Two rules live here, and they answer two different failures the ledger had:
 *
 * 1. `evaluateBudget` returns three verdicts, not two (#3513, the #3222 parity
 *    ledger). The baseline is unusable when `vue-tsc` could not load the
 *    fixture's project configuration, when `vue-tsc --listFiles` proves the two
 *    tools checked different Vue corpora, or when two non-empty diagnostic
 *    streams have no mapped position in common.
 * 2. `assertBudgetPassed` enforces the same verdict on every entry path.
 */

/**
 * `enforce` fails the run on anything that is not a clean pass. It is the only
 * accepted mode: an old caller cannot recover the removed release escape hatch
 * by passing `record-only`, and a typo cannot silently disarm the gate.
 */
const budgetModes = ["enforce"];

export function validateExactParityPerformance(performance) {
  if (!Number.isSafeInteger(performance.hangTimeoutMs) || performance.hangTimeoutMs <= 0) {
    throw new Error("typecheckPerformance.hangTimeoutMs must be a positive safe integer");
  }
  ratio(performance.maxFalsePositiveRatio, "maxFalsePositiveRatio");
  ratio(performance.maxFalseNegativeRatio, "maxFalseNegativeRatio");
  if (performance.maxFalsePositiveRatio !== 0 || performance.maxFalseNegativeRatio !== 0) {
    throw new Error(
      "typecheckPerformance FP/FN ratios must both be 0; unexplained diagnostics are release-blocking",
    );
  }
}

export function parseBudgetMode(value) {
  if (!budgetModes.includes(value)) {
    throw new Error(`--budget-mode must be one of: ${budgetModes.join(", ")}`);
  }
  return value;
}

export function evaluateBudget(performance, summary, coverage, configuration) {
  const messageMismatchPassed = summary.messageMismatchCount === 0;
  const falsePositivePassed = summary.falsePositiveCount === 0;
  const falseNegativePassed = summary.falseNegativeCount === 0;
  // Configuration first: it is the *cause* a coverage or mapping failure would
  // only be a symptom of, and it is the one reason that survives a run where
  // both other checks look clean.
  const unusableReason =
    configuration.unusableReason ??
    coverage.unusableReason ??
    diagnosticMappingUnusableReason(summary);
  const verdict =
    unusableReason != null
      ? "unusable"
      : hasExactUnexplainedParity(summary)
        ? "passed"
        : "breached";
  return {
    maxFalsePositiveRatio: performance.maxFalsePositiveRatio,
    maxFalseNegativeRatio: performance.maxFalseNegativeRatio,
    messageMismatchPassed,
    falsePositivePassed,
    falseNegativePassed,
    unusableReason,
    verdict,
    passed: verdict === "passed",
  };
}

/**
 * The per-PR probes and the release corpus share this exact acceptance rule.
 * Documented differences have already been removed by the shared comparator;
 * anything left in these three buckets is unexplained and must fail.
 */
export function hasExactUnexplainedParity(summary) {
  return (
    summary.messageMismatchCount === 0 &&
    summary.falsePositiveCount === 0 &&
    summary.falseNegativeCount === 0
  );
}

/**
 * Exact file coverage is established separately from diagnostics: a clean
 * project can legitimately produce 0/0, while a one-sided diagnostic stream is
 * a real divergence when both tools checked the same files. The remaining
 * unusable diagnostic shape is two non-empty streams with no mapped position in
 * common, which signals a file or position mapping defect rather than a useful
 * 100% FP/FN score.
 */
function diagnosticMappingUnusableReason(summary) {
  const overlap =
    summary.sharedCount + summary.messageMismatchCount + summary.documentedDifferenceCount;
  if (overlap > 0) return null;
  if (summary.vizeDiagnosticCount === 0 || summary.baselineDiagnosticCount === 0) return null;
  return (
    `vize reported ${summary.vizeDiagnosticCount} and vue-tsc reported ` +
    `${summary.baselineDiagnosticCount} diagnostics with none in common`
  );
}

/**
 * The budget is a gate, not a note (#2971): `budget.passed` used to be computed,
 * embedded in the artifact and printed in the step summary while nothing read
 * it, so a matrix-wide breach reported `Budget passed: false` and the weekly job
 * still went green.
 *
 * This runs after both artifacts are written, so a breach is uploaded and
 * reviewable — every entry path fails with the evidence attached, not instead
 * of it.
 */
export function assertBudgetPassed(artifact, budgetMode = "enforce") {
  // Validated before the passed-verdict return, so an unrecognised mode is
  // rejected on every run rather than only on the runs that breach.
  parseBudgetMode(budgetMode);
  const budget = artifact.budget;
  if (budget.verdict === "passed") return;
  const detail = `${
    budget.verdict === "unusable"
      ? `Typecheck divergence baseline is unusable for ${artifact.project}`
      : `Typecheck divergence budget breached for ${artifact.project}`
  } — ${describeClassification(artifact)}: ${
    budget.verdict === "unusable" ? budget.unusableReason : describeBreaches(artifact).join("; ")
  }`;
  throw new Error(detail);
}

/**
 * Answer the only question a failing shard leaves the reader with (#3738): is
 * this Vize being wrong, or the instrument being broken?
 *
 * The auditor of run 30738583070 could not tell, and read the answer off the
 * ratios — 0.87 to 0.99 looked like two programs, 0.06 looked like a compiler.
 * That inference is not available from a ratio, and it was wrong in both
 * directions on that run. The verdict already carries the answer, because every
 * `unusable` reason is a proof that the baseline did not measure Vize; it just
 * was not stated. So state it, on the failure and in the step summary, and say
 * what the usable case rests on rather than leaving it implied.
 */
export function describeClassification(artifact) {
  const budget = artifact.budget;
  if (budget.verdict === "unusable") {
    return "instrument failure, the vue-tsc baseline did not measure Vize";
  }
  const coverage = artifact.baseline.coverage;
  return (
    "Vize divergence, the vue-tsc baseline loaded cleanly over the same " +
    `${coverage.sharedVueFileCount} Vue files`
  );
}

function describeBreaches(artifact) {
  const budget = artifact.budget;
  const summary = artifact.divergence.summary;
  const breaches = [];
  if (!budget.messageMismatchPassed) {
    breaches.push(
      `${summary.messageMismatchCount} message mismatches require an explicit documented-difference entry`,
    );
  }
  if (!budget.falsePositivePassed) {
    breaches.push(
      `${summary.falsePositiveCount} false positives (ratio ${summary.falsePositiveRatio}) exceed maxFalsePositiveRatio ${budget.maxFalsePositiveRatio}`,
    );
  }
  if (!budget.falseNegativePassed) {
    breaches.push(
      `${summary.falseNegativeCount} false negatives (ratio ${summary.falseNegativeRatio}) exceed maxFalseNegativeRatio ${budget.maxFalseNegativeRatio}`,
    );
  }
  return breaches;
}

function ratio(value, name) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0 || value > 1) {
    throw new Error(`typecheckPerformance.${name} must be a finite number between 0 and 1`);
  }
}

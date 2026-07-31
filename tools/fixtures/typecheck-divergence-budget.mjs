/**
 * Verdict and enforcement for the vize-versus-vue-tsc false-positive /
 * false-negative budget, split out of `typecheck-divergence-report.mjs` so the
 * report script stays inside the per-file line budget.
 *
 * Two rules live here, and they answer two different failures the ledger had:
 *
 * 1. `evaluateBudget` returns three verdicts, not two (#3513, the #3222 parity
 *    ledger). The baseline is unusable when `vue-tsc --listFiles` proves the
 *    two tools checked different Vue corpora, or when two non-empty diagnostic
 *    streams have no mapped position in common.
 * 2. `assertBudgetPassed` enforces on the weekly sweep and records everywhere
 *    else — see `parseBudgetMode`.
 */

/**
 * `enforce` fails the run on anything that is not a clean pass. `record-only`
 * writes the same verdict into the artifact and annotates the run, but exits 0.
 *
 * `enforce` is the default so that a caller which forgets the flag fails closed;
 * only `.github/workflows/real-project-matrix.yml` opts out, and only on the
 * release-evidence dispatch. An unrecognised value is rejected rather than
 * treated as "do not enforce", so a typo cannot silently disarm the gate.
 */
const budgetModes = ["enforce", "record-only"];

export function parseBudgetMode(value) {
  if (!budgetModes.includes(value)) {
    throw new Error(`--budget-mode must be one of: ${budgetModes.join(", ")}`);
  }
  return value;
}

export function evaluateBudget(performance, summary, coverage) {
  const falsePositivePassed = summary.falsePositiveRatio <= performance.maxFalsePositiveRatio;
  const falseNegativePassed = summary.falseNegativeRatio <= performance.maxFalseNegativeRatio;
  const unusableReason = coverage.unusableReason ?? diagnosticMappingUnusableReason(summary);
  const verdict =
    unusableReason != null
      ? "unusable"
      : falsePositivePassed && falseNegativePassed
        ? "passed"
        : "breached";
  return {
    maxFalsePositiveRatio: performance.maxFalsePositiveRatio,
    maxFalseNegativeRatio: performance.maxFalseNegativeRatio,
    falsePositivePassed,
    falseNegativePassed,
    unusableReason,
    verdict,
    passed: verdict === "passed",
  };
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
 * It is the *weekly* gate. `real-project-matrix.yml` is also dispatched as a
 * required release gate, and while the ecosystem baseline is unusable on most of
 * the corpus (#3513) a breach there would block every release on a broken
 * instrument, so the release dispatch passes `--budget-mode record-only`: the
 * verdict is still computed, written to both artifacts and raised as a workflow
 * warning, it just does not fail the job.
 *
 * Either way this runs after both artifacts are written, so a breach is uploaded
 * and reviewable — the run fails with the evidence attached, not instead of it.
 */
export function assertBudgetPassed(artifact, budgetMode = "enforce") {
  // Validated before the passed-verdict return, so an unrecognised mode is
  // rejected on every run rather than only on the runs that breach.
  const mode = parseBudgetMode(budgetMode);
  const budget = artifact.budget;
  if (budget.verdict === "passed") return;
  const detail =
    budget.verdict === "unusable"
      ? `Typecheck divergence baseline is unusable for ${artifact.project}: ${budget.unusableReason}`
      : `Typecheck divergence budget breached for ${artifact.project}: ${describeBreaches(artifact).join("; ")}`;
  if (mode === "enforce") throw new Error(detail);
  // A GitHub workflow command, so a release run that records an unusable
  // baseline still shows a warning on the run instead of a silent green tick.
  process.stdout.write(`::warning title=Typecheck divergence budget not enforced::${detail}\n`);
}

function describeBreaches(artifact) {
  const budget = artifact.budget;
  const summary = artifact.divergence.summary;
  const breaches = [];
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

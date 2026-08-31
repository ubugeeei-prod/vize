/**
 * Verdict and enforcement for the vize-versus-vue-tsc false-positive /
 * false-negative budget, split out of `typecheck-divergence-report.mjs` so the
 * report script stays inside the per-file line budget.
 *
 * Two rules live here, and they answer two different failures the ledger had:
 *
 * 1. `evaluateBudget` returns three verdicts, not two (#3513, the #3222 parity
 *    ledger). The baseline is unusable when `vue-tsc` could not load the
 *    fixture's project configuration, when `vue-tsc --listFilesOnly` proves the two
 *    tools checked different Vue corpora, when that same listing proves the
 *    baseline resolved its Vue runtime outside the fixture (run 31979524200), or
 *    when two
 *    non-empty diagnostic streams have no mapped position in common.
 * 2. `assertBudgetPassed` enforces by default. `record-only` exists only for
 *    explicit ad hoc evidence capture, never for release evidence.
 */

/**
 * `enforce` fails the run on anything that is not a clean pass. `record-only`
 * writes the same verdict into the artifact and annotates the run, but exits 0.
 *
 * `enforce` is the default so that a caller which forgets the flag fails closed.
 * An unrecognised value is rejected rather than treated as "do not enforce", so
 * a typo cannot silently disarm the gate.
 */
const budgetModes = ["enforce", "record-only"];

export function parseBudgetMode(value) {
  if (!budgetModes.includes(value)) {
    throw new Error(`--budget-mode must be one of: ${budgetModes.join(", ")}`);
  }
  return value;
}

export function evaluateBudget(
  performance,
  summary,
  coverage,
  configuration,
  mutationOracle,
  ambient,
) {
  const falsePositivePassed = summary.falsePositiveRatio <= performance.maxFalsePositiveRatio;
  const falseNegativePassed = summary.falseNegativeRatio <= performance.maxFalseNegativeRatio;
  // Configuration first: it is the *cause* a coverage or mapping failure would
  // only be a symptom of, and it is the one reason that survives a run where
  // both other checks look clean. The ambient environment sits directly after
  // it, for the same reason: on run 31979524200 it was the cause and every other
  // check was clean, so a symptom-ordered list would have buried it.
  const unusableReason =
    configuration.unusableReason ??
    ambientUnusableReason(ambient) ??
    coverage.unusableReason ??
    mutationOracleUnusableReason(mutationOracle) ??
    diagnosticMappingUnusableReason(summary);
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
 * A missing ambient evaluation is a missing gate, not a passing one: the failure
 * it exists to catch is invisible in every other field of the artifact, so a
 * caller that forgets to pass it must fail rather than inherit a silent pass.
 */
function ambientUnusableReason(ambient) {
  if (ambient?.verdict === "isolated" && ambient.unusableReason == null) return null;
  return ambient?.unusableReason ?? "baseline ambient environment evidence is missing";
}

function mutationOracleUnusableReason(mutationOracle) {
  if (mutationOracle?.passed === true && mutationOracle.verdict === "passed") return null;
  return mutationOracle?.unusableReason ?? "seeded mutation oracle evidence is missing";
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
 * It is also a required release gate, so release dispatches use the default
 * `enforce` mode and preflight rejects artifacts that were captured in
 * `record-only` mode.
 *
 * Either way this runs after artifacts are written, so a breach is uploaded and
 * reviewable — the run fails with the evidence attached, not instead of it.
 */
export function assertBudgetPassed(artifact, budgetMode = "enforce") {
  // Validated before the passed-verdict return, so an unrecognised mode is
  // rejected on every run rather than only on the runs that breach.
  const mode = parseBudgetMode(budgetMode);
  const failures = collectBudgetFailures([artifact], mode);
  if (failures.length > 0) throw new Error(failures.join("\n"));
}

export function assertBudgetsPassed(artifacts, budgetMode = "enforce") {
  const mode = parseBudgetMode(budgetMode);
  const failures = collectBudgetFailures(artifacts, mode);
  if (failures.length > 0) throw new Error(failures.join("\n"));
}

function collectBudgetFailures(artifacts, mode) {
  const failures = [];
  for (const artifact of artifacts) {
    const detail = budgetFailureDetail(artifact);
    if (detail == null) continue;
    if (mode === "enforce") failures.push(detail);
    else {
      process.stdout.write(`::warning title=Typecheck divergence budget not enforced::${detail}\n`);
    }
  }
  return failures;
}

function budgetFailureDetail(artifact) {
  const budget = artifact.budget;
  if (budget.verdict === "passed") return null;
  return `${
    budget.verdict === "unusable"
      ? `Typecheck divergence baseline is unusable for ${artifact.project}`
      : `Typecheck divergence budget breached for ${artifact.project}`
  } — ${describeClassification(artifact)}: ${
    budget.verdict === "unusable" ? budget.unusableReason : describeBreaches(artifact).join("; ")
  }`;
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
  // The Vue-file count alone was the whole claim on run 31979524200, and it was
  // true while the baseline typed those files against a foreign `vue`.
  // So the environment the files were loaded in is stated beside their count.
  return (
    "Vize divergence, the vue-tsc baseline loaded cleanly over the same " +
    `${coverage.sharedVueFileCount} Vue files, against the fixture's own Vue runtime`
  );
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

/**
 * Pure result classification, formatting, and confirmation for the PR
 * benchmark gate.
 */

export type BenchmarkStatus = "regression" | "faster" | "stable";

/** One lane's paired base/head samples and the budget it is judged against. */
export type BenchmarkRuns = {
  id: string;
  label: string;
  baseRuns: number[];
  headRuns: number[];
  thresholdPercent: number;
};

export type BenchmarkResult = BenchmarkRuns & {
  baseMs: number;
  headMs: number;
  rate: number;
  changePercent: number;
  status: BenchmarkStatus;
  pairRates: number[];
  /** The initial and confirmation samples when a breach was re-measured. */
  attempts?: BenchmarkResult[];
};

/**
 * A budget failure as rendered. JSON stores a non-finite rate as `null`, and
 * artifacts written before per-lane budgets carry no `thresholdPercent`.
 */
export type RegressionLine = {
  label: string;
  rate: number | null;
  changePercent: number | null;
  thresholdPercent?: number | null;
};

/** A lane result as the budget reads it, in memory or back from JSON. */
export type BudgetCandidate = RegressionLine & { id: string; status: string };
export type BudgetRegression = RegressionLine & { id: string };
export type BenchmarkBudget = {
  status: "failed" | "passed";
  regressionCount: number;
  regressions: BudgetRegression[];
};
export type PrBenchmarkReport = {
  baseLabel: string;
  headLabel: string;
  inputDir?: string;
  fileCount: number;
  runs: number;
  warmups: number;
  thresholdPercent: number;
  results: BenchmarkResult[];
  budget?: BenchmarkBudget;
};

export function formatRate(value: number | null): string {
  if (value === null || !Number.isFinite(value)) {
    return "n/a";
  }
  return `${value.toFixed(3)}x`;
}

export function formatPercent(value: number | null): string {
  if (value === null || !Number.isFinite(value)) {
    return "n/a";
  }
  const sign = value > 0 ? "+" : "";
  return `${sign}${value.toFixed(2)}%`;
}

/**
 * One budget failure, naming the budget it broke. Lanes no longer share a
 * single threshold, so a bare rate does not say what the gate compared against.
 */
export function formatRegressionLine(regression: RegressionLine): string {
  const budget =
    regression.thresholdPercent == null ? "" : ` over a ${regression.thresholdPercent}% budget`;
  return `- ${regression.label}: ${formatRate(regression.rate)} (${formatPercent(regression.changePercent)})${budget}`;
}

function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 1) {
    return sorted[mid];
  }
  return (sorted[mid - 1] + sorted[mid]) / 2;
}

export function summarizeBenchmarkRuns({
  id,
  label,
  baseRuns,
  headRuns,
  thresholdPercent,
}: BenchmarkRuns): BenchmarkResult {
  if (baseRuns.length !== headRuns.length || baseRuns.length === 0) {
    throw new Error("Benchmark samples must contain equally sized, non-empty base/head pairs");
  }
  const baseMs = median(baseRuns);
  const headMs = median(headRuns);
  const pairRates = baseRuns.map((baseMs, index) =>
    baseMs === 0 ? Number.NaN : headRuns[index] / baseMs,
  );
  const rate = pairRates.every(Number.isFinite) ? median(pairRates) : Number.NaN;
  const changePercent = Number.isFinite(rate) ? (rate - 1) * 100 : Number.NaN;
  const status: BenchmarkStatus =
    changePercent >= thresholdPercent
      ? "regression"
      : changePercent <= -thresholdPercent
        ? "faster"
        : "stable";

  return {
    id,
    label,
    baseMs,
    headMs,
    rate,
    changePercent,
    status,
    thresholdPercent,
    baseRuns,
    headRuns,
    pairRates,
  };
}

// #3621: no-code comparisons have crossed the 5% budget by both 5.37% and
// 15.11%, then passed when the measurements (not only the budget job) were
// rerun. A real regression must reproduce in a fresh sample; one runner-noise
// excursion must not cancel a release after every other exact-SHA gate passes.
export function confirmRegressions<Task>(
  measurements: { task: Task; result: BenchmarkResult }[],
  remeasure: (task: Task) => BenchmarkResult,
  thresholdPercent: number,
): BenchmarkResult[] {
  return measurements.map(({ task, result }) => {
    if (result.status !== "regression") {
      return result;
    }
    const confirmation = remeasure(task);
    const combined = summarizeBenchmarkRuns({
      id: result.id,
      label: result.label,
      baseRuns: [...result.baseRuns, ...confirmation.baseRuns],
      headRuns: [...result.headRuns, ...confirmation.headRuns],
      // A lane is judged against its own budget, so pooling the confirmation
      // samples must not silently re-judge it against the shared default.
      thresholdPercent: result.thresholdPercent ?? thresholdPercent,
    });
    return {
      ...combined,
      attempts: [result, confirmation],
    };
  });
}

/**
 * Pure result classification and confirmation for the PR benchmark gate.
 */

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  if (sorted.length % 2 === 1) {
    return sorted[mid];
  }
  return (sorted[mid - 1] + sorted[mid]) / 2;
}

export function summarizeBenchmarkRuns({ id, label, baseRuns, headRuns, thresholdPercent }) {
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
  const status =
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
    baseRuns,
    headRuns,
    pairRates,
  };
}

// #3621: no-code comparisons have crossed the 5% budget by both 5.37% and
// 15.11%, then passed when the measurements (not only the budget job) were
// rerun. A real regression must reproduce in a fresh sample; one runner-noise
// excursion must not cancel a release after every other exact-SHA gate passes.
export function confirmRegressions(measurements, remeasure, thresholdPercent) {
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
      thresholdPercent,
    });
    return {
      ...combined,
      attempts: [result, confirmation],
    };
  });
}

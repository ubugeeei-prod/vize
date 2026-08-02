/**
 * Pure result classification, formatting, and confirmation for the PR
 * benchmark gate.
 */

export function formatRate(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  return `${value.toFixed(3)}x`;
}

export function formatPercent(value) {
  if (!Number.isFinite(value)) {
    return "n/a";
  }
  const sign = value > 0 ? "+" : "";
  return `${sign}${value.toFixed(2)}%`;
}

/**
 * One budget failure, naming the budget it broke. Lanes no longer share a
 * single threshold, so a bare rate does not say what the gate compared against.
 */
export function formatRegressionLine(regression) {
  const budget =
    regression.thresholdPercent == null ? "" : ` over a ${regression.thresholdPercent}% budget`;
  return `- ${regression.label}: ${formatRate(regression.rate)} (${formatPercent(regression.changePercent)})${budget}`;
}

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

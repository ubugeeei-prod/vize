import assert from "node:assert/strict";

export const churnCycleSamplePolicy = {
  population:
    "every measured cycle from both churn phases; one sample is four ordered edits through diagnostics convergence",
  preCycleWarmup:
    "initialize, cold open, and baseline convergence finish before cycle timing starts",
  warmupCyclesExcluded: 0,
  outlierSamplesExcluded: 0,
  percentileMethod: "nearest-rank order statistic at sorted[ceil(q * n) - 1]",
} as const;

export type ChurnCycleStats = {
  cycles: number;
  minMs: number;
  medianMs: number;
  p50Ms: number;
  p95Ms: number;
  p99Ms: number;
  maxMs: number;
  samplePolicy: typeof churnCycleSamplePolicy;
};

/** Summarizes every completed churn cycle without warmup or outlier filtering. */
export function summarizeChurnCycles(values: number[]): ChurnCycleStats {
  assert(values.length > 0, "at least one completed churn cycle is required");
  assert(
    values.every((value) => Number.isFinite(value) && value >= 0),
    "churn cycle durations must be finite non-negative milliseconds",
  );
  const sorted = [...values].sort((a, b) => a - b);
  const percentile = (quantile: number) => sorted[Math.ceil(quantile * sorted.length) - 1];
  const p50Ms = percentile(0.5);
  return {
    cycles: sorted.length,
    minMs: sorted[0],
    // Preserve the existing upper-median value for additive schema compatibility.
    medianMs: sorted[Math.floor(sorted.length / 2)],
    p50Ms,
    p95Ms: percentile(0.95),
    p99Ms: percentile(0.99),
    maxMs: sorted[sorted.length - 1],
    samplePolicy: churnCycleSamplePolicy,
  };
}

/**
 * Narrowing for the PR benchmark artifact that `enforce-pr-budget.ts` reads
 * back from JSON: `compare-pr.ts` results, or the stub the workflow writes
 * when it skips the benchmark.
 */

import type { BenchmarkBudget, BudgetCandidate, BudgetRegression } from "./compare-pr-results.ts";
import { isOptional, isRecord, isStringArray } from "./script-types.ts";

export type ArtifactResult = BudgetCandidate & { attempts?: unknown[] };

/** `compare-pr.ts` results, or the stub the workflow writes when it skips. */
export type BenchmarkArtifact = {
  skipped?: boolean;
  reason?: string;
  labels?: string[];
  budget?: BenchmarkBudget;
  results?: ArtifactResult[];
};

// JSON stores a non-finite rate or change as null.
const isRate = (value: unknown): value is number | null =>
  value === null || typeof value === "number";

function isRegression(value: unknown): value is BudgetRegression {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.label === "string" &&
    isRate(value.rate) &&
    isRate(value.changePercent) &&
    isOptional(value.thresholdPercent, isRate)
  );
}

function isArtifactResult(value: unknown): value is ArtifactResult {
  return (
    isRecord(value) &&
    typeof value.status === "string" &&
    isOptional(value.attempts, Array.isArray) &&
    isRegression(value)
  );
}

function isBudget(value: unknown): value is BenchmarkBudget {
  return (
    isRecord(value) &&
    (value.status === "failed" || value.status === "passed") &&
    typeof value.regressionCount === "number" &&
    Array.isArray(value.regressions) &&
    value.regressions.every(isRegression)
  );
}

export function isBenchmarkArtifact(value: unknown): value is BenchmarkArtifact {
  return (
    isRecord(value) &&
    isOptional(value.skipped, (skipped) => typeof skipped === "boolean") &&
    isOptional(value.reason, (reason) => typeof reason === "string") &&
    isOptional(value.labels, isStringArray) &&
    isOptional(value.budget, isBudget) &&
    isOptional(
      value.results,
      (results) => Array.isArray(results) && results.every(isArtifactResult),
    )
  );
}

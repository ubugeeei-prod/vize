import assert from "node:assert/strict";
import { test } from "node:test";

import {
  churnCycleSamplePolicy,
  summarizeChurnCycles,
} from "../performance/support/churn-percentiles.ts";
import { auditChurnCycleStats } from "../performance/support/current-baseline/audit.ts";

test("churn percentiles retain the complete ordered sample population", () => {
  const samples = Array.from({ length: 40 }, (_, index) => 40 - index);
  const original = [...samples];
  const stats = summarizeChurnCycles(samples);

  assert.deepEqual(samples, original, "summary must not reorder the retained raw timings");
  assert.deepEqual(stats, {
    cycles: 40,
    minMs: 1,
    medianMs: 21,
    p50Ms: 20,
    p95Ms: 38,
    p99Ms: 40,
    maxMs: 40,
    samplePolicy: churnCycleSamplePolicy,
  });
  assert.equal(stats.samplePolicy.warmupCyclesExcluded, 0);
  assert.equal(stats.samplePolicy.outlierSamplesExcluded, 0);
});

test("churn percentile input rejects missing and invalid observations", () => {
  assert.throws(() => summarizeChurnCycles([]), /at least one completed churn cycle/);
  assert.throws(() => summarizeChurnCycles([1, Number.NaN]), /finite non-negative/);
  assert.throws(() => summarizeChurnCycles([1, -1]), /finite non-negative/);
});

test("the baseline audit verifies reported percentiles against every raw cycle", () => {
  const samples = Array.from({ length: 40 }, (_, index) => index + 1);
  const stats = summarizeChurnCycles(samples);

  assert.doesNotThrow(() => auditChurnCycleStats(samples, stats, 40));
  assert.throws(
    () => auditChurnCycleStats([...samples.slice(0, -1), -1], stats, 40),
    /finite non-negative/,
  );
  assert.throws(
    () => auditChurnCycleStats(samples, { ...stats, p95Ms: stats.p95Ms + 1 }, 40),
    /p95Ms must match the raw cycle population/,
  );
});

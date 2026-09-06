import assert from "node:assert/strict";
import { test } from "node:test";

import { evaluateBudget } from "./check-gate-report.mjs";

const baseline = {
  rows: [{ id: "vize-check-max", medianMs: 100 }],
};

void test("evaluateBudget fails closed when the head median is invalid", () => {
  for (const headMedianMs of [Number.NaN, Number.POSITIVE_INFINITY, 0, -1]) {
    assert.deepEqual(evaluateBudget(headMedianMs, baseline, 10), {
      status: "invalid-head-median",
      headMedianMs,
      thresholdPercent: 10,
    });
  }
});

void test("evaluateBudget keeps the no-baseline report state for valid head timings", () => {
  assert.deepEqual(evaluateBudget(90, null, 10), {
    status: "no-baseline",
    thresholdPercent: 10,
  });
});

void test("evaluateBudget reports an invalid stored baseline separately", () => {
  assert.deepEqual(evaluateBudget(90, { rows: [] }, 10), {
    status: "invalid-baseline",
    thresholdPercent: 10,
  });
});

void test("evaluateBudget fails only when the threshold is reached", () => {
  assert.deepEqual(evaluateBudget(109.99, baseline, 10), {
    status: "passed",
    baseMedianMs: 100,
    headMedianMs: 109.99,
    changePercent: 9.99,
    thresholdPercent: 10,
  });
  assert.deepEqual(evaluateBudget(110, baseline, 10), {
    status: "failed",
    baseMedianMs: 100,
    headMedianMs: 110,
    changePercent: 10,
    thresholdPercent: 10,
  });
});

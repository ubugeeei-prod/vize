import assert from "node:assert/strict";
import { test } from "node:test";

import { createBenchmarkBudget, renderMarkdown } from "../../bench/compare-pr.mjs";
import { enforceBenchmarkBudget } from "../../bench/enforce-pr-budget.mjs";

const stableResult = {
  id: "compile",
  label: "Compile SFC",
  baseMs: 100,
  headMs: 102,
  rate: 1.02,
  changePercent: 2,
  status: "stable",
  baseRuns: [100, 101, 99],
  headRuns: [102, 101, 103],
};

const regressionResult = {
  id: "check",
  label: "Type check",
  baseMs: 100,
  headMs: 115,
  rate: 1.15,
  changePercent: 15,
  status: "regression",
  baseRuns: [99, 100, 101],
  headRuns: [114, 115, 116],
};

test("benchmark budget marks regression tasks as a failed gate", () => {
  const budget = createBenchmarkBudget([stableResult, regressionResult]);

  assert.equal(budget.status, "failed");
  assert.equal(budget.regressionCount, 1);
  assert.deepEqual(
    budget.regressions.map((regression) => regression.id),
    ["check"],
  );

  const enforcement = enforceBenchmarkBudget({
    results: [stableResult, regressionResult],
    budget,
  });

  assert.equal(enforcement.ok, false);
  assert.match(enforcement.message, /Type check/);
  assert.match(enforcement.message, /1\.150x/);
  assert.match(enforcement.message, /\+15\.00%/);
});

test("benchmark markdown reports the active regression budget", () => {
  const markdown = renderMarkdown({
    baseLabel: "base",
    headLabel: "head",
    fileCount: 300,
    runs: 5,
    warmups: 1,
    thresholdPercent: 5,
    results: [stableResult, regressionResult],
  });

  assert.match(markdown, /Budget: failed \(1 regression\)\./);
  assert.match(markdown, /Regression budget failures:/);
  assert.match(markdown, /Type check: 1\.150x \(\+15\.00%\)/);
});

test("benchmark budget allows skipped benchmark runs", () => {
  const enforcement = enforceBenchmarkBudget({
    skipped: true,
    reason: "base_metadata_invalid",
  });

  assert.equal(enforcement.ok, true);
  assert.match(enforcement.message, /base_metadata_invalid/);
});

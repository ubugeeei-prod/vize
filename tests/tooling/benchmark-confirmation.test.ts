import assert from "node:assert/strict";
import { test } from "node:test";

import { createBenchmarkBudget, renderMarkdown } from "../../tools/benchmarks/scripts/compare-pr.mjs";
import { confirmRegressions, summarizeBenchmarkRuns } from "../../tools/benchmarks/scripts/compare-pr-results.mjs";
import { enforceBenchmarkBudget } from "../../tools/benchmarks/scripts/enforce-pr-budget.mjs";

test("a transient breach is judged over the initial and confirmation pairs together", () => {
  const initial = summarizeBenchmarkRuns({
    id: "lint-max",
    label: "Lint (max)",
    baseRuns: Array(10).fill(100),
    headRuns: Array(10).fill(106),
    thresholdPercent: 5,
  });
  const confirmation = summarizeBenchmarkRuns({
    id: "lint-max",
    label: "Lint (max)",
    baseRuns: Array(10).fill(100),
    headRuns: Array(10).fill(100),
    thresholdPercent: 5,
  });
  const [result] = confirmRegressions(
    [{ task: "lint-max", result: initial }],
    () => confirmation,
    5,
  );

  assert.equal(result.status, "stable");
  assert.deepEqual(result.attempts, [initial, confirmation]);
  assert.equal(result.baseRuns.length, 20);
  assert.equal(result.headRuns.length, 20);
  assert.equal(result.changePercent, 3.0000000000000027);
  assert.equal(createBenchmarkBudget([result]).status, "passed");
  const enforcement = enforceBenchmarkBudget({ results: [result] });
  assert.equal(enforcement.ok, true);
  assert.match(enforcement.message, /paired confirmation samples: Lint \(max\)/);

  const markdown = renderMarkdown({
    baseLabel: "base",
    headLabel: "head",
    fileCount: 300,
    runs: 10,
    warmups: 1,
    thresholdPercent: 5,
    results: [result],
  });
  assert.match(markdown, /Initial Result: 1\.060x \(\+6\.00%\), regression/);
  assert.match(markdown, /Confirmation Result: 1\.000x \(0\.00%\), stable/);
});

test("a persistent regression still fails over the combined paired sample", () => {
  const initial = summarizeBenchmarkRuns({
    id: "check",
    label: "Type check",
    baseRuns: Array(10).fill(100),
    headRuns: Array(10).fill(106),
    thresholdPercent: 5,
  });
  const [result] = confirmRegressions([{ task: "check", result: initial }], () => initial, 5);

  assert.equal(result.status, "regression");
  assert.equal(result.changePercent, 6.000000000000005);
  assert.equal(createBenchmarkBudget([result]).status, "failed");
  const enforcement = enforceBenchmarkBudget({ results: [result] });
  assert.equal(enforcement.ok, false);
  assert.match(enforcement.message, /after paired confirmation/);
});

test("paired rates reject the real no-code sample whose independent medians read +15.11%", () => {
  const result = summarizeBenchmarkRuns({
    id: "check",
    label: "Type check (1T)",
    baseRuns: [
      854.049178, 851.61708, 839.336498, 1236.117059, 1132.082518, 1207.947092, 1124.93331,
      805.315192, 837.220344, 821.461661,
    ],
    headRuns: [
      847.309434, 840.243178, 1251.908881, 1241.231136, 1071.22636, 1205.936823, 1191.733354,
      892.158345, 830.561208, 838.986677,
    ],
    thresholdPercent: 5,
  });

  assert.equal(result.status, "stable");
  assert.equal(result.changePercent, 0.12365040342325884);
  assert.equal(createBenchmarkBudget([result]).status, "passed");
});

test("stable lanes are never measured a second time", () => {
  const stable = summarizeBenchmarkRuns({
    id: "compile",
    label: "Compile SFC",
    baseRuns: Array(10).fill(100),
    headRuns: Array(10).fill(102),
    thresholdPercent: 5,
  });
  let confirmations = 0;
  const [result] = confirmRegressions(
    [{ task: "compile", result: stable }],
    () => {
      confirmations += 1;
      return stable;
    },
    5,
  );

  assert.equal(result, stable);
  assert.equal(confirmations, 0);
});

test("an exact-threshold regression remains a failure after confirmation", () => {
  const initial = summarizeBenchmarkRuns({
    id: "compile",
    label: "Compile SFC",
    baseRuns: Array(10).fill(100),
    headRuns: Array(10).fill(105),
    thresholdPercent: 5,
  });
  const [result] = confirmRegressions([{ task: "compile", result: initial }], () => initial, 5);

  assert.equal(result.status, "regression");
  assert.equal(result.changePercent, 5.000000000000004);
});

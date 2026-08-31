import assert from "node:assert/strict";
import { test } from "node:test";

import {
  UNPINNED_FAST_LANE_THRESHOLD_PERCENT,
  laneThresholdPercent,
  makeTasks,
} from "../../tools/benchmarks/scripts/compare-pr-lanes.mjs";
import {
  confirmRegressions,
  summarizeBenchmarkRuns,
} from "../../tools/benchmarks/scripts/compare-pr-results.mjs";
import {
  createBenchmarkBudget,
  renderMarkdown,
} from "../../tools/benchmarks/scripts/compare-pr.mjs";
import { enforceBenchmarkBudget } from "../../tools/benchmarks/scripts/enforce-pr-budget.mjs";
import { readRepoFile } from "./support/github-workflows.ts";

const DEFAULT_THRESHOLD_PERCENT = 5;

/**
 * Real samples from the `Benchmark` gate of a release commit, whose diff is
 * version strings in manifests, `Cargo.lock`, `editors/*` and READMEs. Nothing
 * in it reaches the measured binary's throughput, so every point of the rate it
 * produces is runner noise.
 *
 * v0.315.1, run 30713745830 (70aaa2e5...f97253d5), lane `fmt-max`.
 */
const manifestOnlyFormatMax = {
  id: "fmt-max",
  label: "Format (max)",
  baseRuns: [
    39.97164899999916, 37.96923400000014, 36.34706499999993, 37.50640199999907, 35.47929900000054,
    40.74189099999967, 36.78259699999944, 36.578010999999606, 37.464882000000216,
    44.984419000000344,
  ],
  headRuns: [
    38.40237999999954, 37.779505999998946, 38.43414000000121, 37.437221000000136, 37.85763399999996,
    34.46102900000005, 39.0690969999996, 37.8511010000002, 39.52768199999991, 49.52019099999961,
  ],
};

/** The same shape of no-code diff, at the rate that cancelled v0.312.0. */
const releaseCancellingLintMax = {
  id: "lint-max",
  label: "Lint (max)",
  baseRuns: Array(10).fill(37),
  headRuns: Array(10).fill(37 * 1.0537),
};

const twentyPercentRegression = {
  id: "lint-max",
  label: "Lint (max)",
  baseRuns: Array(10).fill(37),
  headRuns: Array(10).fill(37 * 1.2),
};

test("only the two sub-40ms unpinned lanes carry a widened budget", () => {
  assert.deepEqual(
    makeTasks("/nonexistent-input-dir", "").map((task) => [
      task.id,
      task.threads ?? 1,
      laneThresholdPercent(task, DEFAULT_THRESHOLD_PERCENT),
    ]),
    [
      ["compile", 1, 5],
      ["lint", 1, 5],
      ["lint-max", "max", 10],
      ["fmt", 1, 5],
      ["fmt-max", "max", 10],
    ],
  );
  assert.equal(UNPINNED_FAST_LANE_THRESHOLD_PERCENT, 10);
});

test("the workflow still supplies the 5% default the pinned lanes are sized for", () => {
  const workflow = readRepoFile(".github", "workflows", "benchmark.yml");
  assert.equal(
    workflow.includes("\n  VIZE_BENCH_REGRESSION_THRESHOLD_PERCENT: 5\n"),
    true,
    "pinned lanes must keep the 5% budget the lane catalogue documents",
  );
});

test("a manifest-only release diff does not fail the budget", () => {
  const result = summarizeBenchmarkRuns({
    ...manifestOnlyFormatMax,
    thresholdPercent: UNPINNED_FAST_LANE_THRESHOLD_PERCENT,
  });

  assert.equal(result.changePercent, 4.4932176205962016);
  assert.equal(result.status, "stable");
  assert.deepEqual(createBenchmarkBudget([result]), {
    status: "passed",
    regressionCount: 0,
    regressions: [],
  });
  assert.deepEqual(enforceBenchmarkBudget({ results: [result] }), {
    ok: true,
    message: "Benchmark budget passed.",
  });

  // The same samples land 0.51 points inside the budget the pinned lanes use,
  // which is why this lane needed its own.
  assert.equal(
    summarizeBenchmarkRuns({
      ...manifestOnlyFormatMax,
      thresholdPercent: DEFAULT_THRESHOLD_PERCENT,
    }).status,
    "stable",
  );
});

test("the no-code rate that cancelled v0.312.0 no longer fails its lane", () => {
  const atLaneBudget = summarizeBenchmarkRuns({
    ...releaseCancellingLintMax,
    thresholdPercent: UNPINNED_FAST_LANE_THRESHOLD_PERCENT,
  });
  assert.equal(atLaneBudget.changePercent, 5.370000000000008);
  assert.equal(atLaneBudget.status, "stable");
  assert.deepEqual(createBenchmarkBudget([atLaneBudget]), {
    status: "passed",
    regressionCount: 0,
    regressions: [],
  });

  // Unchanged for the lanes the 5% figure was actually calibrated on.
  assert.equal(
    summarizeBenchmarkRuns({
      ...releaseCancellingLintMax,
      id: "lint",
      label: "Lint (1T)",
      thresholdPercent: DEFAULT_THRESHOLD_PERCENT,
    }).status,
    "regression",
  );
});

test("a 20% regression still fails the widened lane budget", () => {
  const initial = summarizeBenchmarkRuns({
    ...twentyPercentRegression,
    thresholdPercent: UNPINNED_FAST_LANE_THRESHOLD_PERCENT,
  });
  assert.equal(initial.changePercent, 19.999999999999996);
  assert.equal(initial.status, "regression");

  // Confirmation pools the samples; it must keep judging the lane against the
  // lane's own budget rather than falling back to the shared default.
  const [confirmed] = confirmRegressions(
    [{ task: twentyPercentRegression, result: initial }],
    () =>
      summarizeBenchmarkRuns({
        ...twentyPercentRegression,
        thresholdPercent: UNPINNED_FAST_LANE_THRESHOLD_PERCENT,
      }),
    DEFAULT_THRESHOLD_PERCENT,
  );
  assert.equal(confirmed.thresholdPercent, UNPINNED_FAST_LANE_THRESHOLD_PERCENT);
  assert.equal(confirmed.status, "regression");
  assert.deepEqual(createBenchmarkBudget([confirmed]), {
    status: "failed",
    regressionCount: 1,
    regressions: [
      {
        id: "lint-max",
        label: "Lint (max)",
        rate: 1.2,
        changePercent: 19.999999999999996,
        thresholdPercent: 10,
      },
    ],
  });
  assert.deepEqual(enforceBenchmarkBudget({ results: [confirmed] }), {
    ok: false,
    message:
      "Benchmark regression budget failed for 1 task(s) after paired confirmation:\n" +
      "- Lint (max): 1.200x (+20.00%) over a 10% budget",
  });
});

test("the report names every lane judged against something other than the default", () => {
  const results = [
    summarizeBenchmarkRuns({
      ...manifestOnlyFormatMax,
      thresholdPercent: UNPINNED_FAST_LANE_THRESHOLD_PERCENT,
    }),
    summarizeBenchmarkRuns({
      ...releaseCancellingLintMax,
      id: "lint",
      label: "Lint (1T)",
      thresholdPercent: DEFAULT_THRESHOLD_PERCENT,
    }),
  ];
  const markdown = renderMarkdown({
    baseLabel: "base",
    headLabel: "head",
    fileCount: 1000,
    runs: 10,
    warmups: 2,
    thresholdPercent: DEFAULT_THRESHOLD_PERCENT,
    results,
  });
  assert.equal(
    markdown.split("\n").find((line) => line.startsWith("Median of ")),
    "Median of 10 adjacent head/base pairs after 2 warmup pair(s). Any threshold breach receives " +
      "10 fresh confirmation pairs, and the final rate is the paired median over all samples. " +
      "Times are shown in milliseconds to 0.001ms. Rate below 1.000x is faster. " +
      "Regression threshold: 5%, except Format (max) 10%.",
  );
});

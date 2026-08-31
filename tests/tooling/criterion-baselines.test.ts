import assert from "node:assert/strict";
import { test } from "node:test";

import {
  criterionBenchRunOptions,
  criterionSideTargetDirs,
  resolveSuiteSelection,
} from "../../tools/benchmarks/scripts/criterion-ab.mjs";
import {
  compareBaselineExports,
  criterionEnvironment,
  critcmpArgs,
  critcmpExportArgs,
  evaluateAbsoluteBudgets,
  parseCritcmpExport,
  validateComparisonTable,
} from "../../tools/benchmarks/scripts/criterion-baselines.mjs";
import { renderSummary } from "../../tools/benchmarks/scripts/criterion-summary.mjs";

test("Criterion driver snapshots both baselines before comparing them", () => {
  assert.deepEqual(criterionSideTargetDirs("/work/head/target"), {
    baseTargetDir: "/work/head/target/base-target",
    headTargetDir: "/work/head/target/head-target",
  });
  assert.deepEqual(
    criterionBenchRunOptions({
      checkoutDir: "/work/head",
      targetDir: "/work/head/target/head-target",
    }),
    {
      cwd: "/work/head",
      env: { CARGO_TARGET_DIR: "/work/head/target/head-target" },
      capture: false,
    },
  );
  assert.deepEqual(criterionEnvironment("/work/head/target"), {
    CARGO_TARGET_DIR: "/work/head/target",
  });
  assert.deepEqual(critcmpExportArgs({ targetDir: "/work/head/target", baseline: "base" }), [
    "--target-dir",
    "/work/head/target",
    "--export",
    "base",
  ]);
  assert.deepEqual(
    critcmpArgs({
      targetDir: "/work/head/target",
      baselinePaths: ["/work/base.json", "/work/head.json"],
    }),
    ["--target-dir", "/work/head/target", "/work/base.json", "/work/head.json"],
  );
});

test("Criterion baseline comparison fails closed and uses exported medians", () => {
  const base = parseCritcmpExport(baselineExport("base", { shared: 100 }), "base");
  const head = parseCritcmpExport(baselineExport("head", { shared: 125 }), "head");

  assert.deepEqual(compareBaselineExports(base, head, 10), [{ name: "shared", changePercent: 25 }]);
  assert.throws(
    () => parseCritcmpExport(baselineExport("base", {}), "base"),
    /contains no benchmarks/,
  );
  assert.throws(
    () =>
      compareBaselineExports(
        base,
        parseCritcmpExport(baselineExport("head", { other: 90 }), "head"),
        10,
      ),
    /no shared benchmarks/,
  );
});

test("Criterion absolute budgets pass, fail, and reject missing measurements", () => {
  const head = parseCritcmpExport(
    baselineExport("head", { first: 4_500_000, selection: 150_000 }),
    "head",
  );
  assert.deepEqual(
    evaluateAbsoluteBudgets(head, [
      { name: "first", maxMedianNs: 20_000_000 },
      { name: "selection", maxMedianNs: 100_000 },
    ]),
    [
      { name: "first", medianNs: 4_500_000, maxMedianNs: 20_000_000, exceeded: false },
      { name: "selection", medianNs: 150_000, maxMedianNs: 100_000, exceeded: true },
    ],
  );
  assert.throws(
    () => evaluateAbsoluteBudgets(head, [{ name: "missing", maxMedianNs: 1_000_000 }]),
    /benchmark is missing/,
  );
  assert.throws(
    () => evaluateAbsoluteBudgets(head, [{ name: "first", maxMedianNs: 0 }]),
    /Invalid Criterion absolute budget/,
  );
  assert.throws(
    () =>
      evaluateAbsoluteBudgets(head, [
        { name: "first", maxMedianNs: 1 },
        { name: "first", maxMedianNs: 2 },
      ]),
    /Duplicate/,
  );
});

test("Criterion comparison table requires both base and head columns", () => {
  assert.doesNotThrow(() =>
    validateComparisonTable("group  base  head\n-----  ----  ----\nshared  1.00  1.12\n"),
  );
  assert.throws(
    () => validateComparisonTable("group  head\n-----  ----\nshared  1.00\n"),
    /did not produce base\/head columns/,
  );
});

test("Criterion driver reports a useful summary when no suite is affected", () => {
  const selection = resolveSuiteSelection({ selected: [], reason: "CLI-only change." });
  const summary = renderSummary({ table: "", threshold: undefined, regressions: [], selection });

  assert.match(summary, /Selection: CLI-only change\./);
  assert.match(summary, /Ran: none/);
  assert.match(summary, /Skipped: vize_atelier_sfc, vize_atelier_jsx/);
  assert.match(summary, /timing execution was skipped/);
});

test("Criterion summary makes hard absolute budget results reviewable", () => {
  const selection = resolveSuiteSelection({
    selected: ["vize_benchmarks"],
    reason: "Doctor TUI changed.",
  });
  const summary = renderSummary({
    table: "group  base  head\n-----  ----  ----\nfirst  1.00  1.10\n",
    threshold: undefined,
    regressions: [],
    selection,
    absoluteBudgetResults: [
      { name: "first", medianNs: 4_500_000, maxMedianNs: 20_000_000, exceeded: false },
      { name: "selection", medianNs: 1_250_000, maxMedianNs: 1_000_000, exceeded: true },
    ],
  });

  assert.match(summary, /Absolute median budgets/);
  assert.match(summary, /first \| 4\.50 ms \| 20\.00 ms \| PASS/);
  assert.match(summary, /selection \| 1\.25 ms \| 1\.00 ms \| FAIL/);
});

function baselineExport(name: string, medians: Record<string, number>): string {
  return JSON.stringify({
    name,
    benchmarks: Object.fromEntries(
      Object.entries(medians).map(([benchmark, pointEstimate]) => [
        benchmark,
        { criterion_estimates_v1: { median: { point_estimate: pointEstimate } } },
      ]),
    ),
  });
}

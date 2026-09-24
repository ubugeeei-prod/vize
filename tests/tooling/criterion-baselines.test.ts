import assert from "node:assert/strict";
import { test } from "node:test";

import {
  criterionBenchRunOptions,
  criterionSideTargetDirs,
  cargoBenchArgs,
  resolveSuiteSelection,
} from "../../tools/benchmarks/scripts/criterion-ab.mjs";
import {
  compareBaselineExports,
  compareVaporNativePairs,
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
      env: {
        CARGO_TARGET_DIR: "/work/head/target/head-target",
        CRITERION_HOME: "/work/head/target/head-target/criterion",
      },
      capture: false,
    },
  );
  assert.deepEqual(criterionEnvironment("/work/head/target"), {
    CARGO_TARGET_DIR: "/work/head/target",
    CRITERION_HOME: "/work/head/target/criterion",
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

test("Vapor A/B measures the same complete native/retained fixture on both revisions", () => {
  const base = vaporPairExport("base", {
    "vapor_native_pair/control_flow/s3": 18_000,
    "vapor_native_pair/control_flow/legacy": 16_000,
  });
  const head = vaporPairExport("head", {
    "vapor_native_pair/control_flow/s3": 14_000,
    "vapor_native_pair/control_flow/legacy": 16_000,
  });
  const results = compareVaporNativePairs(base, head);
  assert.equal(results.length, 7);
  const controlFlow = results.find(({ fixture }) => fixture === "control_flow");
  assert.equal(controlFlow?.baseRatio, 1.125);
  assert.equal(controlFlow?.headRatio, 0.875);
  const selection = resolveSuiteSelection({
    selected: ["vize_atelier_vapor"],
    reason: "Vapor-only dispatch.",
  });
  const summary = renderSummary({
    table: "group  base  head\n-----  ----  ----\ncontrol_flow  1.00  0.88\n",
    threshold: undefined,
    regressions: [],
    selection,
    vaporPairResults: results,
  });
  assert.match(
    summary,
    /control_flow \| 18\.00 µs \| 16\.00 µs \| 1\.125x \| 14\.00 µs \| 16\.00 µs \| 0\.875x/,
  );
  assert.match(summary, /report-only/);

  const missingLegacy = vaporPairExport("head", {}, ["vapor_native_pair/control_flow/legacy"]);
  assert.throws(() => compareVaporNativePairs(base, missingLegacy), /Missing Vapor pair benchmark/);
  const missingFixture = ["vapor_native_pair/text_runs/s3", "vapor_native_pair/text_runs/legacy"];
  assert.throws(
    () =>
      compareVaporNativePairs(
        vaporPairExport("base", {}, missingFixture),
        vaporPairExport("head", {}, missingFixture),
      ),
    /differs from the pinned corpus/,
  );
  const changedCorpus = vaporPairExport("head", {
    "vapor_native_pair/other/s3": 14_000,
    "vapor_native_pair/other/legacy": 16_000,
  });
  assert.throws(
    () => compareVaporNativePairs(base, changedCorpus),
    /differs from the pinned corpus/,
  );
});

test("Vapor Criterion driver filters to paired compile cases", () => {
  assert.deepEqual(
    cargoBenchArgs({
      pkg: "vize_atelier_vapor",
      benches: ["davinci"],
      filter: "vapor_native_pair",
      baseline: "head",
      targetDir: "/work/target",
    }),
    [
      "bench",
      "-p",
      "vize_atelier_vapor",
      "--bench",
      "davinci",
      "--target-dir",
      "/work/target",
      "--",
      "vapor_native_pair",
      "--save-baseline",
      "head",
    ],
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

function vaporPairExport(
  name: string,
  overrides: Record<string, number> = {},
  omitted: string[] = [],
) {
  const fixtures = [
    "components",
    "control_flow",
    "events",
    "expressions",
    "spreads",
    "templates",
    "text_runs",
  ];
  const medians = Object.fromEntries(
    fixtures.flatMap((fixture) => [
      [`vapor_native_pair/${fixture}/s3`, 10_000] as const,
      [`vapor_native_pair/${fixture}/legacy`, 10_000] as const,
    ]),
  );
  return parseCritcmpExport(
    baselineExport(
      name,
      Object.fromEntries(
        Object.entries({ ...medians, ...overrides }).filter(
          ([benchmark]) => !omitted.includes(benchmark),
        ),
      ),
    ),
    name,
  );
}

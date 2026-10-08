import assert from "node:assert/strict";
import { test } from "node:test";

import {
  ENGINE_CLASSES,
  evaluateBudget,
  measureRows,
  renderMarkdown,
} from "./check-gate-report.ts";
import type { MeasurementVariant } from "./check-gate-report.ts";

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

void test("cold and warmup observations never enter the rotated steady-state rows", () => {
  const events: string[] = [];
  const variants: MeasurementVariant<{ ms: number; diagnostics: number }>[] = ["a", "b", "c"].map(
    (id, index) => {
      let invocation = 0;
      return {
        id,
        label: id.toUpperCase(),
        engineClass: "tsgo-native",
        expectedDiagnostics: 3,
        notes: `row ${id}`,
        measure: () => {
          const times = [100 * (index + 1), 900, 800, 30, 10, 20];
          const ms = times[invocation++];
          events.push(id);
          return { ms, diagnostics: 3 };
        },
        countDiagnostics: (output) => output.diagnostics,
      };
    },
  );
  const rows = measureRows(variants, { runs: 3, warmups: 2 });
  assert.deepEqual(events, [
    "a",
    "b",
    "c",
    "b",
    "c",
    "a",
    "c",
    "a",
    "b",
    "a",
    "b",
    "c",
    "b",
    "c",
    "a",
    "c",
    "a",
    "b",
  ]);
  assert.deepEqual(
    rows,
    ["a", "b", "c"].map((id, index) => ({
      id,
      label: id.toUpperCase(),
      engineClass: "tsgo-native",
      status: "ok",
      coldMs: 100 * (index + 1),
      runs: [30, 10, 20],
      medianMs: 20,
      diagnosticCount: 3,
      warmupPasses: 2,
      notes: `row ${id}`,
    })),
  );
  rows.forEach((row, index) => assert.equal(row.runs, variants[index].runs));
});

void test("every cold, warmup and measured count mismatch refuses a report", () => {
  for (const [failedInvocation, phase] of [
    [1, "cold startup"],
    [2, "warmup 0"],
    [3, "measured run 0"],
  ] as const) {
    let invocation = 0;
    const variant: MeasurementVariant<{ ms: number; diagnostics: number }> = {
      id: "vize-check-max",
      label: "Vize check (max)",
      engineClass: "tsgo-native",
      expectedDiagnostics: 3,
      measure: () => ({ ms: 1, diagnostics: ++invocation === failedInvocation ? 2 : 3 }),
      countDiagnostics: (output) => output.diagnostics,
    };
    assert.throws(() => measureRows([variant], { runs: 2, warmups: 0 }), {
      name: "Error",
      message: `check-gate: vize-check-max reported 2 diagnostics during ${phase} (expected 3); refusing to publish a timing`,
    });
    assert.equal(invocation, failedInvocation);
  }
});

void test("the entire Markdown report retains engine labels, empty rows and attribution", () => {
  assert.deepEqual(ENGINE_CLASSES, {
    "typescript-js": "JS TypeScript engine (tsc)",
    "tsgo-native": "native TypeScript engine (tsgo)",
  });
  assert.equal(
    renderMarkdown({
      generatedAt: "2026-10-08T00:00:00.000Z",
      versions: { vize: "0.433.0", tsgo: "7.0.2", vueTsc: null, typescript: null, vue: "3.5.35" },
      binaries: { vize: { sha256: "native-sha" }, tsgo: { sha256: "tsgo-sha" } },
      entry: { tsconfigPath: "corpus/tsconfig.json", fileCount: 2, totalBytes: 1234 },
      backend: { vize: { typeError: true, corpus: true } },
      budget: { status: "no-baseline", thresholdPercent: 10 },
      skipped: { "typescript-js": "vue-tsc missing or skipped" },
      rows: [
        {
          id: "vize-check-max",
          label: "Vize check (max)",
          engineClass: "tsgo-native",
          status: "ok",
          coldMs: 1500,
          runs: [30, 10, 20],
          medianMs: 20,
          diagnosticCount: 3,
          warmupPasses: 1,
          notes: "native row",
        },
      ],
    }),
    [
      "## Vize Check Benchmark Gate",
      "",
      "Measured: 2026-10-08T00:00:00.000Z",
      "Versions: `0.433.0` · tsgo `7.0.2` · vue-tsc `missing` (typescript `n/a`) · vue `3.5.35`",
      "Binaries (sha256 of the measured file, re-checked after the run): vize=`native-sha` tsgo=`tsgo-sha`",
      "Entry point: `corpus/tsconfig.json` — 2 unique SFC files, 1,234 bytes.",
      "Backend readiness (planted-diagnostic gates, all required before timing): typeError=pass corpus=pass",
      "Budget: no-baseline",
      "",
      "### JS TypeScript engine (tsc)",
      "",
      "| Row | Cold start | Warmed median | Diagnostics | Measured runs |",
      "| --- | ---: | ---: | ---: | --- |",
      "| (vue-tsc missing or skipped) | n/a | n/a | n/a | n/a |",
      "",
      "### native TypeScript engine (tsgo)",
      "",
      "| Row | Cold start | Warmed median | Diagnostics | Measured runs |",
      "| --- | ---: | ---: | ---: | --- |",
      "| Vize check (max) | 1.50s | 20.0ms | 3 | 30.0ms, 10.0ms, 20.0ms |",
      "",
      "Engine classes are ranked separately: a cross-class ratio measures TypeScript's native rewrite as much as the Vue layer, so it is reported as context only.",
      "",
    ].join("\n"),
  );
});

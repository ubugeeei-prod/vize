import assert from "node:assert/strict";
import { test } from "node:test";

import {
  ENGINE_CLASSES,
  assertDiagnosticParity,
  evaluateBudget,
  fingerprintVizeDiagnostics,
  measureRows,
  renderMarkdown,
} from "./check-gate-report.ts";
import type { MeasurementVariant } from "./check-gate-report.ts";
import {
  MINIMAL_PLANTS,
  CORPUS_PLANT_DIAGNOSTICS,
  CORPUS_PLANT_FILE,
  vizeReportCatchesPlant,
  vizeCorpusPlantMatches,
} from "./check-gate-plants.mjs";

// Complete packets observed with the reported 0.566.0 product and pinned 7.0.2
// runtime. JSON template context is intentional since a67d5791a706; bare script
// diagnostics retain their existing contract.
const templatePackets = [
  {
    id: "templateProp",
    diagnostic:
      "error:6:26 [TS2322] Type 'string' is not assignable to type 'Booleanish | undefined'. (source: <button type=\"button\" :disabled=\"disabledFlag\">go</button>; binding: 'disabled')",
  },
  {
    id: "templateEvent",
    diagnostic:
      "error:6:33 [TS2345] Argument of type 'number' is not assignable to parameter of type '(_e: PointerEvent) => unknown'. (source: <button type=\"button\" @click=\"clickHandler\">go</button>; binding: @click)",
  },
  {
    id: "componentProp",
    diagnostic:
      'error:6:10 [TS2322] Type \'"danger"\' is not assignable to type \'"primary" | "secondary"\'. (source: <Child variant="danger" />; binding: variant)',
  },
] as const;

function plantReport(diagnostic: string) {
  return {
    files: [{ file: "App.vue", diagnostics: [diagnostic] }],
    errorCount: 1,
    warningCount: 0,
    fileCount: 1,
  };
}

void test("template readiness requires every byte of the current authored-context packets", () => {
  const script = MINIMAL_PLANTS.find((plant) => plant.id === "script");
  assert.equal(
    script?.expected.diagnostic,
    "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'.",
  );
  for (const packet of templatePackets) {
    const plant = MINIMAL_PLANTS.find((candidate) => candidate.id === packet.id);
    assert.equal(plant?.expected.diagnostic, packet.diagnostic);
    assert.ok(
      vizeReportCatchesPlant(plantReport(packet.diagnostic), "App.vue", [packet.diagnostic]),
    );
    for (const changed of [
      packet.diagnostic.replace(/ \(source:.*\)$/u, ""),
      packet.diagnostic.replace("source: ", "source: changed "),
      packet.diagnostic.replace(/; binding:.*\)$/u, ")"),
      packet.diagnostic.replace(/binding:.*\)$/u, "binding: changed)"),
      packet.diagnostic.replace("[TS23", "[TS24"),
    ]) {
      assert.notEqual(changed, packet.diagnostic);
      assert.ok(!vizeReportCatchesPlant(plantReport(changed), "App.vue", [packet.diagnostic]));
    }
    const stray = plantReport(packet.diagnostic);
    stray.files.push({ file: "Other.vue", diagnostics: ["error:1:1 [TS2304] unrelated"] });
    assert.ok(!vizeReportCatchesPlant(stray, "App.vue", [packet.diagnostic]));
    const wrongCount = plantReport(packet.diagnostic);
    wrongCount.errorCount++;
    assert.ok(!vizeReportCatchesPlant(wrongCount, "App.vue", [packet.diagnostic]));
  }
});

void test("corpus readiness retains all baseline packets and exact authored template context", () => {
  const script = "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'.";
  const template =
    "error:7:26 [TS2322] Type 'string' is not assignable to type 'Booleanish | undefined'. (source: <button type=\"button\" :disabled=\"disabledFlag\">{{ n }}</button>; binding: 'disabled')";
  assert.deepEqual(CORPUS_PLANT_DIAGNOSTICS, [script, template]);
  const baselineReport = plantReport(script);
  const planted = {
    ...baselineReport,
    files: [...baselineReport.files, { file: CORPUS_PLANT_FILE, diagnostics: [script, template] }],
    errorCount: 3,
    fileCount: 2,
  };
  assert.ok(vizeCorpusPlantMatches(baselineReport, planted));
  for (const changed of [
    template.replace(/ \(source:.*\)$/u, ""),
    template.replace("source: ", "source: changed "),
    template.replace("binding: 'disabled'", "binding: 'other'"),
  ]) {
    const mutated = structuredClone(planted);
    const row = mutated.files.find((file) => file.file === CORPUS_PLANT_FILE);
    assert.ok(row);
    row.diagnostics = [script, changed];
    assert.ok(!vizeCorpusPlantMatches(baselineReport, mutated));
  }
  const lostBaseline = structuredClone(planted);
  const row = lostBaseline.files.find((file) => file.file === "App.vue");
  assert.ok(row);
  row.diagnostics = [];
  assert.ok(!vizeCorpusPlantMatches(baselineReport, lostBaseline));
});

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

void test("diagnostic fingerprints cover exit status, counts, files and messages", () => {
  const report = {
    errorCount: 1,
    warningCount: 0,
    fileCount: 2,
    files: [
      { file: "Component.vue", diagnostics: ["error:2:4 [TS2322] incompatible value"] },
      { file: "Clean.vue", diagnostics: [] },
    ],
  };
  const fingerprint = fingerprintVizeDiagnostics(1, report);
  assert.match(fingerprint, /^[a-f0-9]{64}$/u);
  assert.equal(fingerprintVizeDiagnostics(1, structuredClone(report)), fingerprint);
  assert.equal(
    fingerprintVizeDiagnostics(1, { ...report, files: [...report.files].reverse() }),
    fingerprint,
  );
  assert.notEqual(fingerprintVizeDiagnostics(0, report), fingerprint);
  assert.notEqual(fingerprintVizeDiagnostics(1, { ...report, fileCount: 1 }), fingerprint);
  assert.notEqual(
    fingerprintVizeDiagnostics(1, {
      ...report,
      files: [{ file: "Other.vue", diagnostics: report.files[0].diagnostics }],
    }),
    fingerprint,
  );
  assert.notEqual(
    fingerprintVizeDiagnostics(1, {
      ...report,
      files: [{ file: "Component.vue", diagnostics: ["error:2:4 [TS2322] changed"] }],
    }),
    fingerprint,
  );
  assert.throws(() => fingerprintVizeDiagnostics(1, null), /cannot fingerprint an invalid/u);
});

void test("native checker widths must have complete diagnostic parity", () => {
  const fingerprint = "a".repeat(64);
  assert.deepEqual(
    assertDiagnosticParity([
      { id: "vize-check-1t", diagnosticFingerprint: fingerprint },
      { id: "vize-check-max", diagnosticFingerprint: fingerprint },
    ]),
    {
      status: "passed",
      diagnosticFingerprint: fingerprint,
      variants: ["vize-check-1t", "vize-check-max"],
    },
  );
  assert.throws(
    () =>
      assertDiagnosticParity([
        { id: "vize-check-1t", diagnosticFingerprint: fingerprint },
        { id: "vize-check-max", diagnosticFingerprint: "b".repeat(64) },
      ]),
    /native checker-width diagnostics changed/u,
  );
  assert.throws(
    () => assertDiagnosticParity([{ id: "vize-check-max", diagnosticFingerprint: fingerprint }]),
    /requires valid diagnostic fingerprints/u,
  );
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

void test("every cold, warmup and measured fingerprint mismatch refuses a report", () => {
  const expectedFingerprint = "a".repeat(64);
  for (const [failedInvocation, phase] of [
    [1, "cold startup"],
    [2, "warmup 0"],
    [3, "measured run 0"],
  ] as const) {
    let invocation = 0;
    const variant: MeasurementVariant<{
      ms: number;
      diagnostics: number;
      fingerprint: string;
    }> = {
      id: "vize-check-max",
      label: "Vize check (max)",
      engineClass: "tsgo-native",
      expectedDiagnostics: 3,
      expectedDiagnosticFingerprint: expectedFingerprint,
      measure: () => ({
        ms: 1,
        diagnostics: 3,
        fingerprint: ++invocation === failedInvocation ? "b".repeat(64) : expectedFingerprint,
      }),
      countDiagnostics: (output) => output.diagnostics,
      fingerprintDiagnostics: (output) => output.fingerprint,
    };
    assert.throws(() => measureRows([variant], { runs: 2, warmups: 0 }), {
      name: "Error",
      message: `check-gate: vize-check-max diagnostics changed during ${phase} (${"b".repeat(64)}, expected ${expectedFingerprint}); refusing to publish a timing`,
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
      backend: {
        vize: { typeError: true, corpus: true },
        diagnosticParity: {
          status: "passed",
          diagnosticFingerprint: "a".repeat(64),
          variants: ["vize-check-1t", "vize-check-max"],
        },
      },
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
      `Native checker-width diagnostic parity: passed for vize-check-1t / vize-check-max (\`${"a".repeat(64)}\`).`,
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

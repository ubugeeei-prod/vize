/**
 * Measurement and reporting half of the `vize check` benchmark gate.
 *
 * Cold startup is recorded separately from steady state: the first invocation
 * of every row lands in a dedicated cold column and never participates in the
 * warmed median. Measured rows are rotated so no tool owns a fixed slot, and
 * every measured run must repeat the expected diagnostic count so a row that
 * silently stops analysing can never publish a timing.
 */

export const ENGINE_CLASSES = {
  "typescript-js": "JS TypeScript engine (tsc)",
  "tsgo-native": "native TypeScript engine (tsgo)",
};

export type EngineClass = keyof typeof ENGINE_CLASSES;
type TimedOutput = { ms: number };
export type MeasurementVariant<Output extends TimedOutput> = {
  id: string;
  label: string;
  engineClass: EngineClass;
  expectedDiagnostics: number;
  notes?: string;
  measure: () => Output;
  countDiagnostics: (output: Output) => number;
  coldMs?: number;
  runs?: number[];
};
type InitializedVariant<Output extends TimedOutput> = MeasurementVariant<Output> & {
  coldMs: number;
  runs: number[];
};
export type MeasuredRow = {
  id: string;
  label: string;
  engineClass: EngineClass;
  status: "ok";
  coldMs: number;
  runs: number[];
  medianMs: number;
  diagnosticCount: number;
  warmupPasses: number;
  notes?: string;
};
type BudgetBaseline = { rows?: Array<{ id: string; medianMs?: number | null }> };
export type BudgetResult =
  | { status: "invalid-head-median"; headMedianMs: number; thresholdPercent: number }
  | { status: "no-baseline" | "invalid-baseline"; thresholdPercent: number }
  | {
      status: "passed" | "failed";
      baseMedianMs: number;
      headMedianMs: number;
      changePercent: number;
      thresholdPercent: number;
    };
export type CheckBenchmarkReport = {
  generatedAt: string;
  versions: {
    vize: string;
    tsgo: string;
    vueTsc?: string | null;
    typescript?: string | null;
    vue: string;
  };
  binaries: Record<string, { sha256?: string | null }>;
  entry: { tsconfigPath: string; fileCount: number; totalBytes: number };
  backend: { vize: Record<string, boolean> };
  budget: BudgetResult;
  rows: MeasuredRow[];
  skipped: Record<string, string | null | undefined>;
};

export function median(values: number[]): number {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

export function formatMs(ms: number): string {
  if (!Number.isFinite(ms)) return "n/a";
  return ms >= 1000 ? `${(ms / 1000).toFixed(2)}s` : `${ms.toFixed(1)}ms`;
}

export function rotate<Value>(list: Value[], by: number): Value[] {
  if (list.length === 0) return list;
  const k = ((by % list.length) + list.length) % list.length;
  return [...list.slice(k), ...list.slice(0, k)];
}

function checkedRun<Output extends TimedOutput>(
  variant: MeasurementVariant<Output>,
  phase: string,
): Output {
  const out = variant.measure();
  const count = variant.countDiagnostics(out);
  if (count !== variant.expectedDiagnostics) {
    throw new Error(
      `check-gate: ${variant.id} reported ${count} diagnostics during ${phase} (expected ${variant.expectedDiagnostics}); refusing to publish a timing`,
    );
  }
  return out;
}

function recordColdRows<Output extends TimedOutput>(
  variants: MeasurementVariant<Output>[],
): asserts variants is InitializedVariant<Output>[] {
  for (const variant of variants) {
    variant.coldMs = Number(checkedRun(variant, "cold startup").ms.toFixed(3));
    variant.runs = [];
  }
}

/**
 * Cold run first, then >=1 rotated warmup passes, then rotated measured runs.
 * Throws (fail closed) when any run's diagnostic count drifts.
 */
export function measureRows<Output extends TimedOutput>(
  variants: MeasurementVariant<Output>[],
  { runs, warmups }: { runs: number; warmups: number },
): MeasuredRow[] {
  recordColdRows(variants);
  const warmupPasses = Math.max(1, warmups);
  for (let pass = 0; pass < warmupPasses; pass++) {
    for (const variant of rotate(variants, pass + 1)) checkedRun(variant, `warmup ${pass}`);
  }
  for (let run = 0; run < runs; run++) {
    for (const variant of rotate(variants, run)) {
      variant.runs.push(Number(checkedRun(variant, `measured run ${run}`).ms.toFixed(3)));
    }
  }
  return variants.map((variant) => ({
    id: variant.id,
    label: variant.label,
    engineClass: variant.engineClass,
    status: "ok",
    coldMs: variant.coldMs,
    runs: variant.runs,
    medianMs: Number(median(variant.runs).toFixed(3)),
    diagnosticCount: variant.expectedDiagnostics,
    warmupPasses,
    notes: variant.notes,
  }));
}

/** Pure budget rule so tests can exercise it without re-measuring. */
export function evaluateBudget(
  headMedianMs: number,
  baseline: BudgetBaseline | null | undefined,
  thresholdPercent: number,
): BudgetResult {
  if (!Number.isFinite(headMedianMs) || headMedianMs <= 0) {
    return { status: "invalid-head-median", headMedianMs, thresholdPercent };
  }
  if (baseline == null) return { status: "no-baseline", thresholdPercent };
  const baseMedianMs = baseline?.rows?.find((row) => row.id === "vize-check-max")?.medianMs;
  if (typeof baseMedianMs !== "number" || !Number.isFinite(baseMedianMs) || baseMedianMs <= 0) {
    return { status: "invalid-baseline", thresholdPercent };
  }
  const changePercent = Number((((headMedianMs - baseMedianMs) / baseMedianMs) * 100).toFixed(2));
  return {
    status: changePercent >= thresholdPercent ? "failed" : "passed",
    baseMedianMs,
    headMedianMs,
    changePercent,
    thresholdPercent,
  };
}

export function renderMarkdown(data: CheckBenchmarkReport): string {
  const lines = ["## Vize Check Benchmark Gate", ""];
  lines.push(`Measured: ${data.generatedAt}`);
  lines.push(
    `Versions: \`${data.versions.vize}\` · tsgo \`${data.versions.tsgo}\` · vue-tsc \`${data.versions.vueTsc ?? "missing"}\` (typescript \`${data.versions.typescript ?? "n/a"}\`) · vue \`${data.versions.vue}\``,
  );
  lines.push(
    `Binaries (sha256 of the measured file, re-checked after the run): ${Object.entries(
      data.binaries,
    )
      .map(([label, binary]) => `${label}=\`${binary.sha256 ?? "unknown"}\``)
      .join(" ")}`,
  );
  lines.push(
    `Entry point: \`${data.entry.tsconfigPath}\` — ${data.entry.fileCount} unique SFC files, ${data.entry.totalBytes.toLocaleString("en-US")} bytes.`,
  );
  lines.push(
    `Backend readiness (planted-diagnostic gates, all required before timing): ${Object.entries(
      data.backend.vize,
    )
      .map(([gate, ok]) => `${gate}=${ok ? "pass" : "FAIL"}`)
      .join(" ")}`,
  );
  lines.push(`Budget: ${data.budget.status}`);
  lines.push("");
  for (const [engineClass, label] of Object.entries(ENGINE_CLASSES)) {
    const rows = data.rows.filter((row) => row.engineClass === engineClass);
    lines.push(`### ${label}`);
    lines.push("");
    lines.push("| Row | Cold start | Warmed median | Diagnostics | Measured runs |");
    lines.push("| --- | ---: | ---: | ---: | --- |");
    for (const row of rows) {
      lines.push(
        `| ${row.label} | ${formatMs(row.coldMs)} | ${formatMs(row.medianMs)} | ${row.diagnosticCount} | ${row.runs.map(formatMs).join(", ")} |`,
      );
    }
    if (rows.length === 0) {
      lines.push(`| (${data.skipped[engineClass] ?? "no rows"}) | n/a | n/a | n/a | n/a |`);
    }
    lines.push("");
  }
  lines.push(
    "Engine classes are ranked separately: a cross-class ratio measures TypeScript's native rewrite as much as the Vue layer, so it is reported as context only.",
  );
  return `${lines.join("\n")}\n`;
}

/**
 * Per-SFC projection report for the corpus pair: the bytes each mapper
 * returned for every SFC (from the byte probe's records) and the time tsc
 * spent checking that SFC's projection (`checkSourceFile` in the probe run's
 * `--generateTrace`), for Vize base, Vize head and Vue's mapper.
 */

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";

export type FileCost = { bytes: number; checkMs: number | null };
export type ProjectionRow = { file: string; base: FileCost; head: FileCost; vue: FileCost };
export type ProjectionReport = {
  corpus: string;
  files: number;
  totals: { base: FileCost; head: FileCost; vue: FileCost };
  headAtMostVue: number;
  rows: ProjectionRow[];
};

/** Bytes per SFC, relative to `root`: the last response per file wins, as in `summarizeProbe`. */
export function probeBytesByFile(recordText: string, root: string): Map<string, number> {
  const bytes = new Map<string, number>();
  for (const line of recordText.split("\n")) {
    if (!line.trim()) continue;
    const record = JSON.parse(line) as { fileName: string; bytes?: number };
    if (record.bytes != null) bytes.set(relative(root, record.fileName), record.bytes);
  }
  return bytes;
}

type TraceEvent = {
  ph: string;
  name: string;
  ts: number;
  dur?: number;
  pid?: number;
  tid?: number;
  args?: { path?: string };
};

/** Milliseconds of `checkSourceFile` per file under `root`, summed over every trace in `dir`. */
export function checkMsByFile(dir: string, root: string): Map<string, number> {
  const total = new Map<string, number>();
  const add = (path: string | undefined, us: number) => {
    if (!path) return;
    const file = relative(root, path);
    if (file.startsWith("..")) return;
    total.set(file, (total.get(file) ?? 0) + us / 1000);
  };
  if (!existsSync(dir)) return total;
  for (const name of readdirSync(dir).filter((entry) => /^trace.*\.json$/u.test(entry))) {
    const events = JSON.parse(readFileSync(join(dir, name), "utf8")) as TraceEvent[];
    const open = new Map<string, TraceEvent[]>();
    for (const event of events) {
      if (event.name !== "checkSourceFile") continue;
      if (event.ph === "X") add(event.args?.path, event.dur ?? 0);
      const thread = `${event.pid}:${event.tid}`;
      if (event.ph === "B") open.set(thread, [...(open.get(thread) ?? []), event]);
      if (event.ph === "E") {
        const stack = open.get(thread) ?? [];
        const begin = stack.pop();
        if (begin) add(begin.args?.path, event.ts - begin.ts);
      }
    }
  }
  return total;
}

const cost = (bytes: Map<string, number>, check: Map<string, number>, file: string): FileCost => ({
  bytes: bytes.get(file) ?? 0,
  checkMs: check.size > 0 ? (check.get(file) ?? 0) : null,
});

export function projectionReport(
  corpus: string,
  sources: Record<
    "base" | "head" | "vue",
    { bytes: Map<string, number>; check: Map<string, number> }
  >,
): ProjectionReport {
  const files = [...sources.head.bytes.keys()].sort();
  const rows = files.map((file) => ({
    file,
    base: cost(sources.base.bytes, sources.base.check, file),
    head: cost(sources.head.bytes, sources.head.check, file),
    vue: cost(sources.vue.bytes, sources.vue.check, file),
  }));
  const sum = (side: "base" | "head" | "vue"): FileCost => ({
    bytes: rows.reduce((value, row) => value + row[side].bytes, 0),
    checkMs:
      sources[side].check.size > 0
        ? rows.reduce((value, row) => value + (row[side].checkMs ?? 0), 0)
        : null,
  });
  return {
    corpus,
    files: rows.length,
    totals: { base: sum("base"), head: sum("head"), vue: sum("vue") },
    headAtMostVue: rows.filter((row) => row.head.bytes <= row.vue.bytes).length,
    rows,
  };
}

const kb = (bytes: number) => (bytes / 1024).toFixed(1);
const ms = (value: number | null) => (value == null ? "n/a" : value.toFixed(1));
const ratio = (value: number, of: number) => (of > 0 ? `${(value / of).toFixed(2)}×` : "n/a");

function table(title: string, rows: ProjectionRow[]): string[] {
  return [
    `#### ${title}`,
    "",
    "| SFC | Vize head KB | Vue KB | bytes vs Vue | Vize head check ms | Vue check ms | Vize base check ms |",
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ...rows.map(
      (row) =>
        `| \`${row.file}\` | ${kb(row.head.bytes)} | ${kb(row.vue.bytes)} | ${ratio(row.head.bytes, row.vue.bytes)} | ${ms(row.head.checkMs)} | ${ms(row.vue.checkMs)} | ${ms(row.base.checkMs)} |`,
    ),
    "",
  ];
}

export function renderProjection(report: ProjectionReport, top = 15): string {
  const { totals } = report;
  const by = (key: (row: ProjectionRow) => number) =>
    [...report.rows].sort((a, b) => key(b) - key(a)).slice(0, top);
  return [
    `### Projection per SFC: ${report.corpus}`,
    "",
    `Vize head projects ${kb(totals.head.bytes)} KB over ${report.files} SFCs (${ratio(totals.head.bytes, totals.vue.bytes)} Vue's ${kb(totals.vue.bytes)} KB; base ${kb(totals.base.bytes)} KB). ${report.headAtMostVue} of ${report.files} SFCs are at most Vue's size. tsc spends ${ms(totals.head.checkMs)} ms in checkSourceFile on Vize head projections, ${ms(totals.vue.checkMs)} ms on Vue's and ${ms(totals.base.checkMs)} ms on Vize base's (untimed traced run, summed over checker threads).`,
    "",
    ...table(
      "Largest Vize projections",
      by((row) => row.head.bytes),
    ),
    ...table(
      "Slowest Vize projections to check",
      by((row) => row.head.checkMs ?? 0),
    ),
    ...table(
      "Largest check-time growth, base to head",
      by((row) => (row.head.checkMs ?? 0) - (row.base.checkMs ?? 0)),
    ),
  ].join("\n");
}

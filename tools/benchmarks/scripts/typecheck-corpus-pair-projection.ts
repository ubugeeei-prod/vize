/**
 * Per-SFC projection report for the corpus pair: the bytes each mapper
 * returned for every SFC (from the byte probe's records) and the time tsc
 * spent checking that SFC's projection (`checkSourceFile` in the probe run's
 * `--generateTrace`), for Vize base, Vize head and Vue's mapper.
 */

import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { assertExactDiagnosticFingerprints } from "./typecheck-corpus-pair-parse.ts";

export type ProbeFile = { bytes: number; sourceBytes: number; templateBytes: number };
export type FileCost = { bytes: number; checkMs: number | null };
export type ProjectionRow = {
  file: string;
  sourceBytes: number;
  templateBytes: number;
  base: FileCost;
  head: FileCost;
  vue: FileCost;
};
export type ProjectionReport = {
  corpus: string;
  files: number;
  authored: { sourceBytes: number; templateBytes: number };
  totals: { base: FileCost; head: FileCost; vue: FileCost };
  headAtMostVue: number;
  violations: string[];
  checkers: { base: number[]; head: number[]; vue: number[] };
  rows: ProjectionRow[];
};

/** Bytes per SFC, relative to `root`: the last response per file wins, as in `summarizeProbe`. */
export function probeBytesByFile(recordText: string, root: string): Map<string, ProbeFile> {
  const bytes = new Map<string, ProbeFile>();
  for (const line of recordText.split("\n")) {
    if (!line.trim()) continue;
    const record = JSON.parse(line) as { fileName: string } & Partial<ProbeFile>;
    const file = relative(root, record.fileName);
    assert.ok(!file.startsWith("../") && file.endsWith(".vue"), `unexpected probe file: ${file}`);
    for (const key of ["bytes", "sourceBytes", "templateBytes"] as const)
      assert.ok(Number.isSafeInteger(record[key]) && record[key]! >= 0, `${file}: invalid ${key}`);
    const sizes = {
      bytes: record.bytes!,
      sourceBytes: record.sourceBytes!,
      templateBytes: record.templateBytes!,
    };
    if (bytes.has(file))
      assert.deepEqual(sizes, bytes.get(file), `unstable repeated projection: ${file}`);
    bytes.set(file, sizes);
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

/**
 * Busy milliseconds of each checker thread (every `checkSourceFile`, lib files
 * included), largest first. Check time is bounded by the busiest thread, so a
 * change that cuts total work can still lengthen it by unbalancing them.
 */
export function checkerBusyMs(dir: string): number[] {
  const busy = new Map<string, number>();
  if (!existsSync(dir)) return [];
  for (const name of readdirSync(dir).filter((entry) => /^trace.*\.json$/u.test(entry))) {
    const events = JSON.parse(readFileSync(join(dir, name), "utf8")) as TraceEvent[];
    const open = new Map<string, number[]>();
    for (const event of events) {
      if (event.name !== "checkSourceFile") continue;
      const thread = `${name}:${event.pid}:${event.tid}`;
      const add = (us: number) => busy.set(thread, (busy.get(thread) ?? 0) + us / 1000);
      if (event.ph === "X") add(event.dur ?? 0);
      if (event.ph === "B") open.set(thread, [...(open.get(thread) ?? []), event.ts]);
      if (event.ph === "E") {
        const begin = open.get(thread)?.pop();
        if (begin != null) add(event.ts - begin);
      }
    }
  }
  return [...busy.values()].sort((a, b) => b - a);
}

const cost = (
  bytes: Map<string, ProbeFile>,
  check: Map<string, number>,
  file: string,
): FileCost => ({
  bytes: bytes.get(file)!.bytes,
  checkMs: check.size > 0 ? (check.get(file) ?? 0) : null,
});

export function projectionReport(
  corpus: string,
  sources: Record<
    "base" | "head" | "vue",
    { bytes: Map<string, ProbeFile>; check: Map<string, number>; busy?: number[] }
  >,
): ProjectionReport {
  const files = [...sources.head.bytes.keys()].sort();
  assert.deepEqual([...sources.base.bytes.keys()].sort(), files, "base/head SFC set differs");
  assert.deepEqual([...sources.vue.bytes.keys()].sort(), files, "Vue/head SFC set differs");
  const rows = files.map((file) => {
    const context = ({ sourceBytes, templateBytes }: ProbeFile) => ({
      sourceBytes,
      templateBytes,
    });
    const authored = context(sources.head.bytes.get(file)!);
    assert.deepEqual(
      context(sources.base.bytes.get(file)!),
      authored,
      `${file}: base input differs`,
    );
    assert.deepEqual(context(sources.vue.bytes.get(file)!), authored, `${file}: Vue input differs`);
    return {
      file,
      ...authored,
      base: cost(sources.base.bytes, sources.base.check, file),
      head: cost(sources.head.bytes, sources.head.check, file),
      vue: cost(sources.vue.bytes, sources.vue.check, file),
    };
  });
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
    authored: {
      sourceBytes: rows.reduce((value, row) => value + row.sourceBytes, 0),
      templateBytes: rows.reduce((value, row) => value + row.templateBytes, 0),
    },
    totals: { base: sum("base"), head: sum("head"), vue: sum("vue") },
    headAtMostVue: rows.filter((row) => row.head.bytes <= row.vue.bytes).length,
    violations: rows.filter((row) => row.head.bytes > row.vue.bytes).map((row) => row.file),
    checkers: {
      base: sources.base.busy ?? [],
      head: sources.head.busy ?? [],
      vue: sources.vue.busy ?? [],
    },
    rows,
  };
}

export function assertProjectionBudget(reports: ProjectionReport[]): void {
  const violations = reports.flatMap((report) =>
    report.violations.map((file) => `${report.corpus}/${file}`),
  );
  assert.equal(violations.length, 0, `Vize head exceeds Vue for ${violations.join(", ")}`);
}

export function assertProbeDiagnosticIdentity(
  probes: ReadonlyMap<string, { fingerprint: string }>,
  corpora: readonly { id: string }[],
  hosts: readonly { id: string }[],
): void {
  assertExactDiagnosticFingerprints(
    corpora.flatMap((corpus) =>
      hosts.map((host) => {
        const label = `${corpus.id}-${host.id}`;
        const base = probes.get(`${label}-vize-base`);
        const head = probes.get(`${label}-vize-head`);
        if (!base || !head) throw new Error(`${label}: missing Vize probe`);
        return { label, base: base.fingerprint, head: head.fingerprint };
      }),
    ),
  );
}

const kb = (bytes: number) => (bytes / 1024).toFixed(1);
const ms = (value: number | null) => (value == null ? "n/a" : value.toFixed(1));
const ratio = (value: number, of: number) => (of > 0 ? `${(value / of).toFixed(2)}×` : "n/a");

function table(title: string, rows: ProjectionRow[]): string[] {
  return [
    `#### ${title}`,
    "",
    "| SFC | authored SFC KB | template KB | Vize base KB | Vize head KB | Vue KB | head / Vue | head / authored | Vize base check ms | Vize head check ms | Vue check ms |",
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ...rows.map(
      (row) =>
        `| \`${row.file}\` | ${kb(row.sourceBytes)} | ${kb(row.templateBytes)} | ${kb(row.base.bytes)} | ${kb(row.head.bytes)} | ${kb(row.vue.bytes)} | ${ratio(row.head.bytes, row.vue.bytes)} | ${ratio(row.head.bytes, row.sourceBytes)} | ${ms(row.base.checkMs)} | ${ms(row.head.checkMs)} | ${ms(row.vue.checkMs)} |`,
    ),
    "",
  ];
}

const busy = (threads: number[]) =>
  threads.length > 0 ? threads.map((value) => value.toFixed(0)).join(" / ") : "n/a";

export function renderProjection(report: ProjectionReport, top = 15): string {
  const { authored, totals } = report;
  const by = (key: (row: ProjectionRow) => number) =>
    [...report.rows].sort((a, b) => key(b) - key(a)).slice(0, top);
  return [
    `### Projection per SFC: ${report.corpus}`,
    "",
    `Authored SFCs total ${kb(authored.sourceBytes)} KB, including ${kb(authored.templateBytes)} KB of template content. Vize head projects ${kb(totals.head.bytes)} KB over ${report.files} SFCs (${ratio(totals.head.bytes, totals.vue.bytes)} Vue's ${kb(totals.vue.bytes)} KB; Vize base ${kb(totals.base.bytes)} KB; ${ratio(totals.head.bytes, authored.sourceBytes)} authored SFC bytes). ${report.headAtMostVue} of ${report.files} SFCs are at most Vue's size. tsc spends ${ms(totals.head.checkMs)} ms in checkSourceFile on Vize head projections, ${ms(totals.vue.checkMs)} ms on Vue's and ${ms(totals.base.checkMs)} ms on Vize base's (untimed traced run, summed over checker threads).`,
    "",
    `Checker thread busy ms, busiest first: Vize base ${busy(report.checkers.base)}; Vize head ${busy(report.checkers.head)}; Vue ${busy(report.checkers.vue)}.`,
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

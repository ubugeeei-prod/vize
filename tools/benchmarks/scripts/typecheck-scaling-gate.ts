/**
 * Correctness gate of the type-check scaling lane. A tool's run is timed only
 * when it reports exactly the planted errors: every plant line carries the
 * tool's expected number of errors with an allowed code, and no error lands
 * anywhere else. A run that checked nothing reports no plants and is refused.
 */

import { diagnosticFingerprint, normalizeTypecheckResult } from "./typecheck-command.mjs";
import type { Plant } from "./typecheck-scaling-corpus.ts";
import type { PlantKind } from "./typecheck-scaling-sfc.ts";

/** [file, kind, line, column, code, message], as typecheck-command.mjs normalizes it. */
export type Diagnostic = [string, string, string, string, string, string];
export type StatEntry = [key: string, value: string];
export type ProcessOutput = { status: number | null; stdout: string; stderr: string };

/** Codes each plant may raise; TS2769 is the overload form of an emit call mismatch. */
export const PLANT_CODES: Record<PlantKind, readonly string[]> = {
  script: ["TS2322"],
  "emit-script": ["TS2345", "TS2769"],
  generic: ["TS2322"],
  slot: ["TS2339"],
  "emit-template": ["TS2322", "TS2345"],
  "v-model": ["TS2322"],
  global: ["TS2322"],
};

/**
 * Errors per plant where a tool legitimately reports more than one, keyed by
 * tool id. vue-tsc checks `v-model` both as the prop and as the assignment in
 * the generated `onUpdate:modelValue` handler.
 */
export const PLANT_MULTIPLICITY: Record<string, Partial<Record<PlantKind, number>>> = {};

const STATS_START = /^Files:\s+\d+$/u;
const STAT_LINE = /^(\S.*?):\s+(\d[\d.,]*\s?[A-Za-z%]*)$/u;
const DIAGNOSTIC_LINE = /\(\d+,\d+\): (?:error|warning) TS\d+:/u;

/** Split `--extendedDiagnostics` statistics (from the `Files:` line on) off the diagnostics. */
export function splitStats(stdout: string): { text: string; stats: StatEntry[] } {
  const lines = stdout.split(/\r?\n/u);
  const start = lines.findIndex((line) => STATS_START.test(line.trim()));
  if (start < 0) return { text: stdout, stats: [] };
  const stats: StatEntry[] = [];
  for (const line of lines.slice(start)) {
    if (DIAGNOSTIC_LINE.test(line)) throw new Error(`diagnostic after the statistics: ${line}`);
    const match = STAT_LINE.exec(line.trim());
    if (match) stats.push([match[1] as string, match[2] as string]);
  }
  return { text: lines.slice(0, start).join("\n"), stats };
}

/** First statistic whose key is one of `keys`, as a number (seconds for `…s`). */
export function statNumber(stats: StatEntry[], keys: readonly string[]): number | null {
  for (const key of keys) {
    const entry = stats.find(([name]) => name === key);
    if (!entry) continue;
    const value = Number(entry[1].replaceAll(",", "").replace(/\s?[A-Za-z%]+$/u, ""));
    if (Number.isFinite(value)) return value;
  }
  return null;
}

export const STAT_KEYS = {
  types: ["Types"],
  instantiations: ["Instantiations"],
  checkTimeS: ["Check time"],
} as const;

/** Diagnostics of one run, normalized and sorted, plus its statistics. */
export function parseRun(
  output: ProcessOutput,
  cwd: string,
  format: "json" | "text",
): { diagnostics: Diagnostic[]; fingerprint: string; stats: StatEntry[] } {
  const { text, stats } =
    format === "text" ? splitStats(output.stdout) : { text: output.stdout, stats: [] };
  const report = normalizeTypecheckResult(
    { status: output.status, stdout: text, stderr: output.stderr },
    cwd,
    format,
  ) as unknown as { diagnostics: Diagnostic[] };
  return {
    diagnostics: report.diagnostics,
    fingerprint: diagnosticFingerprint(report.diagnostics) as string,
    stats,
  };
}

export type GateResult = {
  ok: boolean;
  errors: number;
  warnings: number;
  expected: number;
  missing: string[];
  unexpected: string[];
};

const key = (file: string, line: number | string) => `${file}:${line}`;

/** Compare a run's errors with the plants; warnings are counted, never matched. */
export function gate(toolId: string, diagnostics: Diagnostic[], plants: Plant[]): GateResult {
  const errors = diagnostics.filter((d) => d[1] === "error");
  const byLine = new Map<string, Diagnostic[]>();
  for (const diagnostic of errors) {
    const at = key(diagnostic[0], diagnostic[2]);
    byLine.set(at, [...(byLine.get(at) ?? []), diagnostic]);
  }
  const multiplicity = PLANT_MULTIPLICITY[toolId] ?? {};
  const missing: string[] = [];
  const unexpected: string[] = [];
  let expected = 0;
  const planted = new Set<string>();
  for (const plant of plants) {
    const at = key(plant.file, plant.line);
    if (planted.has(at)) throw new Error(`two plants share ${at}`);
    planted.add(at);
    const want = multiplicity[plant.kind] ?? 1;
    expected += want;
    const found = byLine.get(at) ?? [];
    const allowed = found.filter((d) => PLANT_CODES[plant.kind].includes(d[4]));
    if (allowed.length < want) {
      missing.push(
        `${at} ${plant.kind}: want ${want} × ${PLANT_CODES[plant.kind].join("|")}, got ${found.map((d) => d[4]).join(",") || "none"}`,
      );
    }
    for (const extra of [...allowed.slice(want), ...found.filter((d) => !allowed.includes(d))]) {
      unexpected.push(`${at} ${extra[4]} ${extra[5].split("\n")[0]} (on the ${plant.kind} plant)`);
    }
  }
  for (const [at, found] of byLine) {
    if (planted.has(at)) continue;
    for (const d of found) unexpected.push(`${at}:${d[3]} ${d[4]} ${d[5].split("\n")[0]}`);
  }
  return {
    ok: missing.length === 0 && unexpected.length === 0,
    errors: errors.length,
    warnings: diagnostics.length - errors.length,
    expected,
    missing,
    unexpected,
  };
}

/** One-paragraph refusal reason, capped so a broken run cannot flood the summary. */
export function refusalText(toolId: string, result: GateResult, limit = 8): string {
  const lines = [
    `${toolId}: ${result.errors} errors, expected ${result.expected} planted (${result.missing.length} plants missed, ${result.unexpected.length} unexpected)`,
    ...result.missing.slice(0, limit).map((line) => `  missing ${line}`),
    ...result.unexpected.slice(0, limit).map((line) => `  unexpected ${line}`),
  ];
  return lines.join("\n");
}

/**
 * Pure parsing for the real-corpus type-check pair: host output with
 * `--extendedDiagnostics`, GNU time records, content-mapper frames and probe
 * records. Anything outside the expected shapes is an error.
 */

import { diagnosticFingerprint, normalizeTypecheckResult } from "./typecheck-command.mjs";

/** [file, kind, line, column, code, message], as typecheck-command.mjs normalizes it. */
export type Diagnostic = [string, string, string, string, string, string];
export type StatEntry = [key: string, value: string];
export type HostResult = { status: number | null; stdout: string; stderr: string };

const STATS_START = /^Files:\s+\d+$/u;
const STAT_LINE = /^(\S.*?):\s+(\d[\d.]*[A-Za-z]*)$/u;

/** Diagnostics come first; the statistics block starts at `Files:` and runs to the end. */
export function splitHostOutput(stdout: string): { diagnosticText: string; stats: StatEntry[] } {
  const lines = stdout.split(/\r?\n/u);
  const start = lines.findIndex((line) => STATS_START.test(line));
  if (start < 0) throw new Error("host output has no --extendedDiagnostics block");
  const stats: StatEntry[] = [];
  for (const line of lines.slice(start)) {
    if (!line.trim()) continue;
    const match = STAT_LINE.exec(line);
    if (!match) throw new Error(`unexpected --extendedDiagnostics line: ${line}`);
    stats.push([match[1] as string, match[2] as string]);
  }
  return { diagnosticText: lines.slice(0, start).join("\n"), stats };
}

const UNITS = {
  count: /^(\d+)$/u,
  seconds: /^(\d+(?:\.\d+)?)s$/u,
  kilobytes: /^(\d+)K$/u,
} as const;

/** One statistic by exact key; missing, repeated or malformed values are errors. */
export function statValue(stats: StatEntry[], key: string, unit: keyof typeof UNITS): number {
  const values = stats.filter(([name]) => name === key);
  if (values.length !== 1) throw new Error(`statistic "${key}" occurs ${values.length} times`);
  const match = UNITS[unit].exec((values[0] as StatEntry)[1]);
  if (!match)
    throw new Error(`statistic "${key}" is not in ${unit}: ${(values[0] as StatEntry)[1]}`);
  return Number(match[1]);
}

/** Diagnostics and statistics of one host run, normalized and sorted. */
export function parseHostRun(
  result: HostResult,
  cwd: string,
): { diagnostics: Diagnostic[]; fingerprint: string; stats: StatEntry[] } {
  const { diagnosticText, stats } = splitHostOutput(result.stdout);
  const report = normalizeTypecheckResult(
    { status: result.status, stdout: diagnosticText, stderr: result.stderr },
    cwd,
    "text",
  ) as unknown as { diagnostics: Diagnostic[] };
  return {
    diagnostics: report.diagnostics,
    fingerprint: diagnosticFingerprint(report.diagnostics) as string,
    stats,
  };
}

/** `-f` for /usr/bin/time: one tagged line, read back by parseGnuTime. */
export const GNU_TIME_FORMAT = "vize-time wall=%e user=%U sys=%S maxrss_kb=%M status=%x";
const GNU_TIME_LINE =
  /^vize-time wall=(\d+\.\d+) user=(\d+\.\d+) sys=(\d+\.\d+) maxrss_kb=(\d+) status=(\d+)$/u;
const GNU_TIME_NOTE = /^Command (?:exited with non-zero status \d+|terminated by signal \d+)$/u;

export type GnuTime = {
  wallS: number;
  userS: number;
  sysS: number;
  maxRssKb: number;
  status: number;
};

export function parseGnuTime(text: string): GnuTime {
  const lines = text.split(/\r?\n/u).filter((line) => line.trim());
  const records = lines.filter((line) => GNU_TIME_LINE.test(line));
  const other = lines.filter((line) => !GNU_TIME_LINE.test(line) && !GNU_TIME_NOTE.test(line));
  if (records.length !== 1 || other.length > 0) {
    throw new Error(`unexpected GNU time output:\n${text}`);
  }
  const [, wall, user, sys, rss, status] = GNU_TIME_LINE.exec(records[0] as string) as string[];
  return {
    wallS: Number(wall),
    userS: Number(user),
    sysS: Number(sys),
    maxRssKb: Number(rss),
    status: Number(status),
  };
}

/** Multiset difference of two sorted diagnostic lists. */
export function diffDiagnostics(
  base: Diagnostic[],
  head: Diagnostic[],
): { added: Diagnostic[]; removed: Diagnostic[] } {
  const counts = new Map<string, number>();
  for (const diagnostic of base) {
    const key = JSON.stringify(diagnostic);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  const added: Diagnostic[] = [];
  for (const diagnostic of head) {
    const key = JSON.stringify(diagnostic);
    const left = counts.get(key) ?? 0;
    if (left > 0) counts.set(key, left - 1);
    else added.push(diagnostic);
  }
  const removed: Diagnostic[] = [];
  for (const diagnostic of base) {
    const key = JSON.stringify(diagnostic);
    const left = counts.get(key) ?? 0;
    if (left > 0) {
      removed.push(diagnostic);
      counts.set(key, left - 1);
    }
  }
  return { added, removed };
}

export function median(values: number[]): number {
  if (values.length === 0) throw new Error("median of no values");
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? (sorted[mid] as number)
    : ((sorted[mid - 1] as number) + (sorted[mid] as number)) / 2;
}

/** Even runs keep the order, odd runs reverse it, so no variant always runs first. */
export function runOrder<T>(variants: readonly T[], run: number): T[] {
  return run % 2 === 0 ? [...variants] : [...variants].reverse();
}

/** Fail when any paired base/head diagnostic fingerprint differs. */
export function assertExactDiagnosticFingerprints(
  pairs: Array<{ label: string; base: string; head: string }>,
): void {
  const changed = pairs.filter((pair) => pair.base !== pair.head).map((pair) => pair.label);
  if (changed.length > 0) {
    throw new Error(`base/head diagnostic fingerprint changed: ${changed.join(", ")}`);
  }
}

/** Incremental reader of `Content-Length` framed JSON-RPC messages. */
export function frameReader(): (chunk: Buffer) => Record<string, unknown>[] {
  let buffer = Buffer.alloc(0);
  return (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    const messages: Record<string, unknown>[] = [];
    for (;;) {
      const end = buffer.indexOf("\r\n\r\n");
      if (end < 0) return messages;
      const header = buffer.subarray(0, end).toString("ascii");
      const length = /^Content-Length: *(\d+)$/imu.exec(header)?.[1];
      if (length == null) throw new Error(`frame header without Content-Length: ${header}`);
      const start = end + 4;
      if (buffer.length < start + Number(length)) return messages;
      const body: unknown = JSON.parse(buffer.subarray(start, start + Number(length)).toString());
      if (body === null || typeof body !== "object" || Array.isArray(body)) {
        throw new Error("frame body is not a JSON-RPC object");
      }
      messages.push(body as Record<string, unknown>);
      buffer = buffer.subarray(start + Number(length));
    }
  };
}

/** Reply size and tuple counts; the host decodes every span and directive tuple. */
export type ReplyShape = {
  /** UTF-8 bytes of the reply's `result`, serialized again. */
  replyBytes: number;
  spans: number;
  directives: number;
  /** Directives with no original range and the ignore policy: unmapped gaps. */
  ignoreGaps: number;
};

/** One probe line per transform, including authored and returned UTF-8 bytes. */
export type ProbeRecord =
  | ({
      pid: number;
      fileName: string;
      bytes: number;
      sourceBytes: number;
      templateBytes: number;
    } & Partial<ReplyShape>)
  | { pid: number; fileName: string; error: string };

const tuples = (value: unknown): unknown[][] =>
  Array.isArray(value) ? value.filter((row): row is unknown[] => Array.isArray(row)) : [];

export function replyShape(result: Record<string, unknown>): ReplyShape {
  const directives = tuples(
    (result.diagnosticDirectives as { directives?: unknown } | undefined)?.directives,
  );
  return {
    replyBytes: Buffer.byteLength(JSON.stringify(result), "utf8"),
    spans: tuples(result.mappings).length,
    directives: directives.length,
    ignoreGaps: directives.filter((row) => row[0] === 0 && row[1] === 0 && row[4] === 0).length,
  };
}

/** Combine authored byte context with the mapper response's exact generated shape. */
export function probeRecord(
  pid: number,
  fileName: string,
  authored: { sourceBytes: number; templateBytes: number },
  response: Record<string, unknown>,
): ProbeRecord {
  if (Object.hasOwn(response, "error")) {
    return { pid, fileName, error: JSON.stringify(response.error) };
  }
  const result = response.result as { text?: unknown } | null | undefined;
  if (typeof result?.text !== "string") {
    return { pid, fileName, error: `transform result has no text: ${JSON.stringify(result)}` };
  }
  return {
    pid,
    fileName,
    bytes: Buffer.byteLength(result.text, "utf8"),
    ...authored,
    ...replyShape(result as Record<string, unknown>),
  };
}

/** Generated bytes and reply shape: the last response per file, summed over distinct files. */
export function summarizeProbe(text: string): {
  files: number;
  transforms: number;
  processes: number;
  bytes: number;
  sourceBytes: number;
  templateBytes: number;
  errors: string[];
} & ReplyShape {
  const records = text
    .split("\n")
    .filter((line) => line.trim())
    .map((line) => JSON.parse(line) as ProbeRecord);
  const last = new Map<string, Extract<ProbeRecord, { bytes: number }>>();
  const errors: string[] = [];
  for (const record of records) {
    if ("error" in record) errors.push(`${record.fileName}: ${record.error}`);
    else last.set(record.fileName, record);
  }
  const sum = (pick: (record: Extract<ProbeRecord, { bytes: number }>) => number | undefined) =>
    [...last.values()].reduce((total, record) => total + (pick(record) ?? 0), 0);
  return {
    files: last.size,
    transforms: records.length,
    processes: new Set(records.map((record) => record.pid)).size,
    bytes: sum((record) => record.bytes),
    sourceBytes: sum((record) => record.sourceBytes),
    templateBytes: sum((record) => record.templateBytes),
    replyBytes: sum((record) => record.replyBytes),
    spans: sum((record) => record.spans),
    directives: sum((record) => record.directives),
    ignoreGaps: sum((record) => record.ignoreGaps),
    errors,
  };
}

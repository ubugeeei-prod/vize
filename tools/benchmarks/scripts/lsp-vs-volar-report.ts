/**
 * Summary and Markdown for `lsp-vs-volar.ts`. Pure over session results so
 * the evidence test can check it without starting a server.
 */

import { REQUEST_KINDS, type RequestKind } from "./lsp-vs-volar-fixtures.ts";
import { median, percentile } from "./lsp-vs-volar-metrics.ts";
import type { SessionResult } from "./lsp-vs-volar-session.ts";

export interface Latency {
  n: number;
  ok: number;
  p50: number | null;
  p99: number | null;
}

export interface EditorSummary {
  editor: string;
  kind: SessionResult["kind"];
  sessions: number;
  cold: {
    initializeMs: number | null;
    openToDiagnosticsMs: number | null;
    totalMs: number | null;
    cpuSeconds: number | null;
    rssMiB: number | null;
    pssMiB: number | null;
    processes: number | null;
    ok: number;
  };
  warm: {
    cpuSeconds: number | null;
    cpuSecondsPerEdit: number | null;
    peakRssMiB: number | null;
    peakPssMiB: number | null;
    endPssMiB: number | null;
    idleCpuSeconds: number | null;
    /** First session's processes after idle, largest first: `name memory`. */
    endMembers: string;
  };
  diagnostics: { script: Latency; template: Latency; repair: Latency };
  requests: Record<RequestKind, Latency>;
  failures: string[];
}

export const EDITOR_COLD_SPEEDUP_TARGET = 100;
export const EDITOR_INTERACTIVE_SPEEDUP_TARGET = 10;
export const EDITOR_MEMORY_REDUCTION_TARGET = 10;

export interface OperationPerformanceTarget {
  operation: string;
  accurate: boolean;
  p50Speedup: number | null;
  p99Speedup: number | null;
  met: boolean;
}

export interface EditorPerformanceTarget {
  editor: string;
  reference: string;
  accurate: boolean;
  coldSpeedup: number | null;
  coldMemoryReduction: number | null;
  warmMemoryReduction: number | null;
  operations: OperationPerformanceTarget[];
  met: boolean;
  missing: string[];
}

const mib = (bytes: number | null | undefined) => (bytes == null ? null : bytes / 1048576);

function latency(samples: Array<{ ms: number; ok: boolean }>): Latency {
  // Only correct answers count toward latency; a fast wrong one is a failure.
  const correct = samples.filter((sample) => sample.ok).map((sample) => sample.ms);
  return {
    n: samples.length,
    ok: correct.length,
    p50: percentile(correct, 50),
    p99: percentile(correct, 99),
  };
}

function med(values: Array<number | null>): number | null {
  return median(values.filter((value): value is number => value != null));
}

export function summarize(sessions: SessionResult[]): EditorSummary[] {
  const labels = [...new Set(sessions.map((session) => session.editor))];
  return labels.map((label) => {
    const mine = sessions.filter((session) => session.editor === label);
    const edits = mine.flatMap((session) => session.edits.filter((edit) => edit.cycle > 0));
    const requests = mine.flatMap((session) => session.requests);
    return {
      editor: label,
      kind: mine[0].kind,
      sessions: mine.length,
      cold: {
        initializeMs: med(mine.map((session) => session.cold.initializeMs)),
        openToDiagnosticsMs: med(mine.map((session) => session.cold.openToDiagnosticsMs)),
        totalMs: med(mine.map((session) => session.cold.totalMs)),
        cpuSeconds: med(mine.map((session) => session.cold.reading.cpuSeconds)),
        rssMiB: med(mine.map((session) => mib(session.cold.reading.rssBytes))),
        pssMiB: med(mine.map((session) => mib(session.cold.reading.pssBytes))),
        processes: med(mine.map((session) => session.cold.reading.processes)),
        ok: mine.filter((session) => session.cold.ok).length,
      },
      warm: {
        cpuSeconds: med(mine.map((session) => session.warm.cpuSeconds)),
        cpuSecondsPerEdit: med(edits.map((edit) => edit.cpuSeconds)),
        peakRssMiB: med(mine.map((session) => mib(session.warm.peakRssBytes))),
        peakPssMiB: med(mine.map((session) => mib(session.warm.peakPssBytes))),
        endPssMiB: med(mine.map((session) => mib(session.idle.end.pssBytes))),
        idleCpuSeconds: med(mine.map((session) => session.idle.cpuSeconds)),
        endMembers: (mine[0].idle.end.members ?? [])
          .map((member) => ({ name: member.name, bytes: member.pssBytes ?? member.rssBytes }))
          .sort((left, right) => right.bytes - left.bytes)
          .map((member) => `${member.name} ${Math.round(member.bytes / 1048576)}`)
          .join(", "),
      },
      diagnostics: {
        script: latency(edits.filter((edit) => edit.probe === "script" && edit.phase === "break")),
        template: latency(
          edits.filter((edit) => edit.probe === "template" && edit.phase === "break"),
        ),
        repair: latency(edits.filter((edit) => edit.phase === "repair")),
      },
      requests: Object.fromEntries(
        REQUEST_KINDS.map((kind) => [
          kind,
          latency(requests.filter((request) => request.kind === kind)),
        ]),
      ) as Record<RequestKind, Latency>,
      failures: mine.flatMap((session) =>
        session.failures.map((failure) => `round ${session.round}: ${failure}`),
      ),
    };
  });
}

const divide = (numerator: number | null, denominator: number | null): number | null =>
  numerator != null && denominator != null && denominator > 0 ? numerator / denominator : null;

const operationLatencies = (summary: EditorSummary): Array<[string, Latency]> => [
  ["diagnostics after script edit", summary.diagnostics.script],
  ["diagnostics after template edit", summary.diagnostics.template],
  ["diagnostics cleared after repair", summary.diagnostics.repair],
  ...REQUEST_KINDS.map((kind) => [kind, summary.requests[kind]] as [string, Latency]),
];

/**
 * Compare every Vize row with the pinned Vue language-tools row on the same
 * fixture. Missing or incorrect samples are target misses, never fast results.
 */
export function performanceTargets(summaries: EditorSummary[]): EditorPerformanceTarget[] {
  const reference = summaries.find((summary) => summary.kind === "volar");
  if (!reference) return [];
  return summaries
    .filter((summary) => summary.kind === "vize")
    .map((candidate) => {
      const referenceOperations = new Map(operationLatencies(reference));
      const operations = operationLatencies(candidate).map(([operation, latency]) => {
        const baseline = referenceOperations.get(operation)!;
        const accurate =
          latency.n > 0 && latency.ok === latency.n && baseline.n > 0 && baseline.ok === baseline.n;
        const p50Speedup = divide(baseline.p50, latency.p50);
        const p99Speedup = divide(baseline.p99, latency.p99);
        return {
          operation,
          accurate,
          p50Speedup,
          p99Speedup,
          met:
            accurate &&
            (p50Speedup ?? 0) >= EDITOR_INTERACTIVE_SPEEDUP_TARGET &&
            (p99Speedup ?? 0) >= EDITOR_INTERACTIVE_SPEEDUP_TARGET,
        };
      });
      const accurate =
        candidate.failures.length === 0 &&
        reference.failures.length === 0 &&
        candidate.cold.ok === candidate.sessions &&
        reference.cold.ok === reference.sessions &&
        operations.every((operation) => operation.accurate);
      const coldSpeedup = divide(reference.cold.totalMs, candidate.cold.totalMs);
      const coldMemoryReduction = divide(
        reference.cold.pssMiB ?? reference.cold.rssMiB,
        candidate.cold.pssMiB ?? candidate.cold.rssMiB,
      );
      const warmMemoryReduction = divide(
        reference.warm.peakPssMiB ?? reference.warm.peakRssMiB,
        candidate.warm.peakPssMiB ?? candidate.warm.peakRssMiB,
      );
      const missing: string[] = [];
      if (!accurate) missing.push("accuracy");
      if ((coldSpeedup ?? 0) < EDITOR_COLD_SPEEDUP_TARGET) missing.push("cold start 100x");
      if ((coldMemoryReduction ?? 0) < EDITOR_MEMORY_REDUCTION_TARGET)
        missing.push("cold memory 10x");
      if ((warmMemoryReduction ?? 0) < EDITOR_MEMORY_REDUCTION_TARGET)
        missing.push("warm memory 10x");
      for (const operation of operations) {
        if (!operation.met) missing.push(`${operation.operation} 10x`);
      }
      return {
        editor: candidate.editor,
        reference: reference.editor,
        accurate,
        coldSpeedup,
        coldMemoryReduction,
        warmMemoryReduction,
        operations,
        met: missing.length === 0,
        missing,
      };
    });
}

const fixed = (value: number | null, digits = 0) => (value == null ? "–" : value.toFixed(digits));
const ms = (value: number | null) =>
  value == null ? "–" : value < 10 ? value.toFixed(1) : value.toFixed(0);
const times = (value: number | null) => (value == null ? "–" : `×${value.toFixed(2)}`);

function cell(value: Latency): string {
  const counts = value.ok === value.n ? `${value.n}` : `**${value.ok}/${value.n} ok**`;
  return `${ms(value.p50)} / ${ms(value.p99)} (${counts})`;
}

function table(header: string[], rows: string[][]): string {
  return [
    `| ${header.join(" | ")} |`,
    `| ${header.map((_, index) => (index === 0 ? "---" : "---:")).join(" | ")} |`,
    ...rows.map((row) => `| ${row.join(" | ")} |`),
  ].join("\n");
}

export function markdown(
  fixture: string,
  summaries: EditorSummary[],
  sessions: SessionResult[],
  meta: Record<string, string>,
  crashes: string[] = [],
): string {
  const targets = performanceTargets(summaries);
  const names = summaries.map((summary) => summary.editor);
  const column = (pick: (summary: EditorSummary) => string) => summaries.map(pick);
  const lines: string[] = [`## ${fixture}`, ""];
  const described = Object.entries(meta).map(([key, value]) => `${key}=${value}`);
  if (described.length) lines.push(`Inputs: ${described.join(", ")}`, "");
  lines.push(
    "Cold start (median over sessions): spawn to initialize, then open to the planted error being visible.",
    "",
    table(
      ["", ...names],
      [
        ["initialize ms", ...column((s) => ms(s.cold.initializeMs))],
        ["open → diagnostics ms", ...column((s) => ms(s.cold.openToDiagnosticsMs))],
        ["total ms", ...column((s) => ms(s.cold.totalMs))],
        ["CPU s", ...column((s) => fixed(s.cold.cpuSeconds, 2))],
        ["RSS MiB", ...column((s) => fixed(s.cold.rssMiB))],
        ["PSS MiB", ...column((s) => fixed(s.cold.pssMiB))],
        ["processes", ...column((s) => fixed(s.cold.processes))],
        ["planted error seen", ...column((s) => `${s.cold.ok}/${s.sessions}`)],
      ],
    ),
    "",
    "Editing session: CPU is user+sys of the whole process tree; PSS splits shared pages.",
    "",
    table(
      ["", ...names],
      [
        ["session CPU s", ...column((s) => fixed(s.warm.cpuSeconds, 2))],
        ["CPU s per edit (median)", ...column((s) => fixed(s.warm.cpuSecondsPerEdit, 3))],
        ["peak RSS MiB", ...column((s) => fixed(s.warm.peakRssMiB))],
        ["peak PSS MiB", ...column((s) => fixed(s.warm.peakPssMiB))],
        ["PSS after idle MiB", ...column((s) => fixed(s.warm.endPssMiB))],
        ["idle CPU s", ...column((s) => fixed(s.warm.idleCpuSeconds, 2))],
        ["MiB by process after idle", ...column((s) => s.warm.endMembers || "–")],
      ],
    ),
    "",
    "Latency ms, p50 / p99 (samples). Only correct answers count; failures are bold.",
    "",
    table(
      ["", ...names],
      [
        ["diagnostics after script edit", ...column((s) => cell(s.diagnostics.script))],
        ["diagnostics after template edit", ...column((s) => cell(s.diagnostics.template))],
        ["diagnostics cleared after repair", ...column((s) => cell(s.diagnostics.repair))],
        ...REQUEST_KINDS.map((kind) => [kind, ...column((s) => cell(s.requests[kind]))]),
      ],
    ),
    "",
  );
  if (targets.length > 0) {
    lines.push(
      "## Performance targets",
      "",
      `Cold start targets ${EDITOR_COLD_SPEEDUP_TARGET}x lower wall time; every interactive p50 and p99 targets ${EDITOR_INTERACTIVE_SPEEDUP_TARGET}x lower latency; cold and warm process-tree memory target ${EDITOR_MEMORY_REDUCTION_TARGET}x lower usage. Both rows must satisfy the same fixture contract for every sample. Missing or incorrect answers are misses, not timings.`,
      "",
      table(
        ["candidate", "accuracy", "cold wall", "cold memory", "warm memory", "verdict"],
        targets.map((target) => [
          target.editor,
          target.accurate ? "exact fixture contract" : "**miss**",
          times(target.coldSpeedup),
          times(target.coldMemoryReduction),
          times(target.warmMemoryReduction),
          target.met ? "**met**" : `miss: ${target.missing.join(", ")}`,
        ]),
      ),
      "",
      table(
        ["candidate", "operation", "accuracy", "p50 / p99 speedup", "verdict"],
        targets.flatMap((target) =>
          target.operations.map((operation) => [
            target.editor,
            operation.operation,
            operation.accurate ? "exact fixture contract" : "**miss**",
            `${times(operation.p50Speedup)} / ${times(operation.p99Speedup)}`,
            operation.met ? "**met**" : "miss",
          ]),
        ),
      ),
      "",
    );
  }
  const first = names.map((name) => sessions.find((session) => session.editor === name));
  const keys = [...new Set(first.flatMap((session) => Object.keys(session?.answers ?? {})))];
  lines.push(
    "<details><summary>First answers (accuracy review)</summary>",
    "",
    table(
      ["request", ...names],
      keys.map((key) => [
        key,
        ...first.map((session) =>
          (session?.answers[key] ?? "–").replace(/\s+/g, " ").replaceAll("|", "\\|").slice(0, 160),
        ),
      ]),
    ),
    "",
    "</details>",
    "",
  );
  const failures = [
    ...crashes.map((crash) => `- ${crash.replace(/\s+/g, " ").slice(0, 400)}`),
    ...summaries.flatMap((summary) =>
      summary.failures.map((failure) => `- ${summary.editor}, ${failure}`),
    ),
  ];
  if (failures.length) lines.push("Failures:", "", ...failures.slice(0, 50), "");
  return lines.join("\n");
}

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

const fixed = (value: number | null, digits = 0) => (value == null ? "–" : value.toFixed(digits));
const ms = (value: number | null) =>
  value == null ? "–" : value < 10 ? value.toFixed(1) : value.toFixed(0);

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

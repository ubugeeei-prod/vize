import type { SessionResult } from "../../../tools/benchmarks/scripts/lsp-vs-volar-session.ts";

/** One recorded session with one script edit of `editMs` and one hover. */
export function session(editor: string, editMs: number, ok: boolean): SessionResult {
  const reading = {
    cpuSeconds: 1,
    rssBytes: 1048576,
    pssBytes: null,
    processes: 2,
    pssExitedProcesses: 0,
    pids: [],
  };
  return {
    editor,
    kind: editor.startsWith("vize") ? "vize" : "volar",
    round: 1,
    cold: { initializeMs: 10, openToDiagnosticsMs: 20, totalMs: 30, ok: true, reading },
    warm: { wallMs: 1, cpuSeconds: 2, peakRssBytes: 2097152, peakPssBytes: null, end: reading },
    idle: { ms: 1, cpuSeconds: 0, end: reading },
    edits: [{ cycle: 1, probe: "script", phase: "break", ms: editMs, ok, cpuSeconds: 0.1 }],
    requests: [{ cycle: 1, kind: "hover", name: "h", ms: 3, ok: true, size: 4 }],
    answers: { "hover: h": "a | b" },
    series: [],
    failures: ok ? [] : ["cycle 1: script break diagnostics never converged"],
  };
}

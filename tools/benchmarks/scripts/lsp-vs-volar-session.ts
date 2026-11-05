/**
 * One scripted editing session of `lsp-vs-volar.ts` against one editor.
 *
 * Cold: spawn, initialize, open the SFC with a planted type error and wait
 * until the error is visible. Warm: cycles of break/repair edits in script and
 * template (each waits for the exact diagnostic to appear or disappear) with a
 * round of feature requests after each repair. Idle: the tree's CPU while
 * nothing is asked. The process tree is sampled throughout.
 */

import { realpathSync } from "node:fs";
import { relative } from "node:path";
import { pathToFileURL } from "node:url";

import type { Diagnostic, Editor, Location } from "./lsp-vs-volar-editors.ts";
import {
  type FixtureSpec,
  type RequestKind,
  type RequestSpec,
  SCRIPT_BROKEN,
  SCRIPT_CLEAN,
  SCRIPT_CODE,
  TEMPLATE_BROKEN,
  TEMPLATE_CLEAN,
  TEMPLATE_CODE,
} from "./lsp-vs-volar-fixtures.ts";
import {
  type TreeReading,
  TreeSampler,
  killSurvivors,
  machineMemory,
  readTree,
} from "./lsp-vs-volar-metrics.ts";
import { OpenDocument } from "./lsp-vs-volar-text.ts";

export interface SessionOptions {
  cycles: number;
  /** Start and the cold open; a large project's first load can take minutes. */
  coldTimeoutMs: number;
  diagnosticsTimeoutMs: number;
  requestTimeoutMs: number;
  /** Warm phase wall budget: later cycles are skipped (and reported) once it is spent. */
  warmBudgetMs: number;
  sampleIntervalMs: number;
  idleMs: number;
  /**
   * Memory the machine keeps available (Linux `MemAvailable`). Below it the
   * session's tree is killed and the session reported with where the memory
   * went, so no server takes the measuring machine down with it.
   */
  memoryReserveBytes: number | null;
}

export interface EditSample {
  cycle: number;
  probe: "script" | "template";
  phase: "break" | "repair";
  ms: number;
  ok: boolean;
  cpuSeconds: number;
}

export interface RequestSample {
  cycle: number;
  kind: RequestKind;
  name: string;
  ms: number;
  ok: boolean;
  /** Answer size: hover characters, items, locations or tokens. */
  size: number;
  error?: string;
}

export interface SessionResult {
  editor: string;
  kind: Editor["kind"];
  round: number;
  cold: {
    initializeMs: number;
    openToDiagnosticsMs: number;
    totalMs: number;
    ok: boolean;
    reading: TreeReading;
  };
  warm: {
    wallMs: number;
    cpuSeconds: number;
    peakRssBytes: number | null;
    peakPssBytes: number | null;
    end: TreeReading;
  };
  idle: { ms: number; cpuSeconds: number; end: TreeReading };
  edits: EditSample[];
  requests: RequestSample[];
  /** First answer per request, for side-by-side accuracy review. */
  answers: Record<string, string>;
  /** `[ms, rssMiB, pssMiB, cpuSeconds]`, at most 400 points. */
  series: Array<[number, number, number | null, number]>;
  failures: string[];
}

const now = () => performance.now();

class Deadline extends Error {}

function within<T>(promise: Promise<T>, ms: number, what: string): Promise<T> {
  let timer: NodeJS.Timeout | undefined;
  return Promise.race([
    promise.finally(() => clearTimeout(timer)),
    new Promise<T>(
      (_, reject) =>
        (timer = setTimeout(() => reject(new Deadline(`${what} exceeded ${ms}ms`)), ms)),
    ),
  ]);
}

/** One spelling per file: a workspace package reached through its symlink is the package. */
function realPath(path: string): string {
  try {
    return realpathSync.native(path);
  } catch (error) {
    // A file that does not exist has no link to resolve.
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return path;
    throw error;
  }
}

function unique(locations: Location[]): Location[] {
  const seen = new Map<string, Location>();
  for (const location of locations) {
    const path = realPath(location.path);
    seen.set(`${path}:${location.line}:${location.character}`, { ...location, path });
  }
  return [...seen.values()];
}

/** Whether the answer is correct, its size, and a short description of it. */
export function judge(
  spec: RequestSpec,
  answer: string | string[] | Location[] | number,
  workspace: string,
): { ok: boolean; size: number; summary: string } {
  switch (spec.kind) {
    case "hover": {
      const text = answer as string;
      return {
        ok: text.length > 0 && spec.expect.every((part) => text.includes(part)),
        size: text.length,
        summary: text.slice(0, 400),
      };
    }
    case "completion": {
      const labels = new Set(answer as string[]);
      const missing = spec.expect.filter((label) => !labels.has(label));
      return {
        ok: missing.length === 0,
        size: labels.size,
        summary: `${labels.size} items${missing.length ? `; missing ${missing.join(", ")}` : ""}`,
      };
    }
    case "definition": {
      const found = unique(answer as Location[]);
      const root = realPath(workspace);
      const paths = [
        ...new Set(found.map((item) => relative(root, item.path).split("\\").join("/"))),
      ];
      return {
        ok: paths.some((path) => path.endsWith(spec.expectPath)),
        size: found.length,
        summary: paths.join(", "),
      };
    }
    case "references":
    case "rename": {
      const found = unique(answer as Location[]);
      const files = new Set(found.map((item) => item.path)).size;
      return {
        ok: found.length >= spec.min,
        size: found.length,
        summary: `${found.length} locations in ${files} files`,
      };
    }
    case "semanticTokens": {
      const count = answer as number;
      return { ok: count >= spec.min, size: count, summary: `${count} tokens` };
    }
  }
}

function hasError(code: string, line: number) {
  return (diagnostics: Diagnostic[]) =>
    diagnostics.some((item) => item.code === code && item.line === line);
}

export async function runSession(
  editor: Editor,
  spec: FixtureSpec,
  workspace: string,
  filePath: string,
  plantedText: string,
  options: SessionOptions,
  round: number,
): Promise<SessionResult> {
  const sampler = new TreeSampler(() => editor.pids(), options.sampleIntervalMs);
  const reserve = options.memoryReserveBytes;
  let starved: Error | null = null;
  let abort: (error: Error) => void = () => {};
  const aborted = new Promise<never>((_, reject) => (abort = reject));
  aborted.catch(() => {});
  sampler.onReading = (reading) => {
    if (reserve == null || starved) return;
    const machine = machineMemory();
    if (!machine || machine.availableBytes >= reserve) return;
    const gib = (value: number) => (value / 1073741824).toFixed(1);
    const tree = reading.pssBytes ?? reading.rssBytes;
    starved = new Error(
      `machine memory available fell to ${gib(machine.availableBytes)} GiB (reserve ${gib(reserve)} GiB): ` +
        `the tree held ${gib(tree)} GiB in ${reading.processes} processes, shmem ${gib(machine.shmemBytes)} GiB, ` +
        `largest outside the tree (MiB): ${machine.largest(new Set(reading.pids)).join(", ")}; the tree was killed`,
    );
    killSurvivors(readTree(reading.pids, false, true));
    abort(starved);
  };
  /** Every wait ends when the tree is killed for the machine's memory. */
  const bounded = <T>(promise: Promise<T>) => Promise.race([promise, aborted]);
  const failures: string[] = [];
  const edits: EditSample[] = [];
  const requests: RequestSample[] = [];
  const answers: Record<string, string> = {};
  const doc = new OpenDocument(filePath, pathToFileURL(filePath).href, plantedText);
  const scriptLine = doc.resolve({ find: SCRIPT_BROKEN }).line;
  const templateLine = doc.resolve({ find: TEMPLATE_CLEAN }).line;
  const scriptError = hasError(SCRIPT_CODE, scriptLine);
  const templateError = hasError(TEMPLATE_CODE, templateLine);

  let stopped = false;
  try {
    sampler.start();
    const spawnedAt = now();
    await bounded(within(editor.start(), options.coldTimeoutMs, `${editor.label} initialize`));
    const initializeMs = now() - spawnedAt;
    const openedAt = now();
    editor.open(doc);
    const first = await bounded(editor.diagnostics(doc, scriptError, options.coldTimeoutMs));
    const openToDiagnosticsMs = now() - openedAt;
    const coldReading = sampler.mark("cold");
    if (!first.satisfied)
      failures.push(`cold: planted TS${SCRIPT_CODE} on line ${scriptLine + 1} never appeared`);

    const alive = () => {
      if (starved) throw starved;
      const exited = editor.exited();
      if (exited) throw new Error(`${exited}\n${editor.stderr().slice(-4000)}`);
    };

    // A probe whose diagnostics never converged is not edited again: every later wait
    // would only time out, and the other probe and the requests still measure.
    const diverged = new Set<EditSample["probe"]>();
    const edit = async (cycle: number, probe: EditSample["probe"], phase: EditSample["phase"]) => {
      alive();
      if (diverged.has(probe)) return;
      const [from, to] =
        probe === "script"
          ? phase === "break"
            ? [SCRIPT_CLEAN, SCRIPT_BROKEN]
            : [SCRIPT_BROKEN, SCRIPT_CLEAN]
          : phase === "break"
            ? [TEMPLATE_CLEAN, TEMPLATE_BROKEN]
            : [TEMPLATE_BROKEN, TEMPLATE_CLEAN];
      const before = sampler.mark(`${probe}-${phase}`).cpuSeconds;
      const started = now();
      editor.change(doc, doc.replace(from, to));
      const expectError = phase === "break";
      const visible = probe === "script" ? scriptError : templateError;
      const result = await bounded(
        editor.diagnostics(
          doc,
          (diagnostics) => visible(diagnostics) === expectError,
          options.diagnosticsTimeoutMs,
        ),
      );
      const ms = now() - started;
      const cpuSeconds = sampler.mark(`${probe}-${phase}-done`).cpuSeconds - before;
      edits.push({ cycle, probe, phase, ms, ok: result.satisfied, cpuSeconds });
      if (!result.satisfied) {
        diverged.add(probe);
        const seen = result.diagnostics.map((item) => `TS${item.code}@${item.line + 1}`).join(", ");
        failures.push(
          `cycle ${cycle}: ${probe} ${phase} diagnostics never converged (last: [${seen}]); ${probe} edits stop`,
        );
      }
    };

    const ask = async (cycle: number) => {
      for (const request of spec.requests) {
        alive();
        const started = now();
        try {
          const position =
            "anchor" in request ? doc.resolve(request.anchor) : { line: 0, character: 0 };
          const call =
            request.kind === "hover"
              ? editor.hover(doc, position)
              : request.kind === "completion"
                ? editor.completion(doc, position)
                : request.kind === "definition"
                  ? editor.definition(doc, position)
                  : request.kind === "references"
                    ? editor.references(doc, position)
                    : request.kind === "rename"
                      ? editor.rename(doc, position, request.newName)
                      : editor.semanticTokens(doc);
          const answer = await bounded(
            within<string | string[] | Location[] | number>(
              call,
              options.requestTimeoutMs,
              `${request.kind} ${request.name}`,
            ),
          );
          const ms = now() - started;
          const verdict = judge(request, answer, workspace);
          requests.push({
            cycle,
            kind: request.kind,
            name: request.name,
            ms,
            ok: verdict.ok,
            size: verdict.size,
          });
          answers[`${request.kind}: ${request.name}`] ??= verdict.summary;
        } catch (error) {
          const ms = now() - started;
          const message = error instanceof Error ? error.message : String(error);
          requests.push({
            cycle,
            kind: request.kind,
            name: request.name,
            ms,
            ok: false,
            size: 0,
            error: message.slice(0, 500),
          });
          answers[`${request.kind}: ${request.name}`] ??= `error: ${message.slice(0, 200)}`;
        }
      }
    };

    await edit(0, "script", "repair");
    const warmStart = sampler.mark("warm");
    const warmStartedAt = now();
    for (let cycle = 1; cycle <= options.cycles; cycle += 1) {
      if (now() - warmStartedAt > options.warmBudgetMs) {
        failures.push(
          `warm budget of ${options.warmBudgetMs / 1000}s spent after ${cycle - 1} cycles`,
        );
        break;
      }
      console.error(
        `  ${editor.label} cycle ${cycle} at ${((now() - warmStartedAt) / 1000).toFixed(1)}s`,
      );
      await edit(cycle, "script", "break");
      await edit(cycle, "script", "repair");
      await ask(cycle);
      await edit(cycle, "template", "break");
      await edit(cycle, "template", "repair");
      await ask(cycle);
    }
    const warmEnd = sampler.mark("warm-end");
    const warmWallMs = now() - warmStartedAt;
    await bounded(new Promise((resolve) => setTimeout(resolve, options.idleMs)));
    alive();
    const idleEnd = sampler.mark("idle-end");
    sampler.stop();
    stopped = true;
    await editor.stop();
    const survivors = killSurvivors(readTree(idleEnd.pids, false, true));
    if (survivors.length)
      failures.push(
        `${survivors.length} processes outlived shutdown and were killed: ${survivors.join(", ")}`,
      );

    const stride = Math.max(1, Math.ceil(sampler.samples.length / 400));
    const mib = (bytes: number) => Math.round((bytes / 1048576) * 10) / 10;
    return {
      editor: editor.label,
      kind: editor.kind,
      round,
      cold: {
        initializeMs,
        openToDiagnosticsMs,
        totalMs: initializeMs + openToDiagnosticsMs,
        ok: first.satisfied,
        reading: coldReading,
      },
      warm: {
        wallMs: warmWallMs,
        cpuSeconds: warmEnd.cpuSeconds - warmStart.cpuSeconds,
        peakRssBytes: sampler.peak("rssBytes"),
        peakPssBytes: sampler.peak("pssBytes"),
        end: warmEnd,
      },
      idle: {
        ms: options.idleMs,
        cpuSeconds: idleEnd.cpuSeconds - warmEnd.cpuSeconds,
        end: idleEnd,
      },
      edits,
      requests,
      answers,
      series: sampler.samples
        .filter((_, index) => index % stride === 0)
        .map((sample) => [
          Math.round(sample.atMs),
          mib(sample.rssBytes),
          sample.pssBytes == null ? null : mib(sample.pssBytes),
          Math.round(sample.cpuSeconds * 100) / 100,
        ]),
      failures,
    };
  } finally {
    sampler.stop();
    if (!stopped) {
      const last = sampler.samples.at(-1);
      await editor.stop().catch(() => {});
      // An aborted session's processes go with it.
      if (last) killSurvivors(readTree(last.pids, false, true));
    }
  }
}

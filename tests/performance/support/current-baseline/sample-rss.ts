// Bounded Linux RSS observations; no environment/argv inspection or peak/PSS claim.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { fileURLToPath } from "node:url";

const intervalMs = 50;
const deadlineMs = 330_000;
const cleanupMs = 1_000;
const quotaBytes = 64 * 1024 * 1024;

export interface ProcessRow {
  readonly pid: number;
  readonly parentPid: number;
  readonly startTicks: string;
  readonly state: string;
  readonly rssKiB: number;
}

type Ownership = Pick<ProcessRow, "parentPid" | "startTicks">;

/** One `/proc/<pid>/stat` line. The command name may contain spaces and parentheses. */
export function parseStat(pid: number, stat: string, pageSize: number): ProcessRow {
  const fields = stat
    .slice(stat.lastIndexOf(")") + 2)
    .split(/\s+/u)
    .filter((field) => field !== "");
  const [state, parent, ticks, pages] = [fields[0], fields[1], fields[19], fields[21]];
  if (state === undefined || parent === undefined || ticks === undefined || pages === undefined) {
    throw new Error(`unexpected stat line for ${pid}`);
  }
  const parentPid = Number.parseInt(parent, 10);
  const rssPages = Number.parseInt(pages, 10);
  if (!Number.isSafeInteger(parentPid) || !Number.isSafeInteger(rssPages)) {
    throw new Error(`unexpected stat numbers for ${pid}`);
  }
  return {
    pid,
    parentPid,
    startTicks: ticks,
    state,
    rssKiB: Math.floor((rssPages * pageSize) / 1024),
  };
}

/**
 * The root and everything it started: live descendants, and processes seen
 * under the root earlier that were since reparented.
 */
export function owned(
  table: ReadonlyMap<number, Ownership>,
  rootPid: number,
  rootTicks: string,
  known: ReadonlyMap<number, string>,
): number[] {
  const root = table.get(rootPid);
  if (root !== undefined && root.startTicks !== rootTicks) throw new Error("root PID was reused");
  const selected = new Set<number>();
  for (const [pid, ticks] of known) {
    if (table.get(pid)?.startTicks === ticks) selected.add(pid);
  }
  if (root !== undefined) selected.add(rootPid);
  for (;;) {
    const children = [...table]
      .filter(([pid, row]) => selected.has(row.parentPid) && !selected.has(pid))
      .map(([pid]) => pid);
    if (children.length === 0) break;
    for (const pid of children) selected.add(pid);
  }
  return [...selected].sort((left, right) => left - right);
}

function errorCode(error: unknown): string | undefined {
  if (typeof error !== "object" || error === null || !("code" in error)) return undefined;
  return typeof error.code === "string" ? error.code : undefined;
}

/** The process ended between the directory scan and the read. */
function vanished(error: unknown): boolean {
  const code = errorCode(error);
  return code === "ENOENT" || code === "ESRCH";
}

function denied(error: unknown): boolean {
  const code = errorCode(error);
  return code === "EACCES" || code === "EPERM";
}

const monotonicNs = (): string => process.hrtime.bigint().toString();

async function main(argv: readonly string[]): Promise<number> {
  const [rootPidText, rootTicks, destination] = argv;
  if (rootPidText === undefined || rootTicks === undefined || destination === undefined) {
    throw new Error("usage: sample-rss.ts ROOT_PID ROOT_START_TICKS DESTINATION");
  }
  const rootPid = Number.parseInt(rootPidText, 10);
  const pageSize = Number.parseInt(execFileSync("getconf", ["PAGESIZE"], { encoding: "utf8" }), 10);
  const readProcess = (pid: number): ProcessRow =>
    parseStat(pid, fs.readFileSync(`/proc/${pid}/stat`, "utf8"), pageSize);

  const known = new Map<number, string>([[rootPid, rootTicks]]);
  const executables = new Map<string, readonly [string, string]>();
  const deadline = performance.now() + deadlineMs;
  let written = 0;
  let stopped = false;
  let cleanupDeadline = Number.POSITIVE_INFINITY;
  let failure: string | null = null;

  // The observer writes one line, "stop", when it has closed the server.
  let input = "";
  let inputEnded = false;
  process.stdin.setEncoding("utf8");
  process.stdin.on("data", (chunk: string) => {
    input += chunk;
  });
  process.stdin.on("end", () => {
    inputEnded = true;
  });

  const output = fs.openSync(destination, "wx");
  const emit = (row: Readonly<Record<string, unknown>>): void => {
    const line = `${JSON.stringify(row)}\n`;
    written += Buffer.byteLength(line);
    if (written > quotaBytes) throw new Error("RSS raw output quota exceeded");
    fs.writeSync(output, line);
  };

  emit({
    kind: "start",
    pid: rootPid,
    startTicks: rootTicks,
    intervalMs,
    clock: "CLOCK_MONOTONIC",
    ns: monotonicNs(),
  });
  try {
    for (;;) {
      const start = performance.now();
      if (start > deadline) throw new Error("RSS sampler exceeded finite deadline");
      const table = new Map<number, ProcessRow>();
      for (const name of fs.readdirSync("/proc")) {
        if (!/^\d+$/u.test(name)) continue;
        const pid = Number.parseInt(name, 10);
        try {
          table.set(pid, readProcess(pid));
        } catch (error) {
          if (vanished(error)) continue;
          if (denied(error) && !known.has(pid)) continue;
          throw error;
        }
      }
      const members: Record<string, unknown>[] = [];
      for (const pid of owned(table, rootPid, rootTicks, known)) {
        const row = table.get(pid);
        if (row === undefined) continue;
        known.set(pid, row.startTicks);
        if (row.state === "Z") continue;
        const key = `${pid}:${row.startTicks}`;
        let executable: string;
        try {
          executable = fs.readlinkSync(`/proc/${pid}/exe`);
          if (!executables.has(key)) {
            const digest = createHash("sha256").update(fs.readFileSync(`/proc/${pid}/exe`));
            executables.set(key, [executable, digest.digest("hex")]);
          }
        } catch (error) {
          // A short-lived process is explicitly unobserved, never RSS zero.
          if (vanished(error)) continue;
          throw error;
        }
        const identity = executables.get(key);
        if (identity === undefined || identity[0] !== executable) {
          throw new Error("observed process executable changed");
        }
        members.push({ ...row, executable, executableSha256: identity[1] });
      }
      emit({ kind: "sample", ns: monotonicNs(), members, scanMs: performance.now() - start });
      if (!stopped) {
        const newline = input.indexOf("\n");
        if (newline >= 0 || inputEnded) {
          const line = newline >= 0 ? input.slice(0, newline) : input;
          if (line.trim() !== "stop") {
            throw new Error("explicit server-close stop signal is required");
          }
          stopped = true;
          cleanupDeadline = performance.now() + cleanupMs;
        }
      }
      if (stopped && members.length === 0) break;
      if (stopped && performance.now() > cleanupDeadline) {
        throw new Error("observed server descendants survived shutdown");
      }
      await sleep(Math.max(0, intervalMs - (performance.now() - start)));
    }
  } catch (error) {
    failure = error instanceof Error ? error.message : String(error);
  }
  const survivors: { pid: number; startTicks: string }[] = [];
  for (const [pid, ticks] of known) {
    try {
      const row = readProcess(pid);
      if (row.startTicks === ticks && row.state !== "Z") {
        survivors.push({ pid, startTicks: ticks });
        process.kill(pid, "SIGKILL");
      }
    } catch (error) {
      if (!vanished(error)) throw error;
    }
  }
  emit({
    kind: "finish",
    ns: monotonicNs(),
    stopped,
    failure,
    survivorsBeforeForcedCleanup: survivors,
    limits: "50ms sampled RSS sum; missed short-lived processes, not true peak or PSS",
  });
  fs.closeSync(output);
  return failure !== null || survivors.length > 0 || !stopped ? 1 : 0;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exit(await main(process.argv.slice(2)));
}

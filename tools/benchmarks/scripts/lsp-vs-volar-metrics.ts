/**
 * Process-tree CPU and memory for `lsp-vs-volar.ts`, plus the statistics the
 * report prints. Linux reads `/proc` (CPU ticks, RSS and PSS); other systems
 * fall back to `ps` so the harness still runs for a local smoke test.
 */

import { spawnSync } from "node:child_process";
import { readFileSync, readdirSync } from "node:fs";

export interface TreeReading {
  /** user+sys seconds of every live member plus reaped descendants. */
  cpuSeconds: number;
  rssBytes: number;
  /** Proportional set size: shared pages split between the processes mapping them. */
  pssBytes: number | null;
  processes: number;
  /** Members that exited after `/proc/<pid>/stat` discovery but before the PSS read. */
  pssExitedProcesses: number;
  /** Every live member, so a tree past its memory budget can be killed whole. */
  pids: number[];
  /** Per process, at phase marks only: which member holds the memory and CPU. */
  members?: Array<{
    pid: number;
    name: string;
    cpuSeconds: number;
    rssBytes: number;
    pssBytes: number | null;
  }>;
}

const CLOCK_TICKS = 100;
const PAGE_BYTES = 4096;

type ProcRow = { pid: number; ppid: number; name: string; cpuTicks: number; rssPages: number };

function procRows(): ProcRow[] {
  const rows: ProcRow[] = [];
  for (const entry of readdirSync("/proc")) {
    if (!/^\d+$/.test(entry)) continue;
    let stat: string;
    try {
      stat = readFileSync(`/proc/${entry}/stat`, "utf8");
    } catch {
      continue;
    }
    // `comm` may contain spaces and parentheses; fields resume after the last ')'.
    const fields = stat.slice(stat.lastIndexOf(")") + 2).split(" ");
    rows.push({
      pid: Number(entry),
      ppid: Number(fields[1]),
      name: stat.slice(stat.indexOf("(") + 1, stat.lastIndexOf(")")),
      // utime stime cutime cstime: fields 14-17 of stat(5), 11-14 after comm.
      cpuTicks: Number(fields[11]) + Number(fields[12]) + Number(fields[13]) + Number(fields[14]),
      rssPages: Number(fields[21]),
    });
  }
  return rows;
}

function pssBytes(pid: number): number | null {
  try {
    const rollup = readFileSync(`/proc/${pid}/smaps_rollup`, "utf8");
    const match = /^Pss:\s+(\d+) kB/m.exec(rollup);
    return match ? Number(match[1]) * 1024 : null;
  } catch {
    return null;
  }
}

export function reconcilePssSnapshot<T extends { pid: number }>(
  members: T[],
  readPss: (pid: number) => number | null,
  isAlive: (pid: number) => boolean,
): { members: Array<{ member: T; pssBytes: number | null }>; exited: number } {
  const snapshot: Array<{ member: T; pssBytes: number | null }> = [];
  let exited = 0;
  for (const member of members) {
    const pss = readPss(member.pid);
    if (pss == null && !isAlive(member.pid)) {
      exited++;
      continue;
    }
    snapshot.push({ member, pssBytes: pss });
  }
  return { members: snapshot, exited };
}

/** Whether a discovered `/proc/<pid>/stat` row can still own an address space. */
export function procStatHasAddressSpace(stat: string): boolean {
  const fields = stat.slice(stat.lastIndexOf(")") + 2).split(" ");
  return fields[0] !== "Z" && fields[0] !== "X";
}

function procEntryStillAlive(pid: number): boolean {
  try {
    return procStatHasAddressSpace(readFileSync(`/proc/${pid}/stat`, "utf8"));
  } catch {
    return false;
  }
}

function descendants<T extends { pid: number; ppid: number }>(rows: T[], roots: number[]): T[] {
  const byParent = new Map<number, T[]>();
  for (const row of rows) {
    const siblings = byParent.get(row.ppid);
    if (siblings) siblings.push(row);
    else byParent.set(row.ppid, [row]);
  }
  const byPid = new Map(rows.map((row) => [row.pid, row]));
  const members: T[] = [];
  const stack = roots.filter((pid) => byPid.has(pid));
  const seen = new Set<number>();
  while (stack.length) {
    const pid = stack.pop()!;
    if (seen.has(pid)) continue;
    seen.add(pid);
    members.push(byPid.get(pid)!);
    for (const child of byParent.get(pid) ?? []) stack.push(child.pid);
  }
  return members;
}

export interface MachineMemory {
  availableBytes: number;
  /** tmpfs and shared memory: RAM that no process's RSS shows. */
  shmemBytes: number;
  /** Largest processes by RSS, `name MiB`. */
  largest: (except: Set<number>) => string[];
}

/** The machine's memory, from `/proc/meminfo`; `null` where there is none. */
export function machineMemory(): MachineMemory | null {
  if (process.platform !== "linux") return null;
  const info = readFileSync("/proc/meminfo", "utf8");
  const kib = (key: string) =>
    Number(new RegExp(`^${key}:\\s+(\\d+) kB`, "m").exec(info)?.[1] ?? NaN) * 1024;
  return {
    availableBytes: kib("MemAvailable"),
    shmemBytes: kib("Shmem"),
    largest: (except) =>
      procRows()
        .filter((row) => !except.has(row.pid))
        .sort((left, right) => right.rssPages - left.rssPages)
        .slice(0, 5)
        .map((row) => `${row.name} ${Math.round((row.rssPages * PAGE_BYTES) / 1048576)}`),
  };
}

/** Members of `reading` still alive, as `name`; each is SIGKILLed. */
export function killSurvivors(reading: TreeReading): string[] {
  const survivors: string[] = [];
  for (const member of reading.members ?? []) {
    try {
      process.kill(member.pid, "SIGKILL");
      survivors.push(member.name);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ESRCH") throw error;
    }
  }
  return survivors;
}

/** `[[dd-]hh:]mm:ss[.ff]` from `ps -o time`. */
export function psTimeSeconds(text: string): number {
  const [days, rest] = text.includes("-") ? text.split("-") : ["0", text];
  const parts = rest.split(":").map(Number);
  while (parts.length < 3) parts.unshift(0);
  return Number(days) * 86400 + parts[0] * 3600 + parts[1] * 60 + parts[2];
}

export function readTree(roots: number[], withPss = true, withMembers = false): TreeReading {
  if (process.platform === "linux") {
    const discovered = descendants(procRows(), roots);
    const snapshot = withPss
      ? reconcilePssSnapshot(discovered, pssBytes, procEntryStillAlive)
      : { members: discovered.map((member) => ({ member, pssBytes: null })), exited: 0 };
    const members = snapshot.members.map(({ member }) => member);
    const pssByPid = new Map(
      snapshot.members.map(({ member, pssBytes }) => [member.pid, pssBytes]),
    );
    const values = [...pssByPid.values()];
    const pss =
      withPss && values.every((value) => value != null)
        ? values.reduce((sum, value) => sum! + value!, 0)
        : null;
    return {
      cpuSeconds: members.reduce((sum, row) => sum + row.cpuTicks, 0) / CLOCK_TICKS,
      rssBytes: members.reduce((sum, row) => sum + row.rssPages, 0) * PAGE_BYTES,
      pssBytes: pss,
      processes: members.length,
      pssExitedProcesses: snapshot.exited,
      pids: members.map((member) => member.pid),
      ...(withMembers && {
        members: members.map((member) => ({
          pid: member.pid,
          name: member.name,
          cpuSeconds: member.cpuTicks / CLOCK_TICKS,
          rssBytes: member.rssPages * PAGE_BYTES,
          pssBytes: pssByPid.get(member.pid) ?? null,
        })),
      }),
    };
  }
  const ps = spawnSync("ps", ["-Ao", "pid=,ppid=,rss=,time=,comm="], { encoding: "utf8" });
  if (ps.status !== 0) throw new Error(`ps failed (${ps.status}): ${ps.stderr ?? ps.error}`);
  const rows = ps.stdout
    .trim()
    .split("\n")
    .map((line) => /^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\S+)\s+(.*)$/.exec(line))
    .filter((match) => match != null)
    .map((match) => ({
      pid: Number(match[1]),
      ppid: Number(match[2]),
      rssKiB: Number(match[3]),
      cpu: psTimeSeconds(match[4]),
      name: match[5].split("/").at(-1)!,
    }));
  const members = descendants(rows, roots);
  return {
    cpuSeconds: members.reduce((sum, row) => sum + row.cpu, 0),
    rssBytes: members.reduce((sum, row) => sum + row.rssKiB * 1024, 0),
    pssBytes: null,
    processes: members.length,
    pssExitedProcesses: 0,
    pids: members.map((member) => member.pid),
    ...(withMembers && {
      members: members.map((member) => ({
        pid: member.pid,
        name: member.name,
        cpuSeconds: member.cpu,
        rssBytes: member.rssKiB * 1024,
        pssBytes: null,
      })),
    }),
  };
}

/** Periodic readings while a session runs; `mark` adds an exact reading at a phase edge. */
export class TreeSampler {
  readonly samples: Array<TreeReading & { atMs: number; mark?: string }> = [];
  /** Called with every reading, periodic or marked. */
  onReading: ((reading: TreeReading) => void) | undefined;
  private timer: NodeJS.Timeout | undefined;
  private readonly origin = performance.now();
  private readonly roots: () => number[];
  private readonly intervalMs: number;

  constructor(roots: () => number[], intervalMs: number) {
    this.roots = roots;
    this.intervalMs = intervalMs;
  }

  start(): void {
    this.timer = setInterval(() => this.read(), this.intervalMs);
    this.timer.unref();
  }

  stop(): void {
    clearInterval(this.timer);
  }

  mark(label: string): TreeReading {
    return this.read(label);
  }

  private read(mark?: string): TreeReading {
    const reading = readTree(this.roots(), true, mark != null);
    this.samples.push({ ...reading, atMs: performance.now() - this.origin, mark });
    this.onReading?.(reading);
    return reading;
  }

  peak(key: "rssBytes" | "pssBytes"): number | null {
    let peak: number | null = null;
    for (const sample of this.samples) {
      const value = sample[key];
      if (value != null && (peak == null || value > peak)) peak = value;
    }
    return peak;
  }
}

/** Nearest-rank percentile; `p` in [0, 100]. */
export function percentile(values: number[], p: number): number | null {
  if (!values.length) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const rank = Math.ceil((p / 100) * sorted.length);
  return sorted[Math.min(sorted.length, Math.max(1, rank)) - 1];
}

export function median(values: number[]): number | null {
  return percentile(values, 50);
}

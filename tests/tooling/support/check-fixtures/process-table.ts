//! Reading the live process and thread tree (#4126).
//!
//! `EAGAIN` from `fork`/`clone` is a *task* budget failure, and on Linux a task
//! is a thread, not a process: `RLIMIT_NPROC` and the cgroup `pids` controller
//! both count threads. A table that only counted processes would therefore
//! under-report the very quantity that ran out, so `/proc` is read directly for
//! `num_threads` wherever it exists and the `ps` fallback records `null` rather
//! than pretending a process is one task.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export type TaskRecord = {
  /** Process id. */
  readonly pid: number;
  /** Parent process id. */
  readonly ppid: number;
  /**
   * Process group id. Phase children are spawned into their own group, so this
   * is what still identifies a leaked grandchild after `init` reparents it.
   */
  readonly pgid: number;
  /** Single-letter process state; `Z` is a zombie. */
  readonly state: string;
  /** Executable base name, e.g. `node`, `vize`, `tsgo`. */
  readonly command: string;
  /** Live thread count, or `null` where the platform does not report it. */
  readonly threads: number | null;
};

const PROC_ROOT = "/proc";

/** `/proc/<pid>/stat` field offsets counted from the field after `comm`. */
const STAT_PPID = 1;
const STAT_PGRP = 2;
const STAT_NUM_THREADS = 17;

/**
 * Parse one `/proc/<pid>/stat` line.
 *
 * `comm` is unquoted and may itself contain spaces and parentheses, so the
 * split point is the *last* `)`, not the first.
 */
export function parseProcStat(content: string): TaskRecord | null {
  const open = content.indexOf("(");
  const close = content.lastIndexOf(")");
  if (open < 0 || close < open) {
    return null;
  }
  const pid = Number.parseInt(content.slice(0, open).trim(), 10);
  const command = content.slice(open + 1, close);
  const rest = content
    .slice(close + 2)
    .trim()
    .split(/\s+/);
  const ppid = Number.parseInt(rest[STAT_PPID] ?? "", 10);
  const pgid = Number.parseInt(rest[STAT_PGRP] ?? "", 10);
  const threads = Number.parseInt(rest[STAT_NUM_THREADS] ?? "", 10);
  if (!Number.isInteger(pid) || !Number.isInteger(ppid) || !Number.isInteger(pgid)) {
    return null;
  }
  return {
    command,
    pgid,
    pid,
    ppid,
    state: rest[0] ?? "?",
    threads: Number.isInteger(threads) ? threads : null,
  };
}

/** Parse `ps -A -o pid=,ppid=,pgid=,state=,comm=` output. */
export function parsePsTable(stdout: string): TaskRecord[] {
  const records: TaskRecord[] = [];
  for (const line of stdout.split("\n")) {
    const fields = line.trim().split(/\s+/);
    if (fields.length < 5) {
      continue;
    }
    const [rawPid, rawPpid, rawPgid, state] = fields as [string, string, string, string];
    const pid = Number.parseInt(rawPid, 10);
    const ppid = Number.parseInt(rawPpid, 10);
    const pgid = Number.parseInt(rawPgid, 10);
    if (!Number.isInteger(pid) || !Number.isInteger(ppid) || !Number.isInteger(pgid)) {
      continue;
    }
    records.push({
      command: path.basename(fields.slice(4).join(" ")),
      pgid,
      pid,
      ppid,
      state: state.slice(0, 1),
      threads: null,
    });
  }
  return records;
}

function readProcTable(): TaskRecord[] {
  const records: TaskRecord[] = [];
  for (const entry of fs.readdirSync(PROC_ROOT)) {
    if (!/^\d+$/.test(entry)) {
      continue;
    }
    let content: string;
    try {
      content = fs.readFileSync(path.join(PROC_ROOT, entry, "stat"), "utf8");
    } catch {
      // The process exited between `readdir` and `read`; it is not live.
      continue;
    }
    const record = parseProcStat(content);
    if (record != null) {
      records.push(record);
    }
  }
  return records;
}

function readPsTable(): TaskRecord[] {
  const result = spawnSync("ps", ["-A", "-o", "pid=,ppid=,pgid=,state=,comm="], {
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.status !== 0 || typeof result.stdout !== "string") {
    return [];
  }
  return parsePsTable(result.stdout);
}

/** Whether this platform exposes the `/proc` task tree. */
export function hasProcFs(): boolean {
  return process.platform === "linux" && fs.existsSync(path.join(PROC_ROOT, "self", "stat"));
}

/** Snapshot every live process, with thread counts where the platform has them. */
export function readProcessTable(): TaskRecord[] {
  return hasProcFs() ? readProcTable() : readPsTable();
}

/** Sum of live tasks, counting a process with unknown thread count as one. */
export function liveTaskCount(records: readonly TaskRecord[]): number {
  return records.reduce((total, record) => total + (record.threads ?? 1), 0);
}

/**
 * Every process in `pgid`.
 *
 * Group membership, not parentage, is the durable relation: a grandchild whose
 * parent already exited is reparented to `init` and stops being a descendant,
 * but it keeps the process group it was forked into.
 */
export function tasksInGroup(records: readonly TaskRecord[], pgid: number): TaskRecord[] {
  return records.filter((record) => record.pgid === pgid);
}

/** Every transitive child of `rootPid`, excluding `rootPid` itself. */
export function descendantsOf(records: readonly TaskRecord[], rootPid: number): TaskRecord[] {
  const childrenByParent = new Map<number, TaskRecord[]>();
  for (const record of records) {
    const siblings = childrenByParent.get(record.ppid);
    if (siblings == null) {
      childrenByParent.set(record.ppid, [record]);
    } else {
      siblings.push(record);
    }
  }
  const collected: TaskRecord[] = [];
  const seen = new Set<number>([rootPid]);
  const queue: number[] = [rootPid];
  while (queue.length > 0) {
    const pid = queue.shift() as number;
    for (const child of childrenByParent.get(pid) ?? []) {
      if (seen.has(child.pid)) {
        continue;
      }
      seen.add(child.pid);
      collected.push(child);
      queue.push(child.pid);
    }
  }
  return collected;
}

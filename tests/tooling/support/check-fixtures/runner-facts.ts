//! Runner process-budget facts (#4126).
//!
//! Three numbers decide whether a `fork` returns `EAGAIN`, and none of them was
//! recorded when the `vue-parity` job hit one: how many cores the runner has
//! (checker pools and Go runtimes size themselves from it), the per-uid
//! `RLIMIT_NPROC` that `ulimit -u` reports, and the cgroup `pids` controller's
//! current and maximum task counts. They are sampled together so an artifact
//! can distinguish "this lane leaked" from "the box was already full".

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

export type CgroupPids = {
  /** Cgroup path from `/proc/self/cgroup`, or `null` when unavailable. */
  readonly path: string | null;
  /** Directory the counters were read from. */
  readonly source: string | null;
  /** `pids.current`. */
  readonly current: number | null;
  /** `pids.max`; the literal `"max"` means the controller is unbounded. */
  readonly max: number | "max" | null;
};

export type RunnerFacts = {
  readonly platform: string;
  readonly cpuCount: number;
  /** `ulimit -u`, i.e. `RLIMIT_NPROC`. */
  readonly ulimitProcesses: number | "unlimited" | null;
  readonly cgroupPids: CgroupPids;
};

const CGROUP_ROOT = "/sys/fs/cgroup";

/** Parse the output of `ulimit -u`. */
export function parseUlimitProcesses(stdout: string): number | "unlimited" | null {
  const value = stdout.trim();
  if (value === "unlimited") {
    return "unlimited";
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isInteger(parsed) ? parsed : null;
}

/**
 * Parse one row of `/proc/self/limits`.
 *
 * This is read in preference to `ulimit` because the shell spelling is not
 * portable: `/bin/sh` on Ubuntu is dash, whose `ulimit` names `RLIMIT_NPROC`
 * `-p` and rejects bash's `-u` outright. `/proc` has no such dialect.
 */
export function parseProcLimits(
  content: string,
  name: string,
): {
  soft: number | "unlimited" | null;
  hard: number | "unlimited" | null;
} {
  for (const line of content.split("\n")) {
    if (!line.startsWith(name)) {
      continue;
    }
    const rest = line.slice(name.length).trim();
    const [soft, hard] = rest.split(/\s{2,}/);
    return { hard: parseUlimitProcesses(hard ?? ""), soft: parseUlimitProcesses(soft ?? "") };
  }
  return { hard: null, soft: null };
}

const MAX_PROCESSES = "Max processes";

function readProcLimits(): {
  soft: number | "unlimited" | null;
  hard: number | "unlimited" | null;
} {
  try {
    return parseProcLimits(fs.readFileSync("/proc/self/limits", "utf8"), MAX_PROCESSES);
  } catch {
    return { hard: null, soft: null };
  }
}

/** Read the soft `RLIMIT_NPROC`, i.e. what `ulimit -u` reports under bash. */
export function readUlimitProcesses(): number | "unlimited" | null {
  return readProcLimits().soft ?? readUlimit();
}

/**
 * The hard `RLIMIT_NPROC` ceiling.
 *
 * A constrained cycle lowers the soft limit, and a soft limit above the hard
 * one is rejected outright, so the ceiling has to be read before the budget is
 * applied rather than discovered from a failing `ulimit`.
 */
export function readUlimitProcessesHard(): number | "unlimited" | null {
  return readProcLimits().hard ?? readUlimit("-H");
}

function readUlimit(prefix = ""): number | "unlimited" | null {
  const result = spawnSync(
    "/bin/sh",
    ["-c", `ulimit ${prefix}u 2>/dev/null || ulimit ${prefix}p`],
    {
      encoding: "utf8",
    },
  );
  if (result.status !== 0 || typeof result.stdout !== "string") {
    return null;
  }
  return parseUlimitProcesses(result.stdout);
}

/**
 * Extract the cgroup path a task belongs to.
 *
 * cgroup v2 writes a single `0::<path>` line; v1 writes one line per
 * controller and only the `pids` controller carries the counters this reads.
 */
export function parseCgroupPath(content: string): string | null {
  let unified: string | null = null;
  for (const line of content.split("\n")) {
    const fields = line.split(":");
    if (fields.length < 3) {
      continue;
    }
    const controllers = fields[1] ?? "";
    const cgroupPath = fields.slice(2).join(":");
    if (controllers.split(",").includes("pids")) {
      return cgroupPath;
    }
    if (controllers === "" && unified == null) {
      unified = cgroupPath;
    }
  }
  return unified;
}

function readCounter(file: string): string | null {
  try {
    return fs.readFileSync(file, "utf8").trim();
  } catch {
    return null;
  }
}

function candidateDirectories(cgroupPath: string | null): string[] {
  const relative = cgroupPath == null ? "" : cgroupPath.replace(/^\//, "");
  const candidates = [
    // A namespaced container sees its own cgroup at the mount root.
    CGROUP_ROOT,
    path.join(CGROUP_ROOT, relative),
    // cgroup v1 mounts the `pids` controller in its own hierarchy.
    path.join(CGROUP_ROOT, "pids", relative),
    path.join(CGROUP_ROOT, "pids"),
  ];
  return candidates.filter((candidate, index) => candidates.indexOf(candidate) === index);
}

/** Read the cgroup `pids` controller counters for the current task. */
export function readCgroupPids(): CgroupPids {
  const raw = readCounter("/proc/self/cgroup");
  const cgroupPath = raw == null ? null : parseCgroupPath(raw);
  for (const directory of candidateDirectories(cgroupPath)) {
    const current = readCounter(path.join(directory, "pids.current"));
    if (current == null) {
      continue;
    }
    const parsedCurrent = Number.parseInt(current, 10);
    const rawMax = readCounter(path.join(directory, "pids.max"));
    const parsedMax = rawMax == null ? null : Number.parseInt(rawMax, 10);
    return {
      current: Number.isInteger(parsedCurrent) ? parsedCurrent : null,
      max: rawMax === "max" ? "max" : Number.isInteger(parsedMax) ? parsedMax : null,
      path: cgroupPath,
      source: directory,
    };
  }
  return { current: null, max: null, path: cgroupPath, source: null };
}

type StaticRunnerFacts = Omit<RunnerFacts, "cgroupPids">;

let cachedStaticFacts: StaticRunnerFacts | null = null;

/**
 * The platform, CPU count, and process rlimit cannot change while the
 * supervisor runs, and reading the rlimit falls back to spawning `/bin/sh`.
 * Sampling happens on an interval for the lifetime of every phase, so re-read
 * only the cgroup counters and keep the rest for the run.
 */
function staticRunnerFacts(): StaticRunnerFacts {
  cachedStaticFacts ??= {
    cpuCount: os.availableParallelism(),
    platform: `${process.platform}-${process.arch}`,
    ulimitProcesses: readUlimitProcesses(),
  };
  return cachedStaticFacts;
}

/** Sample every runner fact that bounds process creation. */
export function readRunnerFacts(): RunnerFacts {
  return { ...staticRunnerFacts(), cgroupPids: readCgroupPids() };
}

//! Labelled topology samples and the descendant guard (#4126).

import {
  descendantsOf,
  liveTaskCount,
  readProcessTable,
  type TaskRecord,
  tasksInGroup,
} from "./process-table.ts";
import { readRunnerFacts, type RunnerFacts } from "./runner-facts.ts";

export type TopologySample = {
  /** `before`, `peak`, or `after`. */
  readonly label: string;
  readonly atIso: string;
  readonly elapsedMs: number;
  readonly runner: RunnerFacts;
  /** Live tasks across the whole box, threads included where reported. */
  readonly liveTasks: number;
  readonly liveProcesses: number;
  /**
   * The live process and thread tree this lane can be held to: the supervisor,
   * everything descended from it, everything in the supervised process group,
   * and every `node`, `vize`, `tsgo`, `corsa`, or `tsc` process anywhere on the box —
   * so a checker that escaped both relations is still on the record. Unrelated
   * system processes are counted in `liveTasks` but not enumerated, which is
   * what keeps 35 phases x 3 samples inside an artifact worth downloading.
   */
  readonly tasks: readonly TaskRecord[];
  readonly group: {
    /** Process group the supervised phase was spawned into. */
    readonly pgid: number | null;
    readonly liveTasks: number;
    readonly processes: readonly TaskRecord[];
  };
};

/**
 * Commands whose survival past a phase is a leak.
 *
 * `corsa` rides along with `tsgo`, and TypeScript 7 stable exposes `tsc`, so
 * guarding one name would miss part of the supported runtime surface.
 */
export const GUARDED_COMMANDS: readonly string[] = ["node", "vize", "tsgo", "corsa", "tsc"];

export type SampleOptions = {
  /** Process group to attribute, or `null` before one exists. */
  readonly pgid: number | null;
  /** Monotonic origin, from `performance.now()`. */
  readonly startedAt: number;
  /** Pre-read table, so `before`/`peak`/`after` can share one read. */
  readonly records?: readonly TaskRecord[];
  readonly runner?: RunnerFacts;
  /** Supervisor pid; defaults to this process. */
  readonly rootPid?: number;
};

/** The subset of the live table this lane is accountable for. */
export function relevantTasks(
  records: readonly TaskRecord[],
  options: { readonly pgid: number | null; readonly rootPid: number },
): TaskRecord[] {
  const collected = new Map<number, TaskRecord>();
  for (const record of records) {
    const inGroup = options.pgid != null && record.pgid === options.pgid;
    if (record.pid === options.rootPid || inGroup || isGuarded(record)) {
      collected.set(record.pid, record);
    }
  }
  for (const record of descendantsOf(records, options.rootPid)) {
    collected.set(record.pid, record);
  }
  return [...collected.values()].sort((left, right) => left.pid - right.pid);
}

/** Take one labelled sample of the live process and thread tree. */
export function sampleTopology(label: string, options: SampleOptions): TopologySample {
  const records = options.records ?? readProcessTable();
  const rootPid = options.rootPid ?? process.pid;
  const group = options.pgid == null ? [] : tasksInGroup(records, options.pgid);
  return {
    atIso: new Date().toISOString(),
    elapsedMs: Math.round(performance.now() - options.startedAt),
    group: {
      liveTasks: liveTaskCount(group),
      pgid: options.pgid,
      processes: group,
    },
    label,
    liveProcesses: records.length,
    liveTasks: liveTaskCount(records),
    runner: options.runner ?? readRunnerFacts(),
    tasks: relevantTasks(records, { pgid: options.pgid, rootPid }),
  };
}

/** Keep whichever sample observed the most live tasks in the tracked group. */
export function peakOf(left: TopologySample | null, right: TopologySample): TopologySample {
  if (left == null) {
    return right;
  }
  return right.group.liveTasks > left.group.liveTasks ? right : left;
}

export type GuardOptions = {
  /** Process group the phase ran in. */
  readonly pgid: number;
  /** The supervisor's own pid; it and its group are not the phase. */
  readonly rootPid: number;
  readonly ignorePids?: Iterable<number>;
};

function isGuarded(record: TaskRecord): boolean {
  return GUARDED_COMMANDS.includes(record.command.toLowerCase());
}

/**
 * Every `node`, `vize`, `tsgo`, `corsa`, or `tsc` task that outlived its phase.
 *
 * Both relations are checked. Process-group membership survives reparenting, so
 * it catches a grandchild whose parent already exited; direct descent catches a
 * child that was placed in some other group. Zombies count: a task that has
 * exited but not been reaped still occupies a slot in the `pids` controller and
 * in `RLIMIT_NPROC`, which is exactly the budget that ran out.
 */
export function guardedSurvivors(
  records: readonly TaskRecord[],
  options: GuardOptions,
): TaskRecord[] {
  const excluded = new Set<number>(options.ignorePids ?? []);
  excluded.add(options.rootPid);
  const survivors = new Map<number, TaskRecord>();
  for (const record of [
    ...tasksInGroup(records, options.pgid),
    ...descendantsOf(records, options.rootPid),
  ]) {
    if (excluded.has(record.pid) || !isGuarded(record)) {
      continue;
    }
    survivors.set(record.pid, record);
  }
  return [...survivors.values()].sort((left, right) => left.pid - right.pid);
}

/** Render one survivor for a failure message. */
export function describeSurvivor(record: TaskRecord): string {
  const kind = record.state === "Z" ? "zombie" : "live";
  const threads = record.threads == null ? "" : `, threads=${record.threads}`;
  return `${kind} ${record.command} pid=${record.pid} ppid=${record.ppid} pgid=${record.pgid}${threads}`;
}

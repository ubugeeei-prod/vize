//! Spawning one child under process-topology supervision (#4126).

import { spawn } from "node:child_process";

import { readProcessTable, type TaskRecord } from "./process-table.ts";
import { readRunnerFacts } from "./runner-facts.ts";
import { guardedSurvivors, peakOf, sampleTopology, type TopologySample } from "./topology.ts";

export type SupervisedResult = {
  readonly argv: readonly string[];
  readonly pid: number | null;
  readonly pgid: number | null;
  readonly exitCode: number | null;
  readonly signal: string | null;
  readonly spawnError: string | null;
  readonly stdout: string;
  readonly stderr: string;
  readonly durationMs: number;
  readonly samples: {
    readonly before: TopologySample;
    readonly peak: TopologySample;
    readonly after: TopologySample;
  };
  /** Guarded tasks still alive once the child returned. */
  readonly survivors: readonly TaskRecord[];
  /** Whether the group had to be killed to restore the baseline. */
  readonly reaped: boolean;
};

export type SupervisedOptions = {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly env: NodeJS.ProcessEnv;
  readonly sampleIntervalMs: number;
  /** Monotonic origin shared across a run, from `performance.now()`. */
  readonly startedAt: number;
  /** Capture the child's output instead of inheriting the parent's streams. */
  readonly capture: boolean;
};

/** How long a killed group is given to leave the process table. */
const REAP_TIMEOUT_MS = 2_000;
const REAP_POLL_MS = 10;

/**
 * Kill the group and confirm it is gone before claiming it was reaped.
 *
 * `SIGKILL` is delivered asynchronously: `process.kill` returning only proves
 * the signal was queued, so a survivor stays in the table for a short window
 * afterwards, first running and then as a zombie until `init` reaps it.
 * Reporting `reaped` off the `kill` call alone made the claim ahead of the
 * kernel, which left the next phase's baseline racing the last phase's corpses.
 * Convergence is polled instead, and a group that will not leave is reported as
 * not reaped rather than assumed clean.
 */
async function killGroup(pgid: number, rootPid: number): Promise<boolean> {
  try {
    process.kill(-pgid, "SIGKILL");
  } catch {
    return false;
  }
  const deadline = performance.now() + REAP_TIMEOUT_MS;
  for (;;) {
    if (guardedSurvivors(readProcessTable(), { pgid, rootPid }).length === 0) {
      return true;
    }
    if (performance.now() >= deadline) {
      return false;
    }
    await new Promise((resolve) => setTimeout(resolve, REAP_POLL_MS));
  }
}

/**
 * Run `command` in its own process group, sampling the topology throughout.
 *
 * The group is what makes the guard sound: when a child leaks a grandchild and
 * then exits, `init` adopts the orphan and it stops being a descendant of
 * anything the supervisor knows — but it keeps the process group it was forked
 * into. A spawn failure is surfaced, never retried and never swallowed.
 */
export async function runSupervised(options: SupervisedOptions): Promise<SupervisedResult> {
  const runner = readRunnerFacts();
  const before = sampleTopology("before", { pgid: null, runner, startedAt: options.startedAt });
  const beganAt = performance.now();

  const child = spawn(options.command, [...options.args], {
    cwd: options.cwd,
    detached: true,
    env: options.env,
    stdio: options.capture ? ["ignore", "pipe", "pipe"] : "inherit",
  });
  const pgid = child.pid ?? null;

  const stdoutChunks: Buffer[] = [];
  const stderrChunks: Buffer[] = [];
  child.stdout?.on("data", (chunk: Buffer) => stdoutChunks.push(chunk));
  child.stderr?.on("data", (chunk: Buffer) => stderrChunks.push(chunk));

  let peak: TopologySample | null = null;
  const observe = () => {
    if (pgid != null) {
      peak = peakOf(peak, sampleTopology("peak", { pgid, runner, startedAt: options.startedAt }));
    }
  };
  observe();
  const timer = setInterval(observe, options.sampleIntervalMs);
  timer.unref();

  const exit = await new Promise<{
    code: number | null;
    error: Error | null;
    signal: string | null;
  }>((resolve) => {
    child.once("error", (error) => resolve({ code: null, error, signal: null }));
    child.once("close", (code, signal) => resolve({ code, error: null, signal }));
  });
  clearInterval(timer);
  observe();

  const durationMs = Math.round(performance.now() - beganAt);
  // One read backs both the `after` sample and the guard, so the artifact can
  // never disagree with the verdict drawn from it.
  const settled = readProcessTable();
  const after = sampleTopology("after", {
    pgid,
    records: settled,
    runner,
    startedAt: options.startedAt,
  });
  const survivors = pgid == null ? [] : guardedSurvivors(settled, { pgid, rootPid: process.pid });
  const reaped = survivors.length > 0 && pgid != null ? await killGroup(pgid, process.pid) : false;

  return {
    argv: [options.command, ...options.args],
    durationMs,
    exitCode: exit.code,
    pgid,
    pid: child.pid ?? null,
    reaped,
    samples: { after, before, peak: peak ?? before },
    signal: exit.signal,
    spawnError: exit.error == null ? null : `${exit.error.name}: ${exit.error.message}`,
    stderr: Buffer.concat(stderrChunks).toString("utf8"),
    stdout: Buffer.concat(stdoutChunks).toString("utf8"),
    survivors,
  };
}

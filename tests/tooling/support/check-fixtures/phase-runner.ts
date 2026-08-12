//! Running one supervised `test:check:fixtures` phase (#4126).

import type { CheckFixturePhase } from "./manifest.ts";
import type { TaskRecord } from "./process-table.ts";
import { runSupervised } from "./supervised-command.ts";
import { describeSurvivor, type TopologySample } from "./topology.ts";

export type PhaseStatus = "passed" | "failed" | "leaked" | "spawn-failed";

export type PhaseOutcome = {
  readonly id: string;
  readonly file: string;
  readonly argv: readonly string[];
  readonly pid: number | null;
  readonly pgid: number | null;
  readonly exitCode: number | null;
  readonly signal: string | null;
  readonly spawnError: string | null;
  readonly durationMs: number;
  readonly samples: {
    readonly before: TopologySample;
    readonly peak: TopologySample;
    readonly after: TopologySample;
  };
  /** Guarded tasks still alive after the phase returned. */
  readonly survivors: readonly TaskRecord[];
  /** Whether the group had to be killed to restore the pre-phase baseline. */
  readonly reaped: boolean;
  /** Merged child output, or `null` when the phase inherited the lane's streams. */
  readonly output: string | null;
  readonly status: PhaseStatus;
};

export type PhaseOptions = {
  readonly cwd: string;
  readonly nodeArgs: readonly string[];
  readonly env: NodeJS.ProcessEnv;
  readonly sampleIntervalMs: number;
  readonly startedAt: number;
  /**
   * Capture the phase's output instead of inheriting the lane's streams.
   *
   * The lane inherits, so a phase's TAP reaches the job log unchanged. A test
   * that supervises a phase from inside another `node --test` run must capture
   * instead, or the nested TAP would be read as the outer run's own.
   */
  readonly capture?: boolean;
};

/**
 * Classify a finished phase.
 *
 * The order matters: a phase that both failed its assertions and leaked is
 * reported as `failed`, because the assertion is the louder signal, and the
 * survivors travel in the outcome either way.
 */
export function classifyPhase(result: {
  readonly exitCode: number | null;
  readonly signal: string | null;
  readonly spawnError: string | null;
  readonly survivors: readonly TaskRecord[];
}): PhaseStatus {
  if (result.spawnError != null) {
    return "spawn-failed";
  }
  if (result.exitCode !== 0 || result.signal != null) {
    return "failed";
  }
  return result.survivors.length > 0 ? "leaked" : "passed";
}

/**
 * Environment keys a fresh top-level `node --test` must not inherit.
 *
 * Inherit `NODE_TEST_CONTEXT` and the child decides it is already inside a run,
 * prints "run() is being called recursively within a test file. skipping
 * running files", and exits zero without executing anything — a green phase
 * that checked nothing. `NODE_CHANNEL_FD` and `NODE_UNIQUE_ID` point at a
 * parent runner's IPC channel that does not exist in this child, and
 * `NODE_V8_COVERAGE` would make every phase write into the outer run's
 * coverage directory. The lane never meets any of these because it is launched
 * from a task runner, which is exactly why the guard's own tests have to.
 */
const INHERITED_RUNNER_KEYS = ["NODE_CHANNEL_FD", "NODE_UNIQUE_ID", "NODE_V8_COVERAGE"];

/** Strip another runner's context from a phase's environment. */
export function phaseEnv(env: NodeJS.ProcessEnv): NodeJS.ProcessEnv {
  return Object.fromEntries(
    Object.entries(env).filter(
      ([key]) => !key.startsWith("NODE_TEST_") && !INHERITED_RUNNER_KEYS.includes(key),
    ),
  ) as NodeJS.ProcessEnv;
}

/** Run one manifest phase as `node --test --test-concurrency=1 <file>`. */
export async function runPhase(
  phase: CheckFixturePhase,
  options: PhaseOptions,
): Promise<PhaseOutcome> {
  const capture = options.capture === true;
  const result = await runSupervised({
    args: [...options.nodeArgs, phase.file],
    capture,
    command: process.execPath,
    cwd: options.cwd,
    env: phaseEnv(options.env),
    sampleIntervalMs: options.sampleIntervalMs,
    startedAt: options.startedAt,
  });
  return {
    argv: result.argv,
    durationMs: result.durationMs,
    exitCode: result.exitCode,
    file: phase.file,
    id: phase.id,
    output: capture ? `${result.stdout}${result.stderr}` : null,
    pgid: result.pgid,
    pid: result.pid,
    reaped: result.reaped,
    samples: result.samples,
    signal: result.signal,
    spawnError: result.spawnError,
    status: classifyPhase(result),
    survivors: result.survivors,
  };
}

/** One-line verdict for a phase, used in logs and the step summary. */
export function describePhase(outcome: PhaseOutcome): string {
  const head = `${outcome.status} ${outcome.id} (${outcome.durationMs}ms, peak group tasks ${outcome.samples.peak.group.liveTasks})`;
  if (outcome.spawnError != null) {
    return `${head}: ${outcome.spawnError}`;
  }
  if (outcome.survivors.length > 0) {
    return `${head}: ${outcome.survivors.map(describeSurvivor).join("; ")}`;
  }
  return head;
}

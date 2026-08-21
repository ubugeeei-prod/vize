//! One focused cycle under an explicit constrained PID budget (#4126).

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import path from "node:path";

import { checkArgv, type CycleTarget } from "./cycle-targets.ts";
import type { TaskRecord } from "./process-table.ts";
import { parseProcLimits, parseUlimitProcesses, readUlimitProcessesHard } from "./runner-facts.ts";
import { runSupervised } from "./supervised-command.ts";
import { sampleTopology, type TopologySample } from "./topology.ts";

/** Corsa runtime process names before and after package-manager shims `exec`. */
const CORSA_COMMANDS = ["tsgo", "corsa", "tsc"];

/** Substrings that mean the runner ran out of task slots. */
const EAGAIN_MARKERS = ["Resource temporarily unavailable", "EAGAIN", "os error 11"];

export type PidBudget = {
  /** Live tasks on the box before the cycle started. */
  readonly baselineTasks: number;
  /** Documented headroom the cycle is allowed on top of the baseline. */
  readonly taskBudget: number;
  /** Soft `RLIMIT_NPROC` the cycle was asked to run under. */
  readonly ulimitProcesses: number;
  /** Soft limit read back from a child spawned through the same prologue. */
  readonly applied: number | "unlimited" | null;
  /** Whether the hard ceiling forced a looser limit than requested. */
  readonly clamped: boolean;
};

export type CycleRecord = {
  readonly index: number;
  readonly exitCode: number | null;
  readonly signal: string | null;
  readonly durationMs: number;
  readonly budget: PidBudget;
  /** SHA-256 over the exact `vize check --format json` stdout. */
  readonly outputSha256: string;
  readonly outputBytes: number;
  readonly peakGroupProcesses: number;
  readonly peakGroupLiveTasks: number;
  readonly peakCorsaProcesses: number;
  readonly reportedEagain: boolean;
  /**
   * The checker's own words, kept only when the cycle is red.
   *
   * A red cycle whose artifact holds a hash and nothing else cannot be
   * diagnosed without a rerun, which is the position this whole issue started
   * from.
   */
  readonly stderrExcerpt: string | null;
  readonly survivors: readonly TaskRecord[];
  readonly samples: {
    readonly before: TopologySample;
    readonly peak: TopologySample;
    readonly after: TopologySample;
  };
  readonly failures: readonly string[];
};

export function isCorsaRuntimeCommand(command: string): boolean {
  return CORSA_COMMANDS.includes(command.toLowerCase());
}

function countCorsa(sample: TopologySample): number {
  return sample.group.processes.filter((task) => isCorsaRuntimeCommand(task.command)).length;
}

function reportsEagain(text: string): boolean {
  return EAGAIN_MARKERS.some((marker) => text.includes(marker));
}

type CheckOutput = { readonly errorCount?: unknown; readonly fileCount?: unknown };

/**
 * Judge the checker's own report, not just its hash.
 *
 * Two cycles that both checked nothing would agree on every byte, so the file
 * count and the presence of the fixture's authored diagnostics are asserted
 * directly: a run that degraded, sampled, or bailed under budget pressure has
 * to be red rather than merely reproducible.
 */
export function outputFailures(
  stdout: string,
  expected: { readonly fileCount: number; readonly diagnostics: boolean },
): string[] {
  let parsed: CheckOutput;
  try {
    parsed = JSON.parse(stdout) as CheckOutput;
  } catch (error) {
    return [`check output is not JSON: ${String(error)}`];
  }
  const failures: string[] = [];
  if (parsed.fileCount !== expected.fileCount) {
    failures.push(`fileCount ${String(parsed.fileCount)} != ${expected.fileCount}`);
  }
  const errorCount = typeof parsed.errorCount === "number" ? parsed.errorCount : -1;
  // Only the intentional-errors fixture is held to a floor. A clean corpus is
  // pinned by the cycle hash instead: an absolute count there would turn every
  // unrelated diagnostic change into a process-budget failure, which is the
  // opposite of what this lane is for.
  if (expected.diagnostics && errorCount <= 0) {
    failures.push(`errorCount ${String(parsed.errorCount)} must stay above zero`);
  }
  return failures;
}

/** Exit status the budget prologue uses when it cannot lower the limit. */
export const BUDGET_NOT_APPLIED_EXIT = 97;

/**
 * Lower `RLIMIT_NPROC` for the cycle, then `exec` the checker in its place.
 *
 * `ulimit` is applied by the shell and inherited across `exec`, so the `sh`
 * process is replaced rather than left in the group. Both spellings are tried
 * because `/bin/sh` on Ubuntu is dash, which names `RLIMIT_NPROC` `-p` and
 * rejects bash's `-u`; when neither applies the shell exits
 * `BUDGET_NOT_APPLIED_EXIT` rather than running the cycle unconstrained, so a
 * missing constraint can never read as a passing cycle.
 */
export function budgetedArgv(budget: number, command: string, args: readonly string[]): string[] {
  const prologue =
    `ulimit -u ${budget} 2>/dev/null || ulimit -p ${budget} 2>/dev/null || ` +
    `exit ${BUDGET_NOT_APPLIED_EXIT}`;
  return ["-c", `${prologue}; exec "$@"`, "sh", command, ...args];
}

/**
 * Read back the soft limit a budgeted child actually runs under.
 *
 * The prologue returning zero only proves a shell builtin accepted the flag;
 * this proves the kernel applied it. A cycle that cannot demonstrate its own
 * constraint is not evidence of anything, so the reading is asserted rather
 * than assumed.
 */
export function verifyBudget(budget: number): number | "unlimited" | null {
  const reader =
    process.platform === "linux"
      ? { args: ["/proc/self/limits"], command: "cat" }
      : { args: ["-c", "ulimit -u 2>/dev/null || ulimit -p"], command: "/bin/sh" };
  const result = spawnSync("/bin/sh", budgetedArgv(budget, reader.command, reader.args), {
    encoding: "utf8",
  });
  if (result.status !== 0 || typeof result.stdout !== "string") {
    return null;
  }
  return process.platform === "linux"
    ? parseProcLimits(result.stdout, "Max processes").soft
    : parseUlimitProcesses(result.stdout);
}

/**
 * Turn a measured baseline and a documented headroom into a soft `ulimit -u`.
 *
 * The baseline counts every live task on the box, which over-counts the uid's
 * own share that `RLIMIT_NPROC` actually limits. That direction is deliberate:
 * over-counting only ever loosens the ceiling, so the constraint can never fail
 * a cycle for tasks the cycle did not create.
 */
export function resolveBudget(
  baselineTasks: number,
  taskBudget: number,
  hardLimit: number | "unlimited" | null = readUlimitProcessesHard(),
  verify: (budget: number) => number | "unlimited" | null = verifyBudget,
): PidBudget {
  const hard = hardLimit;
  const requested = baselineTasks + taskBudget;
  const clamped = typeof hard === "number" && hard < requested;
  const ulimitProcesses = clamped ? (hard as number) : requested;
  return {
    applied: verify(ulimitProcesses),
    baselineTasks,
    clamped,
    taskBudget,
    ulimitProcesses,
  };
}

export type CycleOptions = {
  readonly repoRoot: string;
  readonly vizeBin: string;
  readonly corsaBin: string;
  readonly sampleIntervalMs: number;
  readonly startedAt: number;
  readonly budgetCpuCount: number;
  /** Hash every later cycle must reproduce, or `null` for the first cycle. */
  readonly expectedSha256: string | null;
  readonly expectedExitCode: number | null;
};

/** Run one cycle of `target` and judge it against the documented bounds. */
export async function runCycle(
  target: CycleTarget,
  index: number,
  options: CycleOptions,
): Promise<CycleRecord> {
  const args = checkArgv(target, options.corsaBin);
  const taskBudget = target.taskBudget(options.budgetCpuCount);
  // The budget is re-measured every cycle: the box is shared, so a ceiling
  // derived once at the start would drift into either uselessness or a flake.
  const baseline = sampleTopology("baseline", { pgid: null, startedAt: options.startedAt });
  const budget = resolveBudget(baseline.liveTasks, taskBudget);

  const result = await runSupervised({
    args: budgetedArgv(budget.ulimitProcesses, options.vizeBin, args),
    capture: true,
    command: "/bin/sh",
    cwd: path.join(options.repoRoot, target.projectDir),
    env: process.env,
    sampleIntervalMs: options.sampleIntervalMs,
    startedAt: options.startedAt,
  });

  const outputSha256 = createHash("sha256").update(result.stdout).digest("hex");
  const peakCorsaProcesses = countCorsa(result.samples.peak);
  const reportedEagain = reportsEagain(result.stderr) || reportsEagain(result.stdout);
  const failures: string[] = [];

  if (result.spawnError != null) {
    failures.push(`spawn failed: ${result.spawnError}`);
  }
  if (budget.applied !== budget.ulimitProcesses) {
    failures.push(
      `constrained budget not applied: asked for ulimit -u ${budget.ulimitProcesses}, read back ${String(budget.applied)}`,
    );
  }
  if (result.exitCode === BUDGET_NOT_APPLIED_EXIT) {
    failures.push(`the budget prologue could not lower RLIMIT_NPROC to ${budget.ulimitProcesses}`);
  }
  if (reportedEagain) {
    failures.push(
      `reported EAGAIN under budget ${budget.ulimitProcesses}: ${result.stderr.trim()}`,
    );
  }
  if (options.expectedExitCode != null && result.exitCode !== options.expectedExitCode) {
    failures.push(`exit code ${String(result.exitCode)} != ${options.expectedExitCode}`);
  }
  if (options.expectedSha256 != null && outputSha256 !== options.expectedSha256) {
    failures.push(`output sha256 ${outputSha256} != ${options.expectedSha256}`);
  }
  failures.push(
    ...outputFailures(result.stdout, {
      diagnostics: target.expectsDiagnostics,
      fileCount: target.expectedFileCount,
    }),
  );
  if (result.samples.peak.group.processes.length > target.peakGroupProcessBound) {
    failures.push(
      `peak group processes ${result.samples.peak.group.processes.length} > ${target.peakGroupProcessBound}`,
    );
  }
  if (result.samples.peak.group.liveTasks > taskBudget) {
    failures.push(`peak group tasks ${result.samples.peak.group.liveTasks} > ${taskBudget}`);
  }
  if (peakCorsaProcesses > target.corsaProcesses) {
    failures.push(`peak Corsa processes ${peakCorsaProcesses} > ${target.corsaProcesses}`);
  }
  if (result.survivors.length > 0) {
    failures.push(`${result.survivors.length} task(s) outlived the cycle`);
  }
  if (result.samples.after.group.processes.length > 0) {
    failures.push(
      `${result.samples.after.group.processes.length} process(es) left in the cycle group`,
    );
  }

  return {
    budget,
    durationMs: result.durationMs,
    exitCode: result.exitCode,
    failures,
    index,
    outputBytes: result.stdout.length,
    outputSha256,
    peakCorsaProcesses,
    peakGroupLiveTasks: result.samples.peak.group.liveTasks,
    peakGroupProcesses: result.samples.peak.group.processes.length,
    reportedEagain,
    samples: result.samples,
    signal: result.signal,
    stderrExcerpt:
      failures.length === 0 ? null : `${result.stderr}${result.stdout}`.slice(0, 2000).trim(),
    survivors: result.survivors,
  };
}

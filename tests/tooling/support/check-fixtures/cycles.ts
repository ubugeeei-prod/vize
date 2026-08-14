//! Focused process-budget cycles for the Vue parity lane (#4126).
//!
//! Usage (from the repository root or `tests/`):
//!
//! ```sh
//! node tests/tooling/support/check-fixtures/cycles.ts
//! node tests/tooling/support/check-fixtures/cycles.ts --cycles 3 --only single-corsa
//! ```
//!
//! Each target is checked `--cycles` times under an explicit `ulimit -u` budget
//! derived from the live task count plus that target's documented headroom. A
//! cycle is red when the checker reports `EAGAIN`, when its diagnostics, ranges
//! or output hash drift, when it exceeds the documented peak, or when it leaves
//! a task behind. Nothing here retries, sleeps, or widens the runner.

import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  checkArgv,
  type CycleTarget,
  cycleTargets,
  resolveCorsaBin,
  resolveVizeBin,
} from "./cycle-targets.ts";
import { type CycleRecord, runCycle } from "./cycle-runner.ts";
import { type CyclesReport, createCyclesReport, writeCyclesReport } from "./cycles-report.ts";

const HERE = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(HERE, "../../../..");
export const DEFAULT_METRICS_DIR = path.join(
  REPO_ROOT,
  "target",
  "vize-tests",
  "metrics",
  "check-fixtures-cycles",
);
export const DEFAULT_CYCLES = 20;
export const DEFAULT_SAMPLE_INTERVAL_MS = 25;
export const BUDGET_CPU_FLOOR_ENV = "VIZE_CHECK_FIXTURES_BUDGET_CPU_FLOOR";

export type CyclesOptions = {
  readonly targets?: readonly CycleTarget[];
  readonly cycles?: number;
  readonly metricsDir?: string;
  readonly sampleIntervalMs?: number;
  readonly repoRoot?: string;
};

export function resolveBudgetCpuCount(
  cpuCount: number,
  environment: NodeJS.ProcessEnv = process.env,
): number {
  const floor = environment[BUDGET_CPU_FLOOR_ENV];
  if (floor == null || floor === "") {
    return cpuCount;
  }
  const parsed = Number(floor);
  if (!Number.isInteger(parsed) || parsed < 1) {
    throw new Error(`${BUDGET_CPU_FLOOR_ENV} must be a positive integer, received: ${floor}`);
  }
  return Math.max(cpuCount, parsed);
}

/**
 * Cycles a target really runs.
 *
 * A target's own cap only ever narrows `--cycles`, never widens it, so
 * `--cycles 1` stays a one-cycle smoke run for every target.
 */
export function resolveTargetCycles(target: CycleTarget, cycles: number): number {
  return target.maxCycles == null ? cycles : Math.min(cycles, target.maxCycles);
}

function summarizeTarget(target: CycleTarget, records: readonly CycleRecord[]): string[] {
  const failures = records.flatMap((record) =>
    record.failures.map((failure) => `cycle ${record.index}: ${failure}`),
  );
  const observedCorsa = Math.max(0, ...records.map((record) => record.peakCorsaProcesses));
  // A cap that is never reached would make the bound vacuous, so the widest
  // observed width has to match the width the target claims to exercise.
  if (records.length > 0 && observedCorsa !== target.corsaProcesses) {
    failures.push(
      `widest observed Corsa width ${observedCorsa} != documented ${target.corsaProcesses}`,
    );
  }
  return failures;
}

/** Run every target for `cycles` cycles and return the report. */
export async function runCycles(options: CyclesOptions = {}): Promise<CyclesReport> {
  const repoRoot = options.repoRoot ?? REPO_ROOT;
  const targets = options.targets ?? cycleTargets;
  const cycles = options.cycles ?? DEFAULT_CYCLES;
  const metricsDir = options.metricsDir ?? DEFAULT_METRICS_DIR;
  const sampleIntervalMs = options.sampleIntervalMs ?? DEFAULT_SAMPLE_INTERVAL_MS;
  const cpuCount = os.availableParallelism();
  const budgetCpuCount = resolveBudgetCpuCount(cpuCount);
  const vizeBin = resolveVizeBin(repoRoot);
  const corsaBin = resolveCorsaBin(repoRoot);
  const report = createCyclesReport({
    budgetCpuCount,
    corsaBin,
    cycles,
    repoRoot,
    vizeBin,
  });
  writeCyclesReport(metricsDir, report);

  const startedAt = performance.now();
  for (const target of targets) {
    const records: CycleRecord[] = [];
    const targetCycles = resolveTargetCycles(target, cycles);
    const entry = {
      argv: [vizeBin, ...checkArgv(target, corsaBin)],
      bounds: {
        corsaProcesses: target.corsaProcesses,
        peakGroupProcesses: target.peakGroupProcessBound,
        taskBudget: target.taskBudget(budgetCpuCount),
      },
      cycles: records,
      description: target.description,
      failures: [] as string[],
      id: target.id,
      plannedCycles: targetCycles,
      projectDir: target.projectDir,
      status: "running" as CyclesReport["targets"][number]["status"],
    };
    report.targets.push(entry);

    for (let index = 1; index <= targetCycles; index += 1) {
      const first = records[0];
      const record = await runCycle(target, index, {
        corsaBin,
        budgetCpuCount,
        expectedExitCode: first?.exitCode ?? null,
        expectedSha256: first?.outputSha256 ?? null,
        repoRoot,
        sampleIntervalMs,
        startedAt,
        vizeBin,
      });
      records.push(record);
      console.log(
        `[check-fixtures-cycles] ${target.id} cycle ${index}/${targetCycles}: ${record.durationMs}ms, ` +
          `peak ${record.peakGroupProcesses} procs / ${record.peakGroupLiveTasks} tasks, ` +
          `${record.peakCorsaProcesses} corsa, ulimit -u ${record.budget.ulimitProcesses}` +
          (record.failures.length === 0 ? "" : ` — ${record.failures.join("; ")}`) +
          (record.stderrExcerpt == null || record.stderrExcerpt.length === 0
            ? ""
            : `\n  child output: ${record.stderrExcerpt}`),
      );
      writeCyclesReport(metricsDir, report);
    }

    entry.failures = summarizeTarget(target, records);
    entry.status = entry.failures.length === 0 ? "passed" : "failed";
    writeCyclesReport(metricsDir, report);
  }

  report.finishedAtIso = new Date().toISOString();
  report.status = report.targets.every((target) => target.status === "passed")
    ? "passed"
    : "failed";
  writeCyclesReport(metricsDir, report);
  return report;
}

/**
 * Parse a count that must be at least one.
 *
 * `Number.parseInt` alone turns `--cycles nope` into `NaN`, and every
 * `index <= NaN` comparison is false, so the harness would run no cycles at all
 * and still report the target as passed. Refuse the input instead.
 */
function parsePositiveInt(flag: string, value: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 1) {
    throw new Error(`${flag} expects an integer of at least 1, received: ${value}`);
  }
  return parsed;
}

export function parseCyclesArgs(argv: readonly string[]): {
  cycles: number | undefined;
  metricsDir: string | undefined;
  only: string[];
} {
  const only: string[] = [];
  let cycles: number | undefined;
  let metricsDir: string | undefined;
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    const value = argv[index + 1];
    if (flag === "--cycles" && value != null) {
      cycles = parsePositiveInt(flag, value);
      index += 1;
    } else if (flag === "--metrics-dir" && value != null) {
      metricsDir = value;
      index += 1;
    } else if (flag === "--only" && value != null) {
      only.push(value);
      index += 1;
    } else {
      throw new Error(`unknown cycles argument: ${String(flag)}`);
    }
  }
  return { cycles, metricsDir, only };
}

/** Resolve `--only` ids against the target list, failing closed on a typo. */
export function selectTargets(only: readonly string[]): readonly CycleTarget[] {
  if (only.length === 0) {
    return cycleTargets;
  }
  return only.map((id) => {
    const target = cycleTargets.find((candidate) => candidate.id === id);
    if (target == null) {
      throw new Error(`unknown cycle target: ${id}`);
    }
    return target;
  });
}

async function main(): Promise<void> {
  const args = parseCyclesArgs(process.argv.slice(2));
  const report = await runCycles({
    cycles: args.cycles,
    metricsDir: args.metricsDir,
    targets: selectTargets(args.only),
  });
  for (const target of report.targets) {
    for (const failure of target.failures) {
      console.error(`[check-fixtures-cycles] ${target.id}: ${failure}`);
    }
  }
  if (report.status !== "passed") {
    process.exitCode = 1;
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await main();
}

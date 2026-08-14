//! The focused-cycle artifact (#4126).

import fs from "node:fs";
import path from "node:path";

import type { CycleRecord } from "./cycle-runner.ts";
import { writeAtomic } from "./report.ts";
import { readRunnerFacts, type RunnerFacts } from "./runner-facts.ts";

export const CYCLES_SCHEMA = "vize.check-fixtures.cycles/1";

export type CyclesTargetReport = {
  readonly id: string;
  readonly description: string;
  readonly projectDir: string;
  readonly argv: readonly string[];
  readonly bounds: {
    readonly peakGroupProcesses: number;
    readonly taskBudget: number;
    readonly corsaProcesses: number;
  };
  readonly cycles: CycleRecord[];
  /**
   * Cycles this target was scheduled to run.
   *
   * Reported per target because a target may cap itself below the run-wide
   * `cyclesPerTarget`, and evidence that reads `20` for a target that ran three
   * cycles would overstate what the artifact proves.
   */
  readonly plannedCycles: number;
  failures: string[];
  status: "running" | "passed" | "failed";
};

export type CyclesReport = {
  readonly schema: string;
  readonly startedAtIso: string;
  finishedAtIso: string | null;
  readonly repoRoot: string;
  readonly vizeBin: string;
  readonly corsaBin: string;
  readonly cyclesPerTarget: number;
  readonly budgetCpuCount: number;
  readonly nodeVersion: string;
  readonly runner: RunnerFacts;
  readonly targets: CyclesTargetReport[];
  status: "running" | "passed" | "failed";
};

export function createCyclesReport(options: {
  readonly repoRoot: string;
  readonly vizeBin: string;
  readonly corsaBin: string;
  readonly cycles: number;
  readonly budgetCpuCount: number;
}): CyclesReport {
  return {
    budgetCpuCount: options.budgetCpuCount,
    corsaBin: options.corsaBin,
    cyclesPerTarget: options.cycles,
    finishedAtIso: null,
    nodeVersion: process.version,
    repoRoot: options.repoRoot,
    runner: readRunnerFacts(),
    schema: CYCLES_SCHEMA,
    startedAtIso: new Date().toISOString(),
    status: "running",
    targets: [],
    vizeBin: options.vizeBin,
  };
}

/** Longest failure text kept in one summary entry. */
const MAX_CELL_CHARS = 300;

/**
 * Flatten a failure into one bounded line.
 *
 * Failures quote process argv and stderr, so they arrive with newlines and can
 * be long enough to bury the table under one bad cycle.
 */
function flatten(value: string): string {
  const single = value.replace(/\r?\n/g, "<br>");
  return single.length <= MAX_CELL_CHARS
    ? single
    : `${single.slice(0, MAX_CELL_CHARS)} (truncated)`;
}

/**
 * Render a Markdown table cell.
 *
 * An unescaped `|` from a quoted command or stderr line would start a new
 * column and shift every later value into the wrong header.
 */
function cell(value: string): string {
  return flatten(value).replace(/\|/g, "\\|");
}

function targetRows(target: CyclesTargetReport): string[] {
  return target.cycles.map((cycle: CycleRecord) =>
    [
      "",
      target.id,
      String(cycle.index),
      String(cycle.durationMs),
      String(cycle.budget.ulimitProcesses),
      String(cycle.peakGroupProcesses),
      String(cycle.peakGroupLiveTasks),
      String(cycle.peakCorsaProcesses),
      cycle.outputSha256.slice(0, 12),
      cycle.reportedEagain ? "**yes**" : "no",
      cycle.failures.length === 0 ? "ok" : cell(cycle.failures.join("\n")),
      "",
    ].join(" | "),
  );
}

/** Render the Markdown appended to `$GITHUB_STEP_SUMMARY`. */
export function renderCyclesSummary(report: CyclesReport): string {
  const { cgroupPids, cpuCount, platform, ulimitProcesses } = report.runner;
  const lines = [
    "## Vue parity focused process-budget cycles",
    "",
    `- status: **${report.status}**`,
    `- runner: ${platform}, ${cpuCount} CPUs, \`ulimit -u\` ${ulimitProcesses ?? "unavailable"}`,
    `- budget CPU count: ${report.budgetCpuCount}`,
    `- cgroup \`pids.current\` / \`pids.max\`: ${cgroupPids.current ?? "unavailable"} / ${cgroupPids.max ?? "unavailable"}`,
    `- cycles requested per target: ${report.cyclesPerTarget}`,
    "",
    "| target | cycle | ms | ulimit -u | peak procs | peak tasks | corsa | sha256 | EAGAIN | verdict |",
    "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
  ];
  for (const target of report.targets) {
    lines.push(...targetRows(target));
  }
  lines.push("");
  for (const target of report.targets) {
    lines.push(
      `- \`${target.id}\`: ${target.status} over ${target.cycles.length}/${target.plannedCycles} ` +
        `cycles — ${target.description}`,
    );
    for (const failure of target.failures) {
      lines.push(`  - ${flatten(failure)}`);
    }
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

/** Write `cycles.json` and `summary.md` into `directory`. */
export function writeCyclesReport(directory: string, report: CyclesReport): void {
  fs.mkdirSync(directory, { recursive: true });
  writeAtomic(path.join(directory, "cycles.json"), `${JSON.stringify(report, null, 2)}\n`);
  writeAtomic(path.join(directory, "summary.md"), renderCyclesSummary(report));
}

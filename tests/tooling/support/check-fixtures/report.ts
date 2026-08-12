//! The always-uploaded process-topology artifact (#4126).
//!
//! Written after every phase, not once at the end: the failure this lane has to
//! explain kills the process mid-run, and a report that only exists on a clean
//! exit would be missing exactly when it is needed.

import fs from "node:fs";
import path from "node:path";

import { describeSurvivor } from "./topology.ts";
import type { PhaseOutcome } from "./phase-runner.ts";
import type { RunnerFacts } from "./runner-facts.ts";

export const REPORT_SCHEMA = "vize.check-fixtures.topology/1";

export type SupervisorReport = {
  readonly schema: string;
  readonly startedAtIso: string;
  finishedAtIso: string | null;
  readonly cwd: string;
  readonly supervisorPid: number;
  readonly nodeVersion: string;
  /** Runner facts sampled before the first phase. */
  readonly runner: RunnerFacts;
  readonly phases: PhaseOutcome[];
  status: "running" | "passed" | "failed";
};

export function createReport(runner: RunnerFacts, cwd: string): SupervisorReport {
  return {
    cwd,
    finishedAtIso: null,
    nodeVersion: process.version,
    phases: [],
    runner,
    schema: REPORT_SCHEMA,
    startedAtIso: new Date().toISOString(),
    status: "running",
    supervisorPid: process.pid,
  };
}

function formatCgroup(runner: RunnerFacts): string {
  const { current, max } = runner.cgroupPids;
  return `${current ?? "unavailable"} / ${max ?? "unavailable"}`;
}

/** Render the Markdown appended to `$GITHUB_STEP_SUMMARY`. */
export function renderSummary(report: SupervisorReport): string {
  const lines = [
    "## Vue parity fixture process topology",
    "",
    `- status: **${report.status}**`,
    `- runner: ${report.runner.platform}, ${report.runner.cpuCount} CPUs`,
    `- \`ulimit -u\`: ${report.runner.ulimitProcesses ?? "unavailable"}`,
    `- cgroup \`pids.current\` / \`pids.max\`: ${formatCgroup(report.runner)}`,
    "",
    "| phase | status | ms | tasks before | peak group tasks | tasks after | survivors |",
    "| --- | --- | --- | --- | --- | --- | --- |",
  ];
  for (const phase of report.phases) {
    lines.push(
      [
        "",
        phase.id,
        phase.status,
        String(phase.durationMs),
        String(phase.samples.before.liveTasks),
        String(phase.samples.peak.group.liveTasks),
        String(phase.samples.after.liveTasks),
        phase.survivors.length === 0 ? "none" : phase.survivors.map(describeSurvivor).join("<br>"),
        "",
      ].join(" | "),
    );
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

/** Write `topology.json` and `summary.md` into `directory`. */
export function writeReport(directory: string, report: SupervisorReport): void {
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(path.join(directory, "topology.json"), `${JSON.stringify(report, null, 2)}\n`);
  fs.writeFileSync(path.join(directory, "summary.md"), renderSummary(report));
}

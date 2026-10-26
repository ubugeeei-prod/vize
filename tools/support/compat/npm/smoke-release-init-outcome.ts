/** Actual child outcomes for the fresh-project release check (#3956). */

import { renderOutput, type ProcessResult } from "./smoke-process.ts";

/** One file of `vize check --format json`. */
export interface CheckReportFile {
  readonly file: string;
  readonly diagnostics: readonly string[];
}

/** The parts of `vize check --format json` the release smoke reads. */
export interface CheckReport {
  readonly files: readonly CheckReportFile[];
  readonly errorCount: number;
}

export interface MachineCheckReport {
  readonly rendered: string;
  readonly report: CheckReport;
  readonly status: number | null;
  readonly signal: NodeJS.Signals | null;
  readonly outcome: ProcessResult;
}

function renderInvocation(
  command: string,
  args: readonly string[],
  cwd: string,
  result: ProcessResult,
): string {
  const rendered = renderOutput(result);
  return [
    `command: ${JSON.stringify([command, ...args])}`,
    `cwd: ${cwd}`,
    `status: ${result.status ?? "<null>"}`,
    `signal: ${result.signal ?? "<none>"}`,
    rendered === "" ? "stdout/stderr: <empty>" : rendered,
  ].join("\n");
}

export function machineCheckReport(
  command: string,
  args: readonly string[],
  cwd: string,
  outcome: ProcessResult,
): MachineCheckReport {
  const rendered = renderInvocation(command, args, cwd, outcome);
  if (outcome.signal !== null || (outcome.status !== 0 && outcome.status !== 1)) {
    throw Object.assign(
      new Error(`project-local vize check did not exit normally\n${rendered}`, { cause: outcome }),
      { outcome },
    );
  }
  let report: CheckReport;
  try {
    report = JSON.parse(outcome.stdout) as CheckReport;
  } catch (error) {
    throw Object.assign(
      new Error(`project-local vize check did not produce JSON\n${rendered}`, { cause: error }),
      { outcome },
    );
  }
  return { rendered, report, status: outcome.status, signal: outcome.signal, outcome };
}

/** Actual child outcomes for the fresh-project release check (#3956). */

import { renderOutput } from "./smoke-process.mjs";

function renderInvocation(command, args, cwd, result) {
  const rendered = renderOutput(result);
  return [
    `command: ${JSON.stringify([command, ...args])}`,
    `cwd: ${cwd}`,
    `status: ${result.status ?? "<null>"}`,
    `signal: ${result.signal ?? "<none>"}`,
    rendered === "" ? "stdout/stderr: <empty>" : rendered,
  ].join("\n");
}

export function machineCheckReport(command, args, cwd, outcome) {
  const rendered = renderInvocation(command, args, cwd, outcome);
  if (outcome.signal !== null || (outcome.status !== 0 && outcome.status !== 1)) {
    throw Object.assign(
      new Error(`project-local vize check did not exit normally\n${rendered}`, { cause: outcome }),
      { outcome },
    );
  }
  let report;
  try {
    report = JSON.parse(outcome.stdout);
  } catch (error) {
    throw Object.assign(
      new Error(`project-local vize check did not produce JSON\n${rendered}`, { cause: error }),
      { outcome },
    );
  }
  return { rendered, report, status: outcome.status, signal: outcome.signal, outcome };
}

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

import { displayCommand, toolArgs, typecheckCorpusGlobs } from "./tool-matrix-command.mjs";
import { collectTypecheckerAuthoredPaths, collectVueInputPaths } from "./tool-matrix-inputs.mjs";
import { validateTypecheckerOutput } from "./tool-matrix-typechecker.mjs";
import { runTypecheckCommand } from "./typecheck-command-runner.mjs";
import { compareTypecheckDiagnostics } from "./typecheck-divergence.mjs";

export async function observeMutationState({
  name,
  project,
  fixtureRoot,
  file,
  sourcePath,
  vizeLaunch,
  vueTsc,
  baselineArgs,
  documentedDifferences,
}) {
  const sourceSha256 = sha256(readFileSync(sourcePath));
  const vize = await runVizeTypecheck(project, fixtureRoot, vizeLaunch);
  assertSourceUnchanged(name, file, sourcePath, sourceSha256, "Vize");
  const baseline = await runVueTsc(project, fixtureRoot, vueTsc, baselineArgs);
  assertSourceUnchanged(name, file, sourcePath, sourceSha256, "vue-tsc");
  return {
    sourceSha256,
    vize,
    baseline,
    comparison: compareTypecheckDiagnostics({
      projectId: project.id,
      cwd: fixtureRoot,
      vizeReport: vize.parsed,
      vueTscOutput: baseline.output,
      documentedDifferences,
    }),
  };
}

async function runVizeTypecheck(project, fixtureRoot, launch) {
  const args = [...launch.prefix, ...toolArgs(project, "typechecker", "<compiler-output>")];
  const result = await runTypecheckCommand(launch.command, args, {
    cwd: fixtureRoot,
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeoutMs: project.typecheckPerformance.hangTimeoutMs,
  });
  if (result.error != null) {
    throw new Error(`Vize mutation run failed: ${errorMessage(result.error)}`);
  }
  if (result.status !== 0 && result.status !== 1) {
    throw new Error(`Vize mutation run exited with unsupported status ${result.status}`);
  }
  let parsed;
  try {
    parsed = JSON.parse(result.stdout ?? "");
  } catch (error) {
    throw new Error(`Vize mutation run emitted invalid JSON: ${errorMessage(error)}`);
  }
  validateTypecheckerOutput(
    project,
    parsed,
    result.status,
    collectVueInputPaths(fixtureRoot, typecheckCorpusGlobs(project)),
    collectTypecheckerAuthoredPaths(fixtureRoot),
  );
  return executionEvidence({
    command: displayCommand(launch.command, args),
    exitCode: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
    parsed,
  });
}

async function runVueTsc(project, fixtureRoot, vueTsc, args) {
  const result = await runTypecheckCommand(vueTsc.path, args, {
    cwd: fixtureRoot,
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeoutMs: project.typecheckPerformance.hangTimeoutMs,
  });
  if (result.error != null) {
    throw new Error(`vue-tsc mutation run failed: ${errorMessage(result.error)}`);
  }
  if (![0, 1, 2].includes(result.status)) {
    throw new Error(`vue-tsc mutation run exited with unsupported status ${result.status}`);
  }
  const stdout = result.stdout ?? "";
  const stderr = result.stderr ?? "";
  return executionEvidence({
    command: displayCommand(vueTsc.path, args),
    exitCode: result.status,
    stdout,
    stderr,
    output: `${stdout}\n${stderr}`,
  });
}

function executionEvidence({ command, exitCode, stdout, stderr, parsed, output }) {
  return {
    command,
    exitCode,
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
    ...(parsed == null ? {} : { parsed }),
    ...(output == null ? {} : { output }),
  };
}

function assertSourceUnchanged(state, file, sourcePath, expectedSha256, tool) {
  const actual = sha256(readFileSync(sourcePath));
  if (actual !== expectedSha256) {
    throw new Error(`${tool} mutated ${file} during seeded ${state} oracle run`);
  }
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { resolveVizeLaunch } from "./tool-matrix-run.mjs";
import { validateTypecheckPerformanceTarget } from "./tool-matrix-typecheck-target.mjs";
import { selectTypecheckPerformanceProjects } from "./typecheck-performance-shard.mjs";
import { evaluateBaselineAmbientEnvironment } from "./typecheck-baseline-ambient.mjs";
import { evaluateBaselineConfiguration } from "./typecheck-baseline-configuration.mjs";
import { materializeBaselineProject } from "./typecheck-baseline-project.mjs";
import { applyIsolatedJsxBaseline } from "./typecheck-baseline-outside-jsx.mjs";
import { typecheckSourceTsconfig } from "./tool-matrix-command.mjs";
import { runVueTscBaseline } from "./typecheck-baseline-run.mjs";
import { evaluateVueProgramCoverage } from "./typecheck-baseline-coverage.mjs";
import { runTypecheckCommand } from "./typecheck-command-runner.mjs";
import { assertBudgetsPassed, evaluateBudget } from "./typecheck-divergence-budget.mjs";
import { renderMarkdown } from "./typecheck-divergence-markdown.mjs";
import {
  createSeededMutationOracle,
  readAndValidateDependencyPreparation,
} from "./typecheck-divergence-provenance.mjs";
import { parseArgs } from "./typecheck-divergence-report-args.mjs";
import {
  readAndValidateSummary,
  readAndValidateVizeRun,
} from "./typecheck-divergence-report-inputs.mjs";
import { compareTypecheckDiagnostics } from "./typecheck-divergence.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const defaultRegistry = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const documentedDifferencesPath = join(
  repoRoot,
  "tests",
  "_fixtures",
  "compat-documented-differences.json",
);

export async function runTypecheckDivergenceReport(argv = process.argv.slice(2)) {
  const args = parseArgs(argv, { repoRoot, defaultRegistry });
  const registry = readJson(args.registry);
  const selected = selectTypecheckPerformanceProjects(registry, args);
  if (selected.length === 0) {
    process.stdout.write(
      `No typecheck performance projects selected for shard ${args.shardIndex}/${args.shardCount}\n`,
    );
    return [];
  }
  const artifacts = [];
  const documentedDifferences = readDocumentedDifferences();
  const vizeLaunch = resolveVizeLaunch(args.vizeBin, false);
  const vueTsc = await resolveVueTsc(args.vueTscBin);
  for (const project of selected) {
    validatePerformanceConfig(project.typecheckPerformance);
    const fixtureRoot = resolve(repoRoot, project.fixturePath);
    validateTypecheckPerformanceTarget(project, fixtureRoot, { requireBaseline: true });
    const summary = readAndValidateSummary(args.reportDir, project);
    const preparation = readAndValidateDependencyPreparation({
      reportDir: args.reportDir,
      project,
      fixtureRoot,
      commitSha: summary.evidence.commitSha,
    });
    const vizeRun = readAndValidateVizeRun(args.reportDir, project, summary);
    const baselineProject = materializeBaselineProject(
      fixtureRoot,
      args.reportDir,
      project,
      vizeRun.payload.parsed,
    );
    const sourceTsconfig = typecheckSourceTsconfig(project);
    if (sourceTsconfig != null) {
      applyIsolatedJsxBaseline(
        fixtureRoot,
        resolve(fixtureRoot, sourceTsconfig),
        baselineProject.path,
      );
    }
    const baselineArgs = ["--noEmit", "--pretty", "false", "-p", baselineProject.path];
    const coverageArgs = [
      "--noEmit",
      "--pretty",
      "false",
      "--listFilesOnly",
      "-p",
      baselineProject.path,
    ];
    const baseline = await runVueTscBaseline({
      vueTsc,
      args: baselineArgs,
      cwd: fixtureRoot,
      timeoutMs: project.typecheckPerformance.hangTimeoutMs,
      label: "vue-tsc baseline",
    });
    const coverageBaseline = await runVueTscBaseline({
      vueTsc,
      args: coverageArgs,
      cwd: fixtureRoot,
      timeoutMs: project.typecheckPerformance.hangTimeoutMs,
      label: "vue-tsc coverage baseline",
    });

    const divergence = compareTypecheckDiagnostics({
      projectId: project.id,
      cwd: fixtureRoot,
      vizeReport: vizeRun.payload.parsed,
      vueTscOutput: baseline.output,
      documentedDifferences,
    });
    const coverage = evaluateVueProgramCoverage(
      vizeRun.payload.parsed,
      coverageBaseline.output,
      fixtureRoot,
    );
    const configuration = evaluateBaselineConfiguration(baseline.output);
    // Coverage proves both tools loaded the same `.vue` files; this proves the
    // baseline loaded them against the fixture's own type environment.
    const ambient = evaluateBaselineAmbientEnvironment(coverageBaseline.output, fixtureRoot);
    const mutationOracle = await createSeededMutationOracle({
      project,
      fixtureRoot,
      vizeReport: vizeRun.payload.parsed,
      coverage,
      configuration,
      vizeLaunch,
      vueTsc,
      baselineArgs,
      documentedDifferences,
    });
    const budget = evaluateBudget(
      project.typecheckPerformance,
      divergence.summary,
      coverage,
      configuration,
      mutationOracle,
      ambient,
    );
    const artifact = {
      schema: "vize.fixtureTypecheckDivergenceRun",
      version: 6,
      project: project.id,
      revision: project.revision,
      tsconfig: baselineProject.sourceProject,
      evidence: summary.evidence,
      enforcement: {
        budgetMode: args.budgetMode,
      },
      preparation,
      source: vizeRun.source,
      baseline: {
        command: displayCommand(vueTsc.path, baselineArgs),
        coverageCommand: displayCommand(vueTsc.path, coverageArgs),
        configSha256: sha256(baselineProject.source),
        sourceConfigSha256: sha256(
          readFileSync(resolve(fixtureRoot, baselineProject.sourceProject)),
        ),
        version: vueTsc.version,
        durationMs: baseline.durationMs,
        coverageDurationMs: coverageBaseline.durationMs,
        exitCode: baseline.exitCode,
        coverageExitCode: coverageBaseline.exitCode,
        ambient,
        configuration,
        coverage,
        stdoutSha256: sha256(baseline.stdout),
        stderrSha256: sha256(baseline.stderr),
        coverageStdoutSha256: sha256(coverageBaseline.stdout),
        coverageStderrSha256: sha256(coverageBaseline.stderr),
      },
      mutationOracle,
      budget,
      divergence,
    };
    const jsonPath = join(args.reportDir, `${project.id}-typecheck-divergence.json`);
    const markdownPath = join(args.reportDir, `${project.id}-typecheck-divergence.md`);
    writeFileSync(jsonPath, `${JSON.stringify(artifact, null, 2)}\n`);
    writeFileSync(markdownPath, renderMarkdown(artifact));
    process.stdout.write(`Wrote ${relative(repoRoot, jsonPath)}\n`);
    process.stdout.write(`Wrote ${relative(repoRoot, markdownPath)}\n`);
    artifacts.push(artifact);
  }
  assertBudgetsPassed(artifacts, args.budgetMode);
  return artifacts;
}

function readDocumentedDifferences() {
  // The per-PR compat ratchet and this weekly report share one reviewed ledger,
  // so an expected difference can never be recorded on only one of the two gates.
  const ledger = readJson(documentedDifferencesPath);
  if (ledger.schema !== "vize.compatDocumentedDifferences" || ledger.version !== 1) {
    throw new Error("Documented difference ledger schema is unsupported");
  }
  if (!Array.isArray(ledger.differences)) {
    throw new Error("Documented difference ledger must list differences");
  }
  return ledger.differences;
}

function validatePerformanceConfig(performance) {
  if (!Number.isSafeInteger(performance.hangTimeoutMs) || performance.hangTimeoutMs <= 0) {
    throw new Error("typecheckPerformance.hangTimeoutMs must be a positive safe integer");
  }
  ratio(performance.maxFalsePositiveRatio, "maxFalsePositiveRatio");
  ratio(performance.maxFalseNegativeRatio, "maxFalseNegativeRatio");
}

async function resolveVueTsc(value) {
  const candidate = isAbsolute(value) ? value : resolve(repoRoot, value);
  const probe = await runTypecheckCommand(candidate, ["--version"], {
    cwd: repoRoot,
    env: process.env,
    maxBuffer: 1024 * 1024,
    timeoutMs: 10_000,
  });
  const version = (probe.stdout ?? "").trim();
  if (probe.error != null || probe.status !== 0 || version === "")
    throw new Error(`vue-tsc is not runnable: ${value}`);
  return { path: candidate, version };
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function ratio(value, name) {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0 || value > 1) {
    throw new Error(`typecheckPerformance.${name} must be a finite number between 0 and 1`);
  }
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function displayCommand(command, args) {
  return [command, ...args].join(" ");
}

const entrypoint =
  process.argv[1] != null && fileURLToPath(import.meta.url) === resolve(process.argv[1]);
if (entrypoint) {
  try {
    await runTypecheckDivergenceReport();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exit(1);
  }
}

#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { validateTypecheckerOutput } from "./tool-matrix-typechecker.mjs";
import { validateTypecheckPerformanceTarget } from "./tool-matrix-typecheck-target.mjs";
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

export function runTypecheckDivergenceReport(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const registry = readJson(args.registry);
  const selected = registry.projects
    .filter((_, index) => index % args.shardCount === args.shardIndex)
    .filter((project) => project.typecheckPerformance?.enabled === true);
  if (selected.length !== 1) {
    throw new Error(
      `Expected exactly one typecheck performance project in shard ${args.shardIndex}/${args.shardCount}, found ${selected.length}`,
    );
  }

  const project = selected[0];
  validatePerformanceConfig(project.typecheckPerformance);
  const fixtureRoot = resolve(repoRoot, project.fixturePath);
  validateTypecheckPerformanceTarget(project, fixtureRoot);
  const summary = readAndValidateSummary(args.reportDir, project);
  const vizeRun = readAndValidateVizeRun(args.reportDir, project, summary);
  const vueTsc = resolveVueTsc(args.vueTscBin);
  const baselineArgs = ["--noEmit", "--pretty", "false", "-p", project.tsconfig];
  const startedAt = Date.now();
  const baseline = spawnSync(vueTsc.path, baselineArgs, {
    cwd: fixtureRoot,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: project.typecheckPerformance.hangTimeoutMs,
  });
  const durationMs = Date.now() - startedAt;
  if (baseline.error != null)
    throw new Error(`vue-tsc failed to run: ${errorMessage(baseline.error)}`);
  if (baseline.status !== 0 && baseline.status !== 2) {
    throw new Error(`vue-tsc exited with unsupported status ${baseline.status}`);
  }

  const divergence = compareTypecheckDiagnostics({
    projectId: project.id,
    cwd: fixtureRoot,
    vizeReport: vizeRun.payload.parsed,
    vueTscOutput: `${baseline.stdout ?? ""}\n${baseline.stderr ?? ""}`,
    documentedDifferences: readDocumentedDifferences(),
  });
  const budget = evaluateBudget(project.typecheckPerformance, divergence.summary);
  const artifact = {
    schema: "vize.fixtureTypecheckDivergenceRun",
    version: 1,
    project: project.id,
    revision: project.revision,
    tsconfig: project.tsconfig,
    evidence: summary.evidence,
    source: vizeRun.source,
    baseline: {
      command: displayCommand(vueTsc.path, baselineArgs),
      version: vueTsc.version,
      durationMs,
      exitCode: baseline.status,
      stdoutSha256: sha256(baseline.stdout ?? ""),
      stderrSha256: sha256(baseline.stderr ?? ""),
    },
    budget,
    divergence,
  };
  const jsonPath = join(args.reportDir, `${project.id}-typecheck-divergence.json`);
  const markdownPath = join(args.reportDir, `${project.id}-typecheck-divergence.md`);
  writeFileSync(jsonPath, `${JSON.stringify(artifact, null, 2)}\n`);
  writeFileSync(markdownPath, renderMarkdown(artifact));
  process.stdout.write(`Wrote ${relative(repoRoot, jsonPath)}\n`);
  process.stdout.write(`Wrote ${relative(repoRoot, markdownPath)}\n`);
  return artifact;
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

function readAndValidateSummary(reportDir, project) {
  const summary = readJson(join(reportDir, "summary.json"));
  if (summary.schema !== "vize.fixtureToolMatrixReport" || summary.version !== 2) {
    throw new Error("Fixture matrix summary schema is unsupported");
  }
  if (!/^[0-9a-f]{40}$/.test(summary.evidence?.commitSha ?? "")) {
    throw new Error("Fixture matrix summary is missing exact commit evidence");
  }
  if (process.env.GITHUB_SHA != null && summary.evidence.commitSha !== process.env.GITHUB_SHA) {
    throw new Error("Fixture matrix summary commit does not match GITHUB_SHA");
  }
  const projectSummary = summary.projects?.filter((entry) => entry.id === project.id) ?? [];
  if (projectSummary.length !== 1 || projectSummary[0].revision !== project.revision) {
    throw new Error(`Fixture matrix summary does not contain pinned project ${project.id}`);
  }
  return summary;
}

function readAndValidateVizeRun(reportDir, project, summary) {
  const projectSummary = summary.projects.find((entry) => entry.id === project.id);
  const runs = projectSummary.runs.filter((run) => run.tool === "typechecker");
  if (runs.length !== 1 || !["ok", "findings"].includes(runs[0].status)) {
    throw new Error(`Fixture matrix summary has no successful typechecker run for ${project.id}`);
  }
  const expectedName = `${project.id}-typechecker.json`;
  const reportedPath = runs[0].outputPath;
  const artifactPath =
    typeof reportedPath === "string" && !isAbsolute(reportedPath)
      ? resolve(repoRoot, reportedPath)
      : null;
  if (
    artifactPath !== resolve(reportDir, expectedName) ||
    basename(reportedPath ?? "") !== expectedName
  ) {
    throw new Error(`Fixture matrix typechecker output path is invalid for ${project.id}`);
  }
  const rawPayload = readFileSync(artifactPath, "utf8");
  const payload = JSON.parse(rawPayload);
  const expectedKeys = [
    "exitCode",
    "parsed",
    "project",
    "schema",
    "stderr",
    "stdout",
    "tool",
    "version",
  ];
  if (JSON.stringify(Object.keys(payload).sort()) !== JSON.stringify(expectedKeys)) {
    throw new Error(`Fixture matrix typechecker artifact keys are invalid for ${project.id}`);
  }
  if (
    payload.schema !== "vize.fixtureToolRun" ||
    payload.version !== 1 ||
    payload.project !== project.id ||
    payload.tool !== "typechecker" ||
    payload.exitCode !== runs[0].exitCode
  ) {
    throw new Error(`Fixture matrix typechecker artifact identity is invalid for ${project.id}`);
  }
  let stdout;
  try {
    stdout = JSON.parse(payload.stdout);
  } catch {
    throw new Error(`Fixture matrix typechecker stdout is not JSON for ${project.id}`);
  }
  if (canonicalJson(stdout) !== canonicalJson(payload.parsed)) {
    throw new Error(
      `Fixture matrix typechecker stdout does not match parsed output for ${project.id}`,
    );
  }
  validateTypecheckerOutput(project, payload.parsed, payload.exitCode);
  const expectedStatus = payload.exitCode === 0 ? "ok" : "findings";
  if (runs[0].status !== expectedStatus) {
    throw new Error(`Fixture matrix typechecker status is inconsistent for ${project.id}`);
  }
  if (runs[0].fileCount !== payload.parsed.fileCount) {
    throw new Error(`Fixture matrix typechecker file count is inconsistent for ${project.id}`);
  }
  return {
    payload,
    source: {
      payloadSha256: sha256(rawPayload),
      fileCount: payload.parsed.fileCount,
    },
  };
}

function validatePerformanceConfig(performance) {
  if (!Number.isSafeInteger(performance.hangTimeoutMs) || performance.hangTimeoutMs <= 0) {
    throw new Error("typecheckPerformance.hangTimeoutMs must be a positive safe integer");
  }
  ratio(performance.maxFalsePositiveRatio, "maxFalsePositiveRatio");
  ratio(performance.maxFalseNegativeRatio, "maxFalseNegativeRatio");
}

function evaluateBudget(performance, summary) {
  const falseNegativeLimit = performance.maxFalseNegativeRatio;
  const falsePositivePassed = summary.falsePositiveRatio <= performance.maxFalsePositiveRatio;
  const falseNegativePassed = summary.falseNegativeRatio <= falseNegativeLimit;
  return {
    maxFalsePositiveRatio: performance.maxFalsePositiveRatio,
    maxFalseNegativeRatio: falseNegativeLimit,
    falsePositivePassed,
    falseNegativePassed,
    passed: falsePositivePassed && falseNegativePassed,
  };
}

function renderMarkdown(artifact) {
  const summary = artifact.divergence.summary;
  return [
    `## ${artifact.project} typecheck divergence`,
    "",
    `Commit: ${artifact.evidence.commitSha}`,
    `Vize diagnostics: ${summary.vizeDiagnosticCount}`,
    `vue-tsc diagnostics: ${summary.baselineDiagnosticCount}`,
    `Shared: ${summary.sharedCount}`,
    `Documented differences: ${summary.documentedDifferenceCount}`,
    `False positives: ${summary.falsePositiveCount} (${summary.falsePositiveRatio})`,
    `False negatives: ${summary.falseNegativeCount} (${summary.falseNegativeRatio})`,
    `Vize excluded non-Vue: ${summary.vizeExcludedNonVueCount}`,
    `vue-tsc excluded non-Vue: ${summary.baselineExcludedNonVueCount}`,
    `vue-tsc excluded project-level: ${summary.baselineExcludedProjectCount}`,
    `vue-tsc excluded external: ${summary.baselineExcludedExternalCount}`,
    `Budget passed: ${artifact.budget.passed ?? "not configured"}`,
    `Digest: ${artifact.divergence.sha256}`,
    "",
  ].join("\n");
}

function resolveVueTsc(value) {
  const candidate = isAbsolute(value) ? value : resolve(repoRoot, value);
  const probe = spawnSync(candidate, ["--version"], { encoding: "utf8", timeout: 10_000 });
  const version = (probe.stdout ?? "").trim();
  if (probe.error != null || probe.status !== 0 || version === "")
    throw new Error(`vue-tsc is not runnable: ${value}`);
  return { path: candidate, version };
}

function parseArgs(argv) {
  const args = {
    registry: defaultRegistry,
    reportDir: null,
    shardCount: 1,
    shardIndex: 0,
    vueTscBin: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const value = () => {
      if (argv[index + 1] == null) throw new Error(`${arg} requires a value`);
      return argv[++index];
    };
    if (arg === "--registry") args.registry = resolve(repoRoot, value());
    else if (arg === "--report-dir") args.reportDir = resolve(repoRoot, value());
    else if (arg === "--shard-count") args.shardCount = integer(value(), arg, 1);
    else if (arg === "--shard-index") args.shardIndex = integer(value(), arg, 0);
    else if (arg === "--vue-tsc-bin") args.vueTscBin = value();
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (args.reportDir == null) throw new Error("--report-dir is required");
  if (args.vueTscBin == null) throw new Error("--vue-tsc-bin is required");
  if (args.shardIndex >= args.shardCount) {
    throw new Error("--shard-index must be less than --shard-count");
  }
  return args;
}

function integer(value, name, minimum) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < minimum) {
    throw new Error(`${name} must be an integer >= ${minimum}`);
  }
  return parsed;
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value != null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
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

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === resolve(process.argv[1])
  : false;
if (entrypoint) {
  try {
    runTypecheckDivergenceReport();
  } catch (error) {
    process.stderr.write(`${errorMessage(error)}\n`);
    process.exit(1);
  }
}

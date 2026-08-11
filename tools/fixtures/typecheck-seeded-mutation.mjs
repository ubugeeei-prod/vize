import { createHash } from "node:crypto";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { evaluateVueProgramCoverage } from "./typecheck-baseline-coverage.mjs";
import { compareTypecheckDiagnostics } from "./typecheck-divergence.mjs";
import { runMeasured } from "./typecheck-process-run.mjs";

const probeFile = ".vize-typecheck-parity-seed.vue";
const configFile = ".vize-typecheck-parity-seed.tsconfig.json";
const cleanSource = [
  '<script lang="ts">',
  'export const vizeParitySeed: string = "clean";',
  "export default {};",
  "</script>",
  "<template><div /></template>",
  "",
].join("\n");
const brokenSource = cleanSource.replace('"clean"', "42");

export function runSeededTypecheckMutation({ fixtureRoot, project, vizeBin, vueTscBin }) {
  const probePath = join(fixtureRoot, probeFile);
  const configPath = join(fixtureRoot, configFile);
  if (existsSync(probePath) || existsSync(configPath)) {
    throw new Error(`Seeded typecheck probe path already exists for ${project.id}`);
  }
  const sourceProject = project.typecheckPerformance.baseline?.tsconfig ?? project.tsconfig;
  const config = {
    extends: configPathValue(sourceProject),
    compilerOptions: { composite: false, incremental: false },
    files: [`./${probeFile}`],
    include: [],
    references: [],
  };
  writeFileSync(configPath, `${JSON.stringify(config, null, 2)}\n`, { flag: "wx" });
  try {
    const states = [
      runState("clean", cleanSource),
      runState("broken", brokenSource),
      runState("repaired", cleanSource),
    ];
    validateStates(states, project.id);
    return {
      tier: "sfc-script-ts2322",
      configSha256: sha256(readFileSync(configPath)),
      probeFile,
      states,
    };
  } finally {
    rmSync(probePath, { force: true });
    rmSync(configPath, { force: true });
  }

  function runState(state, source) {
    writeFileSync(probePath, source);
    const vizeArgs = [
      "check",
      probeFile,
      "--format",
      "json",
      "--no-config",
      "--tsconfig",
      configFile,
      "--servers",
      "1",
    ];
    const baselineArgs = ["--noEmit", "--pretty", "false", "--listFiles", "-p", configPath];
    const timeout = project.typecheckPerformance.hangTimeoutMs;
    const vize = execute(vizeBin, vizeArgs, fixtureRoot, timeout);
    const baseline = execute(vueTscBin, baselineArgs, fixtureRoot, timeout);
    if (![0, 1].includes(vize.exitCode)) {
      throw new Error(`Seeded Vize ${state} run failed for ${project.id}: ${vize.exitCode}`);
    }
    // The inherited fixture config may emit an unrelated project/non-Vue
    // diagnostic (for example TS6's deprecation diagnostic) even though this
    // one-file probe is clean. The full baseline configuration gate owns those
    // diagnostics; this oracle owns the probe SFC, whose exact coverage and
    // normalized diagnostics are checked below.
    if (baseline.exitCode !== 0 && baseline.exitCode !== 2) {
      throw new Error(`Seeded vue-tsc ${state} run failed for ${project.id}: ${baseline.exitCode}`);
    }
    let report;
    try {
      report = JSON.parse(vize.stdout);
    } catch {
      throw new Error(`Seeded Vize ${state} output is not JSON for ${project.id}`);
    }
    const coverage = evaluateVueProgramCoverage(report, baseline.stdout, fixtureRoot);
    if (coverage.verdict !== "usable" || coverage.sharedVueFileCount !== 1) {
      throw new Error(`Seeded ${state} file coverage is unusable for ${project.id}`);
    }
    const divergence = compareTypecheckDiagnostics({
      projectId: `${project.id}-seed`,
      cwd: fixtureRoot,
      vizeReport: report,
      vueTscOutput: `${baseline.stdout}\n${baseline.stderr}`,
      documentedDifferences: [],
      includedFiles: [probeFile],
    });
    return {
      state,
      sourceSha256: sha256(source),
      vize: evidence(vize),
      baseline: evidence(baseline),
      coverage,
      divergence,
    };
  }
}

function validateStates(states, projectId) {
  const [clean, broken, repaired] = states;
  for (const state of [clean, repaired]) {
    if (
      state.divergence.summary.vizeDiagnosticCount !== 0 ||
      state.divergence.summary.baselineDiagnosticCount !== 0
    ) {
      throw new Error(`Seeded ${state.state} state is not clean for ${projectId}`);
    }
  }
  const summary = broken.divergence.summary;
  if (
    summary.vizeDiagnosticCount !== 1 ||
    summary.baselineDiagnosticCount !== 1 ||
    summary.sharedCount !== 1 ||
    summary.messageMismatchCount !== 0 ||
    summary.falsePositiveCount !== 0 ||
    summary.falseNegativeCount !== 0 ||
    broken.divergence.shared[0]?.code !== 2322 ||
    broken.divergence.shared[0]?.file !== probeFile
  ) {
    throw new Error(`Seeded broken diagnostic did not match exactly for ${projectId}`);
  }
  if (
    clean.sourceSha256 !== repaired.sourceSha256 ||
    clean.sourceSha256 === broken.sourceSha256 ||
    clean.coverage.vizeVueFilesSha256 !== broken.coverage.vizeVueFilesSha256 ||
    clean.coverage.vizeVueFilesSha256 !== repaired.coverage.vizeVueFilesSha256
  ) {
    throw new Error(`Seeded clean/broken/repaired identity is inconsistent for ${projectId}`);
  }
}

function execute(command, args, cwd, timeout) {
  const result = runMeasured(command, args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout,
  });
  if (result.error != null) throw result.error;
  return {
    command: [command, ...args],
    durationMs: result.durationMs,
    peakRssBytes: result.peakRssBytes,
    exitCode: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function evidence(result) {
  return {
    command: result.command,
    durationMs: result.durationMs,
    peakRssBytes: result.peakRssBytes,
    exitCode: result.exitCode,
    stdoutSha256: sha256(result.stdout),
    stderrSha256: sha256(result.stderr),
  };
}

function configPathValue(value) {
  return value.startsWith(".") ? value : `./${value}`;
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

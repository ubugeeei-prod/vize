import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { retainFixtureCommand } from "../support/typecheck-fixture-custody.ts";

import {
  summarizeTypecheckerCoverage,
  validateTypecheckerOutput,
} from "../../../tools/support/compat/fixtures/tool-matrix-typechecker.mjs";
import {
  collectTypecheckerAuthoredPaths,
  collectVueInputPaths,
} from "../../../tools/support/compat/fixtures/tool-matrix-inputs.mjs";
import {
  type MutationDiagnosticMode,
  writeVize,
  writeVueTsc,
  writeVueTscFixture,
} from "./typecheck-divergence-checker-fakes.ts";

export { writeVueTsc };

/**
 * Scaffolding shared by every `tools/commands/fixtures/typecheck-divergence-report.rs`
 * test. It lives here rather than in one test file so a second suite can drive
 * the same report without duplicating the matrix artifacts it validates.
 */
export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const script = path.join(
  root,
  "tools",
  "commands",
  "fixtures",
  "typecheck-divergence-report.rs",
);
export const commitSha = "a".repeat(40);
export const pnpmInstallCommand = [
  "pnpm",
  "install",
  "--frozen-lockfile",
  "--ignore-scripts",
  "--prefer-offline",
  "--ignore-workspace",
];

/** The one diagnostic both sides report, so the default fixture diverges nowhere. */
export const sharedVizeDiagnostic = "error:1:1 [TS2322] shared";
export const sharedBaselineOutput = "src/App.vue(1,1): error TS2322: shared\n";

/**
 * The classification #3738 added to every failing verdict. A reader of run
 * 30738583070 could not tell a Vize finding from a broken baseline, so both
 * failure messages now name which one they are before quoting the numbers.
 */
export const instrumentClassification =
  "instrument failure, the vue-tsc baseline did not measure Vize";

export function divergenceClassification(sharedVueFileCount: number) {
  return (
    "Vize divergence, the vue-tsc baseline loaded cleanly over the same " +
    `${sharedVueFileCount} Vue files, against the fixture's own Vue runtime`
  );
}

export function unusableFailure(reason: string) {
  return `Typecheck divergence baseline is unusable for fixture — ${instrumentClassification}: ${reason}`;
}

export function breachFailure(sharedVueFileCount: number, breaches: string) {
  return (
    "Typecheck divergence budget breached for fixture — " +
    `${divergenceClassification(sharedVueFileCount)}: ${breaches}`
  );
}

export type FixtureOptions = {
  /** Diagnostics the fake `vize check` artifact reports for `src/App.vue`. */
  vizeDiagnostics?: string[];
  /** Raw stdout the fake `vue-tsc` writes before exiting with a diagnostic status. */
  baselineOutput?: string;
  /** Stream the fake `vue-tsc` writes diagnostics to. */
  baselineOutputStream?: "stdout" | "stderr";
  /** Exit code for the fake `vue-tsc` baseline run. */
  baselineExitCode?: number;
  /** Stream the fake `vue-tsc --listFilesOnly` writes program-file evidence to. */
  coverageOutputStream?: "stdout" | "stderr";
  /** Exit code for the fake `vue-tsc --listFilesOnly` coverage run. */
  coverageExitCode?: number;
  /** Vue source files emitted by the fake `vue-tsc --listFilesOnly` run. */
  baselineFiles?: string[];
  /** Absolute program paths outside the fixture the same run also loaded. */
  baselineProgramFiles?: string[];
  /** How the fake Vize binary reports the seeded mutation during oracle runs. */
  vizeMutation?: MutationDiagnosticMode;
  /** How the fake vue-tsc binary reports the seeded mutation during oracle runs. */
  baselineMutation?: MutationDiagnosticMode;
};

export function setup(options: FixtureOptions = {}) {
  const vizeDiagnostics = options.vizeDiagnostics ?? [sharedVizeDiagnostic];
  const baselineOutput = options.baselineOutput ?? sharedBaselineOutput;
  const vizeMutation = options.vizeMutation ?? "match";
  const baselineMutation = options.baselineMutation ?? "match";
  const fixtureRoot = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "typecheck-divergence-"),
  );
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-divergence-report-"));
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-divergence-vue-tsc-"));
  const fixturePath = path.relative(root, fixtureRoot);
  const project = {
    id: "fixture",
    fixturePath,
    revision: "b".repeat(40),
    vueGlobs: ["src/**/*.vue"],
    tsconfig: "tsconfig.json",
    typecheckPerformance: {
      enabled: true,
      compareTo: "vue-tsc",
      packageManager: "pnpm",
      packageManagerVersion: "10.0.0",
      lockfile: "pnpm-lock.yaml",
      hangTimeoutMs: 5_000,
      maxFalsePositiveRatio: 0.05,
      maxFalseNegativeRatio: 0.05,
    },
  };
  fs.mkdirSync(path.join(fixtureRoot, "src"));
  fs.writeFileSync(path.join(fixtureRoot, "tsconfig.json"), "{}\n");
  fs.writeFileSync(path.join(fixtureRoot, "pnpm-lock.yaml"), "lockfileVersion: '9.0'\n");
  fs.writeFileSync(path.join(fixtureRoot, "src", "App.vue"), "<template />\n");
  const registryPath = path.join(fixtureRoot, "registry.json");
  writeJson(registryPath, { projects: [project] });
  const outputPath = path.join(reportDir, "fixture-typechecker.json");
  const parsed = {
    errorCount: vizeDiagnostics.length,
    warningCount: 0,
    fileCount: 1,
    files: [{ file: "src/App.vue", diagnostics: vizeDiagnostics }],
  };
  // `vize check` exits 0 only when it found nothing, and the matrix summary
  // status is derived from that, so a fixture with no diagnostics has to agree.
  const exitCode = vizeDiagnostics.length === 0 ? 0 : 1;
  const typecheckerCoverage = validateTypecheckerOutput(
    project,
    parsed,
    exitCode,
    ["src/App.vue"],
    ["src/App.vue"],
  );
  writeJson(outputPath, {
    schema: "vize.fixtureToolRun",
    version: 1,
    project: "fixture",
    tool: "typechecker",
    exitCode,
    stdout: JSON.stringify(parsed),
    stderr: "",
    parsed,
    typecheckerCoverage,
  });
  writeJson(path.join(reportDir, "summary.json"), {
    schema: "vize.fixtureToolMatrixReport",
    version: 3,
    evidence: {
      commitSha,
      runtime: { name: "node", version: process.versions.node },
      machine: {
        platform: process.platform,
        arch: process.arch,
        cpuModel: "synthetic",
        logicalCpuCount: 1,
        totalMemoryBytes: 1,
      },
    },
    projects: [
      {
        id: "fixture",
        revision: project.revision,
        runs: [
          {
            tool: "typechecker",
            status: exitCode === 0 ? "ok" : "findings",
            exitCode,
            fileCount: 1,
            outputPath: path.relative(root, outputPath),
            coverage: summarizeTypecheckerCoverage(typecheckerCoverage),
          },
        ],
      },
    ],
  });
  writeJson(path.join(reportDir, "fixture-typecheck-dependencies.json"), {
    schema: "vize.fixtureTypecheckDependencyInstall",
    version: 2,
    project: "fixture",
    revision: project.revision,
    evidence: {
      commitSha,
      runtime: { name: "node", version: process.versions.node },
    },
    packageManager: {
      name: "pnpm",
      version: "10.0.0",
    },
    lockfile: {
      path: "pnpm-lock.yaml",
      sizeBytes: fs.readFileSync(path.join(fixtureRoot, "pnpm-lock.yaml")).byteLength,
      sha256: sha256(fs.readFileSync(path.join(fixtureRoot, "pnpm-lock.yaml"))),
    },
    install: {
      command: pnpmInstallCommand,
      durationMs: 1,
      exitCode: 0,
      stdoutSha256: sha256("installed"),
      stderrSha256: sha256(""),
    },
    baselinePrepare: null,
  });
  const vize = path.join(fakeDir, "vize.mjs");
  const vueTsc = path.join(fakeDir, "vue-tsc.mjs");
  const invocationPath = path.join(fakeDir, "invocation.json");
  writeVize(vize, {
    baselineDiagnostics: vizeDiagnostics,
    mutation: vizeMutation,
  });
  writeVueTscFixture(
    vueTsc,
    {
      baselineOutput,
      baselineOutputStream: options.baselineOutputStream,
      coverageExitCode: options.coverageExitCode,
      coverageOutputStream: options.coverageOutputStream,
      exitCode: options.baselineExitCode,
      files: options.baselineFiles ?? ["src/App.vue"],
      fixtureRoot,
      mutation: baselineMutation,
      programFiles: options.baselineProgramFiles,
    },
    invocationPath,
  );
  return {
    fixtureRoot,
    reportDir,
    fakeDir,
    registryPath,
    outputPath,
    vize,
    vueTsc,
    invocationPath,
  };
}

export function run(
  fixture: ReturnType<typeof setup>,
  env: NodeJS.ProcessEnv = {},
  extraArgs: string[] = [],
  options: { timeoutMs?: number; retainEvidence?: boolean } = {},
) {
  const args = [
    script,
    "--registry",
    fixture.registryPath,
    "--report-dir",
    fixture.reportDir,
    "--vize-bin",
    fixture.vize,
    "--vue-tsc-bin",
    fixture.vueTsc,
    ...extraArgs,
  ];
  const commandOptions = {
    cwd: root,
    env: { ...process.env, GITHUB_SHA: commitSha, ...env },
    timeout: options.timeoutMs,
  };
  if (!options.retainEvidence)
    return spawnSync("rust-script", args, { ...commandOptions, encoding: "utf8" });
  const result = spawnSync("rust-script", args, { ...commandOptions, encoding: "buffer" });
  const evidence = retainFixtureCommand(
    path.join(root, "target/differential/typecheck-mutation"),
    { executable: "rust-script", args, cwd: root, timeoutMs: options.timeoutMs },
    result,
    { fixture: fixture.fixtureRoot, report: fixture.reportDir, checkers: fixture.fakeDir },
  );
  console.error(`Mutation deadline evidence: ${evidence}`);
  return {
    ...result,
    stdout: result.stdout?.toString("utf8") ?? "",
    stderr: result.stderr?.toString("utf8") ?? "",
  };
}

export function cleanup(fixture: ReturnType<typeof setup>) {
  fs.rmSync(fixture.fixtureRoot, { recursive: true, force: true });
  fs.rmSync(fixture.reportDir, { recursive: true, force: true });
  fs.rmSync(fixture.fakeDir, { recursive: true, force: true });
}

export function readJson(pathname: string) {
  return JSON.parse(fs.readFileSync(pathname, "utf8"));
}

/** Record-only preserves the whole observation even when its budget is unusable or breached. */
export function assertRecordOnlyObservations(
  fixture: ReturnType<typeof setup>,
  artifact: any,
  expected: { baselineOutput: string; baselineFiles: string[] },
) {
  const observations = readJson(
    path.join(fixture.reportDir, "fixture-typecheck-observations.json"),
  );
  const payloadBytes = fs.readFileSync(fixture.outputPath);
  const config = fs.readFileSync(
    path.join(fixture.reportDir, "fixture-vue-tsc.tsconfig.json"),
    "utf8",
  );
  const sourceConfig = fs.readFileSync(path.join(fixture.fixtureRoot, "tsconfig.json"), "utf8");
  const authority = {
    config,
    configSha256: sha256(config),
    sourceConfig,
    sourceConfigSha256: sha256(sourceConfig),
  };
  assert.deepEqual(artifact.enforcement, { budgetMode: "record-only" });
  assert.deepEqual(observations, {
    schema: "vize.fixtureTypecheckObservations",
    version: 1,
    project: "fixture",
    evidence: readJson(path.join(fixture.reportDir, "summary.json")).evidence,
    preparation: artifact.preparation,
    source: {
      payloadSha256: sha256(payloadBytes),
      fileCount: readJson(fixture.outputPath).parsed.fileCount,
    },
    vize: readJson(fixture.outputPath),
    baseline: {
      command: artifact.baseline.command,
      version: "3.3.4",
      exitCode: expected.baselineOutput === "" ? 0 : 2,
      runError: null,
      stdout: expected.baselineOutput,
      stderr: "",
      config,
      sourceConfig,
    },
    ...(expected.baselineOutput === ""
      ? {}
      : {
          preSemantic: {
            command: `${artifact.baseline.command} --noCheck`,
            providerVersion: "3.3.4",
            configSha256: sha256(config),
            exitCode: 0,
            processStatus: "exit status: 0",
            runError: null,
            stdout: "",
            stderr: "",
          },
          preSemanticAuthorityBefore: authority,
          preSemanticAuthorityAfter: authority,
        }),
    coverage: {
      command: artifact.baseline.coverageCommand,
      exitCode: 0,
      runError: null,
      stdout: expected.baselineFiles
        .map((file) => `${path.join(fixture.fixtureRoot, file)}\n`)
        .join(""),
      stderr: "",
    },
  });
  assert.deepEqual(artifact.evidence, observations.evidence);
  assert.deepEqual(artifact.source, observations.source);
  assert.equal(
    artifact.preparation.payloadSha256,
    sha256(fs.readFileSync(path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"))),
  );
  for (const [raw, hash] of [
    [observations.baseline.stdout, artifact.baseline.stdoutSha256],
    [observations.baseline.stderr, artifact.baseline.stderrSha256],
    [observations.coverage.stdout, artifact.baseline.coverageStdoutSha256],
    [observations.coverage.stderr, artifact.baseline.coverageStderrSha256],
    [observations.baseline.config, artifact.baseline.configSha256],
    [observations.baseline.sourceConfig, artifact.baseline.sourceConfigSha256],
  ])
    assert.equal(sha256(raw), hash);
}

export function updateVizeOutput(
  fixture: ReturnType<typeof setup>,
  mutate: (parsed: Record<string, any>) => void,
) {
  const payload = readJson(fixture.outputPath);
  mutate(payload.parsed);
  payload.stdout = JSON.stringify(payload.parsed);
  const project = readJson(fixture.registryPath).projects[0];
  payload.typecheckerCoverage = validateTypecheckerOutput(
    project,
    payload.parsed,
    payload.exitCode,
    collectVueInputPaths(fixture.fixtureRoot, project.vueGlobs),
    collectTypecheckerAuthoredPaths(fixture.fixtureRoot),
  );
  writeJson(fixture.outputPath, payload);

  const summaryPath = path.join(fixture.reportDir, "summary.json");
  const summary = readJson(summaryPath);
  summary.projects[0].runs[0].fileCount = payload.parsed.fileCount;
  summary.projects[0].runs[0].coverage = summarizeTypecheckerCoverage(payload.typecheckerCoverage);
  writeJson(summaryPath, summary);
}

export function writeJson(pathname: string, value: unknown) {
  fs.writeFileSync(pathname, `${JSON.stringify(value, null, 2)}\n`);
}

export function updateJson(pathname: string, update: (value: any) => void) {
  const value = readJson(pathname);
  update(value);
  writeJson(pathname, value);
}

export function sha256(value: string | Buffer) {
  return createHash("sha256").update(value).digest("hex");
}

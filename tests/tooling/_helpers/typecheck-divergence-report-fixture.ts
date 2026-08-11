import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  summarizeTypecheckerCoverage,
  validateTypecheckerOutput,
} from "../../../tools/fixtures/tool-matrix-typechecker.mjs";
import {
  collectTypecheckerAuthoredPaths,
  collectVueInputPaths,
} from "../../../tools/fixtures/tool-matrix-inputs.mjs";

/**
 * Scaffolding shared by every `tools/fixtures/typecheck-divergence-report.mjs`
 * test. It lives here rather than in one test file so a second suite can drive
 * the same report without duplicating the matrix artifacts it validates.
 */
export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const script = path.join(root, "tools", "fixtures", "typecheck-divergence-report.mjs");
export const commitSha = "a".repeat(40);

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
    `${sharedVueFileCount} Vue files`
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
  /** Raw stdout the fake `vue-tsc` writes before exiting with status 2. */
  baselineOutput?: string;
  /** Vue source files emitted by the fake `vue-tsc --listFiles` run. */
  baselineFiles?: string[];
};

export function setup(options: FixtureOptions = {}) {
  const vizeDiagnostics = options.vizeDiagnostics ?? [sharedVizeDiagnostic];
  const baselineOutput = options.baselineOutput ?? sharedBaselineOutput;
  const fixtureRoot = fs.mkdtempSync(
    path.join(root, "tests", "_fixtures", "typecheck-divergence-"),
  );
  const reportDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-divergence-report-"));
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-divergence-vue-tsc-"));
  const vueTsc = path.join(fakeDir, "vue-tsc.mjs");
  const vize = path.join(fakeDir, "vize.mjs");
  const invocationPath = path.join(fakeDir, "invocation.json");
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
      maxFalsePositiveRatio: 0,
      maxFalseNegativeRatio: 0,
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
            command: `${vize} check src/**/*.vue --format json --no-config --tsconfig tsconfig.json`,
            cwd: fixturePath,
            durationMs: 1,
            peakRssBytes: 1,
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
  const lockfile = fs.readFileSync(path.join(fixtureRoot, "pnpm-lock.yaml"));
  const baselineConfig = fs.readFileSync(path.join(fixtureRoot, "tsconfig.json"));
  writeJson(path.join(reportDir, "fixture-typecheck-dependencies.json"), {
    schema: "vize.fixtureTypecheckDependencyInstall",
    version: 3,
    project: "fixture",
    revision: project.revision,
    evidence: {
      commitSha,
      runtime: { name: "node", version: process.versions.node },
    },
    packageManager: { name: "pnpm", version: "10.0.0" },
    lockfile: {
      path: "pnpm-lock.yaml",
      sizeBytes: lockfile.byteLength,
      sha256: sha256(lockfile),
    },
    install: {
      command: ["pnpm", "install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
      durationMs: 1,
      exitCode: 0,
      stdoutSha256: sha256(""),
      stderrSha256: sha256(""),
    },
    baselinePrepare: null,
    baselineConfig: {
      path: "tsconfig.json",
      sizeBytes: baselineConfig.byteLength,
      sha256: sha256(baselineConfig),
    },
  });
  writeVueTsc(
    vueTsc,
    `process.stdout.write(${JSON.stringify(
      `${baselineOutput}${(options.baselineFiles ?? ["src/App.vue"])
        .map((file) => `${path.join(fixtureRoot, file)}\n`)
        .join("")}`,
    )}); process.exit(2);`,
    invocationPath,
  );
  writeFakeVize(vize);
  return {
    fixtureRoot,
    reportDir,
    fakeDir,
    registryPath,
    outputPath,
    vueTsc,
    vize,
    invocationPath,
  };
}

export function writeVueTsc(pathname: string, runBody: string, invocationPath?: string) {
  const recordInvocation =
    invocationPath == null
      ? ""
      : `fs.writeFileSync(${JSON.stringify(invocationPath)}, JSON.stringify({ cwd: process.cwd(), args: process.argv.slice(2) }));`;
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node\nimport fs from "node:fs";\nimport path from "node:path";\nif (process.argv.includes("--version")) { console.log("3.3.4"); process.exit(0); }\n${seededVueTscBody()}\n${recordInvocation}\n${runBody}\n`,
  );
  fs.chmodSync(pathname, 0o755);
}

export function run(
  fixture: ReturnType<typeof setup>,
  env: NodeJS.ProcessEnv = {},
  extraArgs: string[] = [],
) {
  return spawnSync(
    process.execPath,
    [
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
    ],
    { cwd: root, encoding: "utf8", env: { ...process.env, GITHUB_SHA: commitSha, ...env } },
  );
}

export function cleanup(fixture: ReturnType<typeof setup>) {
  fs.rmSync(fixture.fixtureRoot, { recursive: true, force: true });
  fs.rmSync(fixture.reportDir, { recursive: true, force: true });
  fs.rmSync(fixture.fakeDir, { recursive: true, force: true });
}

export function readJson(pathname: string) {
  return JSON.parse(fs.readFileSync(pathname, "utf8"));
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

function sha256(value: string | Buffer) {
  return createHash("sha256").update(value).digest("hex");
}

function writeFakeVize(pathname: string) {
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node\nimport fs from "node:fs";\nif (process.argv.includes("--version")) { console.log("vize 0.0.0"); process.exit(0); }\nconst probe = ".vize-typecheck-parity-seed.vue";\nconst source = fs.readFileSync(probe, "utf8");\nconst broken = source.includes("= 42;");\nconst report = { errorCount: broken ? 1 : 0, warningCount: 0, fileCount: 1, files: [{ file: probe, diagnostics: broken ? ["error:2:7 [TS2322] Type 'number' is not assignable to type 'string'."] : [] }] };\nprocess.stdout.write(JSON.stringify(report));\nprocess.exit(broken ? 1 : 0);\n`,
  );
  fs.chmodSync(pathname, 0o755);
}

function seededVueTscBody() {
  return `const config = process.argv.at(-1);\nif (config?.endsWith(".vize-typecheck-parity-seed.tsconfig.json")) {\n  const probe = path.join(process.cwd(), ".vize-typecheck-parity-seed.vue");\n  const broken = fs.readFileSync(probe, "utf8").includes("= 42;");\n  if (broken) process.stdout.write(probe + "(2,7): error TS2322: Type 'number' is not assignable to type 'string'.\\n");\n  process.stdout.write(probe + "\\n");\n  process.exit(broken ? 2 : 0);\n}`;
}

export function updateJson(pathname: string, update: (value: any) => void) {
  const value = readJson(pathname);
  update(value);
  writeJson(pathname, value);
}

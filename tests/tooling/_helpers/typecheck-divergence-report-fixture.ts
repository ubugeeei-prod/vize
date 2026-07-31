import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

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

export type FixtureOptions = {
  /** Diagnostics the fake `vize check` artifact reports for `src/App.vue`. */
  vizeDiagnostics?: string[];
  /** Raw stdout the fake `vue-tsc` writes before exiting with status 2. */
  baselineOutput?: string;
};

export function setup(options: FixtureOptions = {}) {
  const vizeDiagnostics = options.vizeDiagnostics ?? [sharedVizeDiagnostic];
  const baselineOutput = options.baselineOutput ?? sharedBaselineOutput;
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
  writeJson(outputPath, {
    schema: "vize.fixtureToolRun",
    version: 1,
    project: "fixture",
    tool: "typechecker",
    exitCode: 1,
    stdout: JSON.stringify(parsed),
    stderr: "",
    parsed,
  });
  writeJson(path.join(reportDir, "summary.json"), {
    schema: "vize.fixtureToolMatrixReport",
    version: 2,
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
            status: "findings",
            exitCode: 1,
            fileCount: 1,
            outputPath: path.relative(root, outputPath),
          },
        ],
      },
    ],
  });
  const vueTsc = path.join(fakeDir, "vue-tsc.mjs");
  const invocationPath = path.join(fakeDir, "invocation.json");
  writeVueTsc(
    vueTsc,
    `process.stdout.write(${JSON.stringify(baselineOutput)}); process.exit(2);`,
    invocationPath,
  );
  return { fixtureRoot, reportDir, fakeDir, registryPath, outputPath, vueTsc, invocationPath };
}

export function writeVueTsc(pathname: string, runBody: string, invocationPath?: string) {
  const recordInvocation =
    invocationPath == null
      ? ""
      : `fs.writeFileSync(${JSON.stringify(invocationPath)}, JSON.stringify({ cwd: process.cwd(), args: process.argv.slice(2) }));`;
  fs.writeFileSync(
    pathname,
    `#!/usr/bin/env node\nimport fs from "node:fs";\nif (process.argv.includes("--version")) { console.log("3.3.4"); process.exit(0); }\n${recordInvocation}\n${runBody}\n`,
  );
  fs.chmodSync(pathname, 0o755);
}

export function run(fixture: ReturnType<typeof setup>, env: NodeJS.ProcessEnv = {}) {
  return spawnSync(
    process.execPath,
    [
      script,
      "--registry",
      fixture.registryPath,
      "--report-dir",
      fixture.reportDir,
      "--vue-tsc-bin",
      fixture.vueTsc,
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

export function writeJson(pathname: string, value: unknown) {
  fs.writeFileSync(pathname, `${JSON.stringify(value, null, 2)}\n`);
}

export function updateJson(pathname: string, update: (value: any) => void) {
  const value = readJson(pathname);
  update(value);
  writeJson(pathname, value);
}

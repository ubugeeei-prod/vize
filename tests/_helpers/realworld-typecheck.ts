import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { repoRoot, symlinkDirectory } from "./realworld-patch.ts";

export type CheckReport = {
  errorCount: number;
  fileCount: number;
  files: Array<{ diagnostics: string[]; file: string }>;
  programs: Array<{ files: string[]; root: string; tsconfig?: string }>;
  warningCount: number;
};

export type CommandResult = {
  status: number | null;
  stderr: string;
  stdout: string;
};

export type VizeCheckResult = CommandResult & { report: CheckReport };

export function omitProgramEvidence(report: CheckReport): Omit<CheckReport, "programs"> {
  const { programs: _programs, ...diagnosticReport } = report;
  return diagnosticReport;
}

export function runVizeCheck(
  workspaceDir: string,
  corsaPath: string,
  patterns: string[],
  tsconfigPath: string | null = "tsconfig.json",
): VizeCheckResult {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const tsconfigArgs = tsconfigPath === null ? [] : ["--tsconfig", tsconfigPath];
  const result = runCommand(
    command,
    [
      ...prefixArgs,
      "check",
      ...patterns,
      ...tsconfigArgs,
      "--format",
      "json",
      "--quiet",
      "--corsa-path",
      corsaPath,
    ],
    workspaceDir,
  );
  return { ...result, report: JSON.parse(result.stdout) as CheckReport };
}

export function runVueTsc(workspaceDir: string, vueTscPath: string): CommandResult {
  return runCommand(
    vueTscPath,
    ["--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
    workspaceDir,
  );
}

export function runVueTscBuild(workspaceDir: string, vueTscPath: string): CommandResult {
  return runCommand(
    vueTscPath,
    ["--build", "tsconfig.json", "--pretty", "false", "--force"],
    workspaceDir,
  );
}

export function resolveTsgoBinary(): string {
  return requireBinary(
    [
      process.env.VIZE_TEST_TSGO,
      path.join(repoRoot, "../corsa-bind/.cache/tsgo"),
      path.join(repoRoot, "node_modules/.bin/tsgo"),
      path.join(repoRoot, "tests/node_modules/.bin/tsgo"),
    ],
    "tsgo",
  );
}

export function resolveVueTscBinary(): string {
  return requireBinary(
    [
      process.env.VIZE_TEST_VUE_TSC,
      path.join(repoRoot, "node_modules/.bin/vue-tsc"),
      path.join(repoRoot, "tests/node_modules/.bin/vue-tsc"),
    ],
    "vue-tsc",
  );
}

export function symlinkVueTypes(workspaceDir: string): void {
  const candidates = [
    path.join(repoRoot, "node_modules/vue"),
    path.join(repoRoot, "tests/node_modules/vue"),
  ];
  const vuePackage = candidates.find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for real-world patch oracles");
  symlinkDirectory(vuePackage, path.join(workspaceDir, "node_modules/vue"));
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    symlinkDirectory(vueNamespace, path.join(workspaceDir, "node_modules/@vue"));
  }
}

function runCommand(command: string, args: string[], cwd: string): CommandResult {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    maxBuffer: 64 * 1024 * 1024,
    timeout: 120_000,
  });
  if (result.error != null) throw result.error;
  return { status: result.status, stderr: result.stderr, stdout: result.stdout };
}

export function resolveVizeCommand(): string[] {
  const candidates = [
    process.env.VIZE_TEST_BIN,
    path.join(repoRoot, "target/debug/vize"),
    path.join(repoRoot, "target/ci/vize"),
    path.join(repoRoot, "target/release/vize"),
    "vize",
  ]
    .filter((candidate): candidate is string => Boolean(candidate))
    .map(normalizeCommandPath);
  for (const candidate of candidates) {
    if (spawnSync(candidate, ["--version"], { cwd: repoRoot }).status === 0) return [candidate];
  }
  return ["cargo", "run", "-q", "-p", "vize", "--"];
}

function normalizeCommandPath(command: string): string {
  if (path.isAbsolute(command) || (!command.includes("/") && !command.includes("\\"))) {
    return command;
  }
  return path.resolve(repoRoot, command);
}

function requireBinary(candidates: Array<string | undefined>, name: string): string {
  const binary = candidates.find((candidate): candidate is string =>
    Boolean(candidate && fs.existsSync(candidate)),
  );
  assert.ok(binary, `${name} binary is required for real-world patch oracles`);
  return binary;
}

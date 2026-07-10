import assert from "node:assert/strict";
import { execSync } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";

import type { AppConfig } from "../../_helpers/apps.ts";
import { CORSA_BIN, VIZE_BIN } from "../../_helpers/apps.ts";

export interface VizeCheckSummary {
  fileCount: number;
  errorCount: number;
  durationMs: number;
}

export interface VizeLintSummary {
  fileCount: number;
  durationMs: number;
}

export interface InjectedTypeErrorSummary extends VizeCheckSummary {
  file: string;
  diagnostics: string[];
}

interface InjectedTypeErrorOptions {
  timeoutMs?: number;
  tsconfig?: {
    relativePath: string;
    content: string;
  };
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function readCommandStdout(cmd: string, cwd: string, timeoutMs: number): string {
  try {
    return execSync(cmd, {
      cwd,
      timeout: timeoutMs,
      maxBuffer: 100 * 1024 * 1024,
    }).toString();
  } catch (error: unknown) {
    const commandError = error as {
      status?: number;
      stdout?: { toString(): string };
      stderr?: { toString(): string };
      signal?: string;
    };

    const stdout = commandError.stdout?.toString() ?? "";
    if (commandError.status === 1 && stdout.trim().length > 0) {
      return stdout;
    }

    const stderr = commandError.stderr?.toString() ?? "";
    const signal = commandError.signal ? `, signal ${commandError.signal}` : "";
    throw new Error(
      `vize command crashed (exit ${commandError.status ?? "unknown"}${signal})\n${stderr}`,
    );
  }
}

function buildVizeCheckCommand(
  checkConfig: NonNullable<AppConfig["check"]>,
  patterns: string[],
): string {
  const patternArgs = patterns.map(shellQuote).join(" ");
  const tsconfig = checkConfig.tsconfig ? ` --tsconfig ${shellQuote(checkConfig.tsconfig)}` : "";
  return [
    shellQuote(VIZE_BIN),
    "check",
    patternArgs,
    "--format json",
    "--quiet",
    "--servers 1",
    "--corsa-path",
    shellQuote(CORSA_BIN),
    tsconfig,
  ]
    .filter(Boolean)
    .join(" ");
}

function runVizeCheckJson(
  checkConfig: NonNullable<AppConfig["check"]>,
  patterns: string[],
  timeoutMs: number,
): {
  fileCount?: number;
  errorCount?: number;
  files?: Array<{ file?: string; diagnostics?: string[] }>;
} {
  const cmd = buildVizeCheckCommand(checkConfig, patterns);
  console.log(`Running: ${cmd}`);

  const stdout = readCommandStdout(cmd, checkConfig.cwd, timeoutMs);
  return JSON.parse(stdout) as {
    fileCount?: number;
    errorCount?: number;
    files?: Array<{ file?: string; diagnostics?: string[] }>;
  };
}

export function runCrashFreeVizeCheck(
  app: AppConfig,
  options: { timeoutMs?: number } = {},
): VizeCheckSummary {
  const checkConfig = app.check!;
  const timeoutMs = options.timeoutMs ?? 300_000;

  const startedAt = performance.now();
  const parsed = runVizeCheckJson(checkConfig, checkConfig.patterns, timeoutMs);
  const durationMs = performance.now() - startedAt;

  assert.ok((parsed.fileCount ?? 0) > 0, "check fileCount should be > 0");
  assert.ok(durationMs < timeoutMs, `check should finish before ${timeoutMs}ms`);

  return {
    fileCount: parsed.fileCount ?? 0,
    errorCount: parsed.errorCount ?? 0,
    durationMs,
  };
}

function firstExistingPatternBase(cwd: string, patterns: string[]): string {
  for (const pattern of patterns) {
    const normalized = pattern.replaceAll("\\", "/");
    const globIndex = normalized.search(/[*?[{]/);
    const prefix = globIndex === -1 ? path.dirname(normalized) : normalized.slice(0, globIndex);
    const slashIndex = prefix.lastIndexOf("/");
    const relativeDir = slashIndex === -1 ? "." : prefix.slice(0, slashIndex);
    const candidate = path.join(cwd, relativeDir.length > 0 ? relativeDir : ".");
    if (fs.existsSync(candidate) && fs.statSync(candidate).isDirectory()) {
      return relativeDir.length > 0 ? relativeDir : ".";
    }
  }
  return ".";
}

function injectedTypeErrorFile(app: AppConfig): string {
  return `__vize_intentional_type_error_${app.name.replace(/[^a-z0-9]+/gi, "_")}__.vue`;
}

export function runVizeCheckWithInjectedTypeError(
  app: AppConfig,
  options: InjectedTypeErrorOptions = {},
): InjectedTypeErrorSummary {
  const checkConfig = app.check!;
  const timeoutMs = options.timeoutMs ?? 300_000;
  const relativeDir = firstExistingPatternBase(checkConfig.cwd, checkConfig.patterns);
  const relativeFile = path.posix.join(
    relativeDir.replaceAll(path.sep, "/"),
    injectedTypeErrorFile(app),
  );
  const absoluteFile = path.join(checkConfig.cwd, relativeFile);
  const tempTsconfig = options.tsconfig;
  const absoluteTempTsconfig = tempTsconfig
    ? path.join(checkConfig.cwd, tempTsconfig.relativePath)
    : null;

  fs.writeFileSync(
    absoluteFile,
    `<script setup lang="ts">
const __vizeIntentionalTypeError: number = "not-a-number";
</script>

<template>
  <div>{{ __vizeIntentionalTypeError }}</div>
</template>
`,
    "utf-8",
  );
  if (tempTsconfig && absoluteTempTsconfig) {
    fs.mkdirSync(path.dirname(absoluteTempTsconfig), { recursive: true });
    fs.writeFileSync(absoluteTempTsconfig, tempTsconfig.content, "utf-8");
  }

  try {
    const startedAt = performance.now();
    const parsed = runVizeCheckJson(
      tempTsconfig ? { ...checkConfig, tsconfig: tempTsconfig.relativePath } : checkConfig,
      [relativeFile],
      timeoutMs,
    );
    const durationMs = performance.now() - startedAt;
    const files = parsed.files ?? [];
    const injected = files.find((file) => file.file?.replaceAll("\\", "/") === relativeFile);
    const diagnostics = injected?.diagnostics ?? [];

    assert.ok((parsed.fileCount ?? 0) > 0, "injected check fileCount should be > 0");
    assert.ok((parsed.errorCount ?? 0) > 0, "injected check should report at least one error");
    assert.ok(
      diagnostics.some((diagnostic) => /\[TS2322\]/.test(diagnostic)),
      `expected injected TS2322 diagnostic in ${relativeFile}, got ${JSON.stringify(files)}`,
    );
    assert.ok(durationMs < timeoutMs, `injected check should finish before ${timeoutMs}ms`);

    return {
      file: relativeFile,
      diagnostics,
      fileCount: parsed.fileCount ?? 0,
      errorCount: parsed.errorCount ?? 0,
      durationMs,
    };
  } finally {
    fs.rmSync(absoluteFile, { force: true });
    if (absoluteTempTsconfig) {
      fs.rmSync(absoluteTempTsconfig, { force: true });
    }
  }
}

export function runCrashFreeVizeLint(
  app: AppConfig,
  options: { timeoutMs?: number } = {},
): VizeLintSummary {
  const lintConfig = app.lint!;
  const timeoutMs = options.timeoutMs ?? 180_000;
  const patterns = lintConfig.patterns.map(shellQuote).join(" ");
  const cmd = [shellQuote(VIZE_BIN), "lint", patterns, "--format json", "--quiet"].join(" ");
  console.log(`Running: ${cmd}`);

  const startedAt = performance.now();
  const stdout = readCommandStdout(cmd, lintConfig.cwd, timeoutMs);
  const durationMs = performance.now() - startedAt;
  const parsed = JSON.parse(stdout) as unknown[];

  assert.ok(Array.isArray(parsed) && parsed.length > 0, "lint should produce file results");
  assert.ok(durationMs < timeoutMs, `lint should finish before ${timeoutMs}ms`);

  return {
    fileCount: parsed.length,
    durationMs,
  };
}

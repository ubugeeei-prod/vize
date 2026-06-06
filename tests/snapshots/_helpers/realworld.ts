import assert from "node:assert/strict";
import { execSync } from "node:child_process";

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

export function runCrashFreeVizeCheck(
  app: AppConfig,
  options: { timeoutMs?: number } = {},
): VizeCheckSummary {
  const checkConfig = app.check!;
  const timeoutMs = options.timeoutMs ?? 300_000;
  const patterns = checkConfig.patterns.map(shellQuote).join(" ");
  const tsconfig = checkConfig.tsconfig ? ` --tsconfig ${shellQuote(checkConfig.tsconfig)}` : "";
  const cmd = [
    shellQuote(VIZE_BIN),
    "check",
    patterns,
    "--format json",
    "--quiet",
    "--servers 1",
    "--corsa-path",
    shellQuote(CORSA_BIN),
    tsconfig,
  ]
    .filter(Boolean)
    .join(" ");
  console.log(`Running: ${cmd}`);

  const startedAt = performance.now();
  const stdout = readCommandStdout(cmd, checkConfig.cwd, timeoutMs);
  const durationMs = performance.now() - startedAt;
  const parsed = JSON.parse(stdout) as { fileCount?: number; errorCount?: number };

  assert.ok((parsed.fileCount ?? 0) > 0, "check fileCount should be > 0");
  assert.ok(durationMs < timeoutMs, `check should finish before ${timeoutMs}ms`);

  return {
    fileCount: parsed.fileCount ?? 0,
    errorCount: parsed.errorCount ?? 0,
    durationMs,
  };
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

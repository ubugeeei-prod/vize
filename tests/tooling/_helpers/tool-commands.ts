import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

export type CommandEnvironment = Record<string, string | undefined>;

export interface ToolCommandResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

/**
 * Runs a TypeScript command script with the current Node executable.
 * `undefined` environment values remove inherited variables.
 */
export function runToolCommand(
  script: string,
  args: readonly string[] = [],
  options: { cwd?: string; env?: CommandEnvironment } = {},
): ToolCommandResult {
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries({ ...process.env, ...options.env })) {
    if (value !== undefined) env[key] = value;
  }
  // Commands can launch their own `node --test` processes. Do not leak the
  // parent runner's private child marker into those independent runners.
  delete env.NODE_TEST_CONTEXT;
  const result = spawnSync(process.execPath, [path.join(repoRoot, script), ...args], {
    cwd: options.cwd ?? repoRoot,
    env,
    encoding: "utf8",
  });
  if (result.error !== undefined) throw result.error;
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

export interface RecordedCall {
  readonly name: string;
  readonly argv: string[];
  readonly cwd: string;
  readonly env: Record<string, string>;
}

/**
 * Writes a fake command that appends each invocation to `logPath`.
 * `exitCodes` are consumed in call order; the last one repeats. The command
 * runs its CommonJS body with the current Node executable, so it can also
 * stand in for `node` itself without resolving to its own wrapper.
 */
export function writeRecordingCommand(
  binDir: string,
  name: string,
  logPath: string,
  options: { exitCodes?: readonly number[]; stdout?: string; script?: string } = {},
): void {
  fs.mkdirSync(binDir, { recursive: true });
  const body = path.join(binDir, `.${name}.cjs`);
  fs.writeFileSync(
    body,
    [
      "const fs = require('node:fs');",
      `const logPath = ${JSON.stringify(logPath)};`,
      `const exitCodes = ${JSON.stringify(options.exitCodes ?? [0])};`,
      `const previous = fs.existsSync(logPath) ? fs.readFileSync(logPath, 'utf8').split('\\n').filter((line) => line && JSON.parse(line).name === ${JSON.stringify(name)}).length : 0;`,
      `fs.appendFileSync(logPath, JSON.stringify({ name: ${JSON.stringify(name)}, argv: process.argv.slice(2), cwd: process.cwd(), env: process.env }) + '\\n');`,
      options.script ?? "",
      `process.stdout.write(${JSON.stringify(options.stdout ?? "")});`,
      "process.exit(exitCodes[Math.min(previous, exitCodes.length - 1)]);",
      "",
    ].join("\n"),
  );
  const quote = (value: string) => `'${value.replaceAll("'", "'\\''")}'`;
  const unixPath = path.join(binDir, name);
  fs.writeFileSync(unixPath, `#!/bin/sh\nexec ${quote(process.execPath)} ${quote(body)} "$@"\n`);
  fs.chmodSync(unixPath, 0o755);
  if (process.platform === "win32") {
    fs.writeFileSync(
      path.join(binDir, `${name}.cmd`),
      `@echo off\r\n"${process.execPath}" "${body}" %*\r\n`,
    );
  }
}

export function readRecordedCalls(logPath: string): RecordedCall[] {
  if (!fs.existsSync(logPath)) return [];
  return fs
    .readFileSync(logPath, "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as RecordedCall);
}

/** Prepends `binDir` to the inherited `PATH`. */
export function pathWith(binDir: string): string {
  return `${binDir}${path.delimiter}${process.env.PATH ?? ""}`;
}

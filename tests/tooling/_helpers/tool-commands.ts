import {
  configuredNodeLauncher,
  shellLauncher,
  windowsLauncher,
} from "../../../tools/support/process-launcher.ts";
import { toolingProgram } from "../support/program-path.ts";
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
  options: {
    exitCodes?: readonly number[];
    stdout?: string;
    hook?: { name: string; args: string[] };
  } = {},
): void {
  fs.mkdirSync(binDir, { recursive: true });
  const body = path.join(binDir, `.${name}.cjs`);
  const hook = options.hook && {
    program: toolingProgram("process/" + options.hook.name + ".cjs"),
    args: options.hook.args,
  };
  fs.writeFileSync(
    body,
    configuredNodeLauncher(new URL("../programs/process/record-tool-call.cjs", import.meta.url), {
      logPath,
      name,
      hook,
      exitCodes: options.exitCodes ?? [0],
      stdout: options.stdout ?? "",
    }),
  );
  const unixPath = path.join(binDir, name);
  fs.writeFileSync(unixPath, shellLauncher("exec", { COMMAND: [process.execPath, body] }));
  fs.chmodSync(unixPath, 0o755);
  if (process.platform === "win32") {
    fs.writeFileSync(path.join(binDir, `${name}.cmd`), windowsLauncher([process.execPath, body]));
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

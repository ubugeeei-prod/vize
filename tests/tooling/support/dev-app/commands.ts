import { execFileSync, spawnSync, type SpawnSyncReturns } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { BASE_ENV, REPO_ROOT } from "./env.ts";

/**
 * Runs a foreground command and streams output directly to the terminal.
 *
 * Dev app startup is intentionally noisy: when a real-world fixture fails to
 * build, the developer needs the native Cargo, pnpm, or Docker output instead
 * of a summarized wrapper error.
 */
export function run(
  command: string,
  args: string[],
  cwd = REPO_ROOT,
  env?: Record<string, string>,
): void {
  console.log(`$ ${command} ${args.join(" ")}`);
  execFileSync(command, args, {
    cwd,
    env: {
      ...BASE_ENV,
      ...env,
    },
    stdio: "inherit",
  });
}

/**
 * Checks whether a command can be executed in the same environment used by
 * dev-app subprocesses.
 */
export function commandAvailable(command: string, args: string[] = ["--version"]): boolean {
  const result = spawnSync(command, args, {
    cwd: REPO_ROOT,
    env: BASE_ENV,
    stdio: "ignore",
  });
  return result.status === 0;
}

/**
 * Spawns a command and captures UTF-8 output for readiness probes.
 *
 * Captured probes are used for Misskey middleware checks where a failure should
 * become part of a clear retry error rather than streaming every transient
 * connection failure to the foreground terminal.
 */
export function spawnForOutput(
  command: string,
  args: string[],
  cwd: string,
  env?: Record<string, string>,
): SpawnSyncReturns<string> {
  return spawnSync(command, args, {
    cwd,
    env: {
      ...BASE_ENV,
      ...env,
    },
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

/**
 * Writes a file only when the content has changed.
 *
 * Several fixture setup steps run before every dev launch. Avoiding redundant
 * writes keeps file timestamps stable and prevents unrelated watchers from
 * rebuilding purely because the launcher refreshed generated config files.
 */
export function ensureFileContent(filePath: string, content: string): void {
  const current = fs.existsSync(filePath) ? fs.readFileSync(filePath, "utf-8") : null;
  if (current === content) {
    return;
  }

  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

export function sleep(milliseconds: number): void {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, milliseconds);
}

function trimOutput(output: string | Buffer | null | undefined): string {
  if (typeof output === "string") {
    return output.trim();
  }
  if (output == null) {
    return "";
  }
  return output.toString("utf8").trim();
}

/**
 * Combines captured stdout and stderr into the compact failure message used by
 * retry loops.
 */
export function formatCommandFailure(
  stdout: string | Buffer | null,
  stderr: string | Buffer | null,
): string {
  const trimmedStdout = trimOutput(stdout);
  const trimmedStderr = trimOutput(stderr);
  return [trimmedStdout, trimmedStderr].filter(Boolean).join("\n");
}

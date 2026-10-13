import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

/** Returns the TypeScript entry point of a release or publishing command. */
export function releaseCommandPath(name: string): string {
  return path.join(repoRoot, "tools", "support", "release", name, "main.ts");
}

/** Runs a release command with the current Node.js runtime and collects its output. */
export function runReleaseCommand(
  name: string,
  args: readonly string[] = [],
  options: { cwd?: string; env?: NodeJS.ProcessEnv } = {},
) {
  const env: NodeJS.ProcessEnv = { ...process.env, ...options.env };
  // Commands can launch their own `node --test` processes. Do not leak the
  // parent runner's private child marker into those independent runners.
  delete env.NODE_TEST_CONTEXT;
  return spawnSync(process.execPath, [releaseCommandPath(name), ...args], {
    cwd: options.cwd ?? repoRoot,
    env,
    encoding: "utf8",
  });
}

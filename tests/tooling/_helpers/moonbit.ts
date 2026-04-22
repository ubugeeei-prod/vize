import fs from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

export function moonScriptPath(name: string): string {
  return path.join(repoRoot, "tools", "moon", "scripts", `${name}.mbtx`);
}

function stripMoonCacheLogs(output: string): string {
  return output.replace(/^Using cached .*\n/gm, "");
}

export function runMoonScript(
  name: string,
  args: string[] = [],
  options: {
    cwd?: string;
    env?: NodeJS.ProcessEnv;
  } = {},
) {
  const result = spawnSync("moon", ["run", "-q", "--target", "native", "-", "--", ...args], {
    cwd: options.cwd ?? repoRoot,
    env: {
      ...process.env,
      ...options.env,
    },
    encoding: "utf8",
    input: fs.readFileSync(moonScriptPath(name), "utf8"),
  });
  return {
    ...result,
    stdout: stripMoonCacheLogs(result.stdout),
    stderr: stripMoonCacheLogs(result.stderr),
  };
}

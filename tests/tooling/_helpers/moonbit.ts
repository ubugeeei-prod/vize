import fs from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

export function moonScriptPath(name: string): string {
  return path.join(repoRoot, "tools", "moon", "scripts", `${name}.mbtx`);
}

function resolveRunnerShim(env: NodeJS.ProcessEnv): string | undefined {
  const runnerTemp = env.RUNNER_TEMP;
  if (!runnerTemp) {
    return undefined;
  }
  const shimPath = path.join(
    runnerTemp,
    "moonbit-shims",
    process.platform === "win32" ? "moon.cmd" : "moon",
  );
  return fs.existsSync(shimPath) ? shimPath : undefined;
}

function resolveMoonCommand(env: NodeJS.ProcessEnv): string {
  if (env.MOON_BIN) {
    return env.MOON_BIN;
  }
  const runnerShim = resolveRunnerShim(env);
  if (runnerShim) {
    return runnerShim;
  }
  return "moon";
}

function stripMoonCacheLogs(output: string): string {
  return output.replace(/^(Using cached|Downloading) .*\n/gm, "");
}

export function runMoonScript(
  name: string,
  args: string[] = [],
  options: {
    cwd?: string;
    env?: NodeJS.ProcessEnv;
  } = {},
) {
  const env = {
    ...process.env,
    ...options.env,
  };
  const moonCommand = resolveMoonCommand(env);
  const result = spawnSync(moonCommand, ["run", "-q", "--target", "native", "-", "--", ...args], {
    cwd: options.cwd ?? repoRoot,
    env,
    encoding: "utf8",
    input: fs.readFileSync(moonScriptPath(name), "utf8"),
  });
  return {
    ...result,
    stdout: stripMoonCacheLogs(result.stdout),
    stderr: stripMoonCacheLogs(result.stderr),
  };
}

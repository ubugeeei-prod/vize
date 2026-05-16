import fs from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const workspaceMoonHome = path.join(repoRoot, ".cache", "moonbit");
const workspaceMoonCommand = path.join(
  workspaceMoonHome,
  "bin",
  process.platform === "win32" ? "moon.cmd" : "moon",
);
const agentTempDir = path.join(repoRoot, "__agent_only", "moonbit-tmp");

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
  if (fs.existsSync(workspaceMoonCommand)) {
    return workspaceMoonCommand;
  }
  return "moon";
}

function stripMoonCacheLogs(output: string): string {
  return output.replace(/^(Using cached|Downloading) .*\n/gm, "");
}

function hasExplicitEnvValue(env: NodeJS.ProcessEnv | undefined, name: string): boolean {
  return Object.prototype.hasOwnProperty.call(env ?? {}, name);
}

export function runMoonScript(
  name: string,
  args: string[] = [],
  options: {
    buildOnly?: boolean;
    cwd?: string;
    denyWarn?: boolean;
    env?: NodeJS.ProcessEnv;
  } = {},
) {
  fs.mkdirSync(agentTempDir, { recursive: true });
  const env = {
    ...process.env,
    ...options.env,
  };
  if (!hasExplicitEnvValue(options.env, "TMPDIR")) {
    env.TMPDIR = agentTempDir;
  }
  if (!hasExplicitEnvValue(options.env, "TEMP")) {
    env.TEMP = agentTempDir;
  }
  if (!hasExplicitEnvValue(options.env, "TMP")) {
    env.TMP = agentTempDir;
  }
  const moonCommand = resolveMoonCommand(env);
  if (moonCommand === workspaceMoonCommand && !hasExplicitEnvValue(options.env, "MOON_HOME")) {
    env.MOON_HOME = workspaceMoonHome;
  }
  if (moonCommand === workspaceMoonCommand && !hasExplicitEnvValue(options.env, "MOON_BIN")) {
    env.MOON_BIN = workspaceMoonCommand;
  }
  const runArgs = [
    "run",
    "-q",
    ...(options.buildOnly ? ["--build-only"] : []),
    ...(options.denyWarn ? ["--deny-warn"] : []),
    "--target",
    "native",
    "-",
    "--",
    ...args,
  ];
  const result = spawnSync(moonCommand, runArgs, {
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

import fs from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const testOutputRoot = path.join(repoRoot, "target", "vize-tests");
const workspaceMoonHome = path.join(repoRoot, ".cache", "moonbit");
const workspaceMoonCommand = path.join(
  workspaceMoonHome,
  "bin",
  process.platform === "win32" ? "moon.cmd" : "moon",
);
const moonbitTempDir = path.join(testOutputRoot, "moonbit-tmp");

export function moonScriptPath(name: string): string {
  return path.join(repoRoot, "tools", "moon", "cmd", ...name.split("/"), "main.mbt");
}

export function moonScriptPackagePath(name: string): string {
  return path.join(repoRoot, "tools", "moon", "cmd", ...name.split("/"));
}

function moonScriptPackageArg(name: string, cwd: string): string {
  const packagePath = moonScriptPackagePath(name);
  return cwd === repoRoot ? path.relative(repoRoot, packagePath) : packagePath;
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
  fs.mkdirSync(moonbitTempDir, { recursive: true });
  const env = {
    ...process.env,
    ...options.env,
  };
  // Moon commands can launch their own `node --test` processes. Do not leak
  // the parent runner's private child marker into those independent runners.
  delete env.NODE_TEST_CONTEXT;
  if (!hasExplicitEnvValue(options.env, "TMPDIR")) {
    env.TMPDIR = moonbitTempDir;
  }
  if (!hasExplicitEnvValue(options.env, "TEMP")) {
    env.TEMP = moonbitTempDir;
  }
  if (!hasExplicitEnvValue(options.env, "TMP")) {
    env.TMP = moonbitTempDir;
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
    moonScriptPackageArg(name, options.cwd ?? repoRoot),
    "--",
    ...args,
  ];
  const result = spawnSync(moonCommand, runArgs, {
    cwd: options.cwd ?? repoRoot,
    env,
    encoding: "utf8",
  });
  return {
    ...result,
    stdout: stripMoonCacheLogs(result.stdout),
    stderr: stripMoonCacheLogs(result.stderr),
  };
}

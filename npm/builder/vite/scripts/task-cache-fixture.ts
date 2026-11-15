import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export type FixtureInstaller = "bun" | "npm";

export interface TaskRunOptions {
  hit?: boolean;
  status?: number;
  diagnostic?: RegExp;
  args?: string[];
  unchanged?: boolean;
}

export interface TaskRunReceipt {
  label: string;
  task: string;
  args: string[];
  status: number | null;
  stdout: string;
  stderr: string;
  details: string;
  cacheHit: boolean;
  before: Record<string, string>;
  after: Record<string, string>;
  milliseconds: number;
}

type RequireSuccess = (binary: string, args: string[], cwd?: string) => SpawnSyncReturns<string>;
type RunTask = (label: string, task: string) => unknown;

interface CacheProbeRow {
  label: string;
  cacheHit: boolean;
}

interface CacheProbeOptions {
  directory: string;
  artifactsDirectory: string;
  nodeModules: string;
  cachedTasks: string[];
  unexpectedMisses: string[];
  rows: CacheProbeRow[];
  run: RunTask;
  install: (fresh?: boolean) => void;
  installSourceArchive: (archive: string) => void;
  sourceArchivePath?: string;
}

export function createFixtureCommands(
  directory: string,
  env: NodeJS.ProcessEnv,
): {
  command: (binary: string, args: string[], cwd?: string) => SpawnSyncReturns<string>;
  requireSuccess: RequireSuccess;
} {
  const command = (binary: string, args: string[], cwd = directory): SpawnSyncReturns<string> => {
    const result = spawnSync(binary, args, {
      cwd,
      env,
      encoding: "utf8",
      timeout: 180_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.signal, null);
    return result;
  };
  const requireSuccess = (
    binary: string,
    args: string[],
    cwd?: string,
  ): SpawnSyncReturns<string> => {
    const result = command(binary, args, cwd);
    assert.equal(result.status, 0, `${args.join(" ")}\n${result.stdout}\n${result.stderr}`);
    return result;
  };
  return { command, requireSuccess };
}

export function installFixture(
  requireSuccess: RequireSuccess,
  installer: FixtureInstaller,
  fresh = false,
): void {
  if (installer === "bun") {
    requireSuccess("bun", ["install", "--no-progress", ...(fresh ? ["--frozen-lockfile"] : [])]);
  } else {
    requireSuccess("npm", [
      fresh ? "ci" : "install",
      "--legacy-peer-deps",
      "--no-audit",
      "--no-fund",
    ]);
  }
}

export function installSourceArchive(requireSuccess: RequireSuccess, archive: string): void {
  // npm's no-save overlay avoids changing either package manager's manifest
  // or lockfile while replacing only the package built from this checkout.
  requireSuccess("npm", [
    "install",
    "--no-save",
    "--package-lock=false",
    "--legacy-peer-deps",
    "--no-audit",
    "--no-fund",
    archive,
  ]);
}

export async function installSourceCliLauncher(
  sourceCli: string,
  expected: string | undefined,
  receipt: string | undefined,
  require: NodeRequire,
): Promise<unknown> {
  assert.equal(process.platform, "linux");
  assert.ok(expected && /^[a-f0-9]{64}$/u.test(expected));
  const hash = createHash("sha256");
  for await (const chunk of fs.createReadStream(sourceCli)) hash.update(chunk);
  assert.equal(hash.digest("hex"), expected, "the exact verified source CLI executes");
  assert.ok(receipt);
  const launcher = path.resolve(path.dirname(require.resolve("vize")), "../bin/vize");
  const originalLauncher = fs.readFileSync(launcher, "utf8");
  // Only this disposable install's launcher delegates to the real, verified
  // Rust CLI. Its original arguments, environment and all streams pass through.
  fs.writeFileSync(
    launcher,
    `#!/usr/bin/env node\nimport { spawn } from "node:child_process";\nconsole.error("VIZE_TASK_CACHE_SOURCE_ENV " + JSON.stringify({ cwd: process.cwd(), xdgCacheHome: process.env.XDG_CACHE_HOME, tmpdir: process.env.TMPDIR, argv: process.argv.slice(2) }));\nconst child = spawn(${JSON.stringify(sourceCli)}, process.argv.slice(2), { stdio: "inherit" });\nconst stop = (signal) => { if (!child.killed) child.kill(signal); };\nprocess.on("SIGTERM", () => stop("SIGTERM"));\nprocess.on("SIGINT", () => stop("SIGINT"));\nchild.on("error", (error) => { throw error; });\nchild.on("exit", (code, signal) => {\n  if (signal) process.kill(process.pid, signal);\n  else process.exitCode = code ?? 1;\n});\n`,
  );
  return {
    verified: JSON.parse(fs.readFileSync(receipt, "utf8")) as unknown,
    launcher: {
      path: launcher,
      original: originalLauncher,
      originalSha256: createHash("sha256").update(originalLauncher).digest("hex"),
      delegatedSha256: createHash("sha256").update(fs.readFileSync(launcher)).digest("hex"),
    },
  };
}

export function runNodeModulesCacheProbes(options: CacheProbeOptions): void {
  const probe = (label: string, mutate: () => void): void => {
    mutate();
    for (const task of options.cachedTasks) {
      options.run(`${label}-${task}`, task);
      const row = options.rows.at(-1)!;
      if (!row.cacheHit) options.unexpectedMisses.push(row.label);
    }
  };
  const dependencyProbe = path.join(options.nodeModules, "task-cache-install-probe");
  probe("node-modules-entry-added", () => {
    fs.mkdirSync(dependencyProbe);
    fs.writeFileSync(path.join(dependencyProbe, "package.json"), "{}\n");
  });
  probe("node-modules-entry-removed", () => {
    fs.rmSync(dependencyProbe, { recursive: true });
  });
  const toolCache = path.join(options.nodeModules, ".cache");
  probe("tool-cache-added", () => {
    fs.mkdirSync(toolCache);
    fs.writeFileSync(path.join(toolCache, "result.json"), "{}\n");
  });
  probe("tool-cache-removed", () => {
    fs.rmSync(toolCache, { recursive: true });
  });
  probe("vite-task-cache-churn", () => {
    const marker = path.join(options.nodeModules, ".vite/task-cache/v18/external-probe.json");
    fs.mkdirSync(path.dirname(marker), { recursive: true });
    fs.writeFileSync(marker, "{}\n");
  });

  const taskCache = path.join(options.nodeModules, ".vite/task-cache");
  const preservedTaskCache = path.join(options.artifactsDirectory, "task-cache");
  assert.ok(fs.existsSync(taskCache), "Vite Task cache exists before fresh install");
  fs.cpSync(taskCache, preservedTaskCache, { recursive: true });
  fs.rmSync(options.nodeModules, { recursive: true });
  options.install(true);
  if (options.sourceArchivePath) options.installSourceArchive(options.sourceArchivePath);
  fs.mkdirSync(path.dirname(taskCache), { recursive: true });
  fs.cpSync(preservedTaskCache, taskCache, { recursive: true });
  probe("fresh-install-with-restored-task-cache", () => {});

  assert.deepEqual(
    options.unexpectedMisses,
    [],
    `node_modules-only changes must retain read-only task cache entries: ${options.unexpectedMisses.join(", ")}`,
  );
}

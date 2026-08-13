#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { validateTypecheckPerformanceTarget } from "./tool-matrix-typecheck-target.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const defaultRegistry = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

export function runTypecheckDependencyPrepare(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const registry = readJson(args.registry);
  const selected = registry.projects
    .filter((_, index) => index % args.shardCount === args.shardIndex)
    .filter((project) => project.typecheckPerformance?.enabled === true);
  if (selected.length !== 1) {
    throw new Error(
      `Expected exactly one typecheck performance project in shard ${args.shardIndex}/${args.shardCount}, found ${selected.length}`,
    );
  }

  const project = selected[0];
  const fixtureRoot = resolve(repoRoot, project.fixturePath);
  validateTypecheckPerformanceTarget(project, fixtureRoot);
  requireDirectory(args.outputDir);
  const commitSha = requireCommitSha(process.env.GITHUB_SHA);
  const performance = project.typecheckPerformance;
  const lockfilePath = resolve(fixtureRoot, performance.lockfile);
  const lockfileBefore = readFileSync(lockfilePath);
  requireCleanFixture(fixtureRoot, "before dependency installation");

  const manager = performance.packageManager;
  const probe = spawnSync(manager, ["--version"], {
    cwd: fixtureRoot,
    encoding: "utf8",
    timeout: 10_000,
  });
  if (probe.error != null || probe.status !== 0) {
    throw new Error(`${manager} is not runnable: ${errorMessage(probe.error ?? probe.status)}`);
  }
  const detectedVersion = (probe.stdout ?? "").trim();
  if (detectedVersion !== performance.packageManagerVersion) {
    throw new Error(
      `Detected ${manager} version ${detectedVersion} does not match ${performance.packageManagerVersion}`,
    );
  }

  const installArgs = installArguments(manager);
  const startedAt = Date.now();
  const install = spawnSync(manager, installArgs, {
    cwd: fixtureRoot,
    encoding: "utf8",
    env: {
      ...process.env,
      CI: "true",
      npm_config_ignore_scripts: "true",
      YARN_ENABLE_SCRIPTS: "false",
    },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: args.timeoutMs,
  });
  const durationMs = Date.now() - startedAt;
  if (install.error != null) {
    throw new Error(`${manager} install failed to run: ${errorMessage(install.error)}`);
  }
  if (install.status !== 0) {
    throw new Error(`${manager} install exited with status ${install.status}`);
  }

  const lockfileAfter = readFileSync(lockfilePath);
  if (!lockfileBefore.equals(lockfileAfter)) {
    throw new Error(`${manager} install modified frozen lockfile ${performance.lockfile}`);
  }
  requireCleanFixture(fixtureRoot, "after dependency installation");
  const baselinePrepare = runBaselinePrepare(project, fixtureRoot, args.timeoutMs);
  validateTypecheckPerformanceTarget(project, fixtureRoot, { requireBaseline: true });
  requireCleanFixture(fixtureRoot, "after baseline preparation");
  const artifact = {
    schema: "vize.fixtureTypecheckDependencyInstall",
    version: 2,
    project: project.id,
    revision: project.revision,
    evidence: {
      commitSha,
      runtime: { name: "node", version: process.versions.node },
    },
    packageManager: { name: manager, version: detectedVersion },
    lockfile: {
      path: performance.lockfile,
      sizeBytes: lockfileAfter.byteLength,
      sha256: sha256(lockfileAfter),
    },
    install: {
      command: [manager, ...installArgs],
      durationMs,
      exitCode: install.status,
      stdoutSha256: sha256(install.stdout ?? ""),
      stderrSha256: sha256(install.stderr ?? ""),
    },
    baselinePrepare,
  };
  const artifactPath = join(args.outputDir, `${project.id}-typecheck-dependencies.json`);
  writeFileSync(artifactPath, `${JSON.stringify(artifact, null, 2)}\n`);
  process.stdout.write(`Wrote ${relative(repoRoot, artifactPath)}\n`);
  return artifact;
}

function runBaselinePrepare(project, fixtureRoot, timeoutMs) {
  const command = project.typecheckPerformance.baseline?.prepare;
  if (command == null) return null;
  const startedAt = Date.now();
  const prepared = spawnSync(command[0], command.slice(1), {
    cwd: fixtureRoot,
    encoding: "utf8",
    env: { ...process.env, CI: "true", NUXT_TELEMETRY_DISABLED: "1" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: timeoutMs,
  });
  const durationMs = Date.now() - startedAt;
  if (prepared.error != null) {
    throw new Error(`baseline prepare failed to run: ${errorMessage(prepared.error)}`);
  }
  if (prepared.status !== 0) {
    throw new Error(`baseline prepare exited with status ${prepared.status}`);
  }
  return {
    command,
    durationMs,
    exitCode: prepared.status,
    stdoutSha256: sha256(prepared.stdout ?? ""),
    stderrSha256: sha256(prepared.stderr ?? ""),
  };
}

export function installArguments(manager) {
  return {
    npm: ["ci", "--ignore-scripts", "--prefer-offline", "--no-audit", "--no-fund"],
    pnpm: ["install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
    yarn: ["install", "--immutable", "--mode=skip-build"],
  }[manager];
}

function requireCleanFixture(fixtureRoot, phase) {
  const status = spawnSync("git", ["status", "--porcelain=v1", "--untracked-files=no"], {
    cwd: fixtureRoot,
    encoding: "utf8",
    timeout: 10_000,
  });
  if (status.error != null || status.status !== 0) {
    throw new Error(`Unable to inspect fixture source ${phase}`);
  }
  if (status.stdout !== "") throw new Error(`Fixture tracked source changed ${phase}`);
}

function parseArgs(argv) {
  const args = {
    outputDir: null,
    registry: defaultRegistry,
    shardCount: 1,
    shardIndex: 0,
    timeoutMs: 600_000,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const value = () => {
      if (argv[index + 1] == null) throw new Error(`${arg} requires a value`);
      return argv[++index];
    };
    if (arg === "--output-dir") args.outputDir = resolve(repoRoot, value());
    else if (arg === "--registry") args.registry = resolve(repoRoot, value());
    else if (arg === "--shard-count") args.shardCount = integer(value(), arg, 1);
    else if (arg === "--shard-index") args.shardIndex = integer(value(), arg, 0);
    else if (arg === "--timeout-ms") args.timeoutMs = integer(value(), arg, 1);
    else throw new Error(`Unknown argument: ${arg}`);
  }
  if (args.outputDir == null) throw new Error("--output-dir is required");
  if (args.shardIndex >= args.shardCount) {
    throw new Error("--shard-index must be less than --shard-count");
  }
  return args;
}

function requireDirectory(path) {
  let metadata;
  try {
    metadata = statSync(path);
  } catch {
    throw new Error(`Output directory does not exist: ${path}`);
  }
  if (!metadata.isDirectory()) throw new Error(`Output path is not a directory: ${path}`);
}

function requireCommitSha(value) {
  if (!/^[0-9a-f]{40}$/.test(value ?? "")) {
    throw new Error("GITHUB_SHA must be a full lowercase commit SHA");
  }
  return value;
}

function integer(value, name, minimum) {
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < minimum) {
    throw new Error(`${name} must be a safe integer >= ${minimum}`);
  }
  return parsed;
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === resolve(process.argv[1])
  : false;
if (entrypoint) {
  try {
    runTypecheckDependencyPrepare();
  } catch (error) {
    process.stderr.write(`${errorMessage(error)}\n`);
    process.exit(1);
  }
}

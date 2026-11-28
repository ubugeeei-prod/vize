import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  baselinePrepareCommands,
  validateTypecheckPerformanceTarget,
} from "./tool-matrix-typecheck-target.ts";
import { runTypecheckCommand } from "./typecheck-command-runner.mjs";
import { isolateFixtureTypePackages } from "./typecheck-baseline-isolation.ts";
import {
  isolateUniqueLocalTypePackages,
  isolateUniqueUiLibraryPackages,
  isolateUniqueVueEditorPackages,
  isolateUniqueVueFormPackages,
  isolateUniqueVueI18nPackages,
  isolateUniqueVueQueryPackages,
  isolateUniqueVueRuntimePackages,
  isolateUniqueVueUsePackages,
} from "./typecheck-baseline-isolation-unique.ts";
import { applyIsolatedTypecheckOverlays } from "./typecheck-baseline-outside-overlays.ts";
import {
  commandOptions,
  installArguments,
  packageManagerRunner,
  qualifyLockfileBytes,
  runPackageManager,
} from "./typecheck-package-manager.ts";
import { prepareTypecheckSetupProject } from "./typecheck-setup-prepare.ts";
import { typecheckUnitTsconfigs } from "./typecheck-targets.ts";
import { prepareTypecheckInputOverlays } from "./typecheck-input-overlays.ts";

export { installArguments } from "./typecheck-package-manager.ts";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..", "..", "..");
const defaultRegistry = join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

export async function runTypecheckDependencyPrepare(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const registry = readJson(args.registry);
  const selected = selectTypecheckDependencyProjects(registry, args);
  const commitSha = requireCommitSha(process.env.GITHUB_SHA);
  requireDirectory(args.outputDir);
  if (selected.length === 0) {
    process.stdout.write(
      `No typecheck performance projects selected or setup projects selected for shard ${args.shardIndex}/${args.shardCount}\n`,
    );
    return [];
  }
  const artifacts = [];
  for (const project of selected) {
    artifacts.push(
      project.typecheckPerformance?.enabled === true
        ? await preparePerformanceProject({ args, commitSha, project })
        : await prepareTypecheckSetupProject({ args, commitSha, project, repoRoot }),
    );
  }
  return artifacts;
}

function selectTypecheckDependencyProjects(registry, { shardIndex, shardCount }) {
  if (!Array.isArray(registry?.projects)) throw new Error("Fixture registry must list projects");
  return registry.projects.filter(
    (project, index) =>
      index % shardCount === shardIndex &&
      (project.typecheckPerformance?.enabled === true || project.typecheckSetup?.enabled === true),
  );
}

async function preparePerformanceProject({ args, commitSha, project }) {
  const fixtureRoot = resolve(repoRoot, project.fixturePath);
  validateTypecheckPerformanceTarget(project, fixtureRoot);
  const performance = project.typecheckPerformance;
  const lockfilePath = resolve(fixtureRoot, performance.lockfile);
  const lockfileBefore = readFileSync(lockfilePath);
  const qualifiedLockfile = qualifyLockfileBytes(performance.packageManager, lockfileBefore, {
    workspacePackages: performance.workspacePackages,
  });
  requireCleanFixture(fixtureRoot, "before dependency installation");

  const manager = performance.packageManager;
  const managerRunner = packageManagerRunner(manager, performance.packageManagerVersion);
  const probe = await runPackageManager(managerRunner, ["--version"], {
    cwd: fixtureRoot,
    encoding: "utf8",
    timeout: 10_000,
  });
  if (probe.error != null || probe.status !== 0) {
    throw new Error(
      `${managerRunner.label} is not runnable: ${errorMessage(probe.error ?? probe.status)}`,
    );
  }
  const detectedVersion = (probe.stdout ?? "").trim();
  if (detectedVersion !== performance.packageManagerVersion) {
    throw new Error(
      `Detected ${manager} version ${detectedVersion} does not match ${performance.packageManagerVersion}`,
    );
  }

  const installArgs = installArguments(manager, fixtureRoot, performance.packageManagerVersion, {
    nodeLinker: performance.nodeLinker,
    workspacePackages: performance.workspacePackages,
  });
  const startedAt = Date.now();
  const transformedLockfile = !lockfileBefore.equals(qualifiedLockfile);
  if (transformedLockfile) writeFileSync(lockfilePath, qualifiedLockfile);
  let install;
  let lockfileAfter;
  try {
    install = await runPackageManager(managerRunner, installArgs, {
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
    lockfileAfter = readFileSync(lockfilePath);
  } finally {
    if (transformedLockfile) writeFileSync(lockfilePath, lockfileBefore);
  }
  const durationMs = Date.now() - startedAt;
  if (install.error != null) {
    throw new Error(`${managerRunner.label} install failed to run: ${errorMessage(install.error)}`);
  }
  if (install.status !== 0) {
    throw new Error(
      `${managerRunner.label} install exited with status ${install.status}${commandDiagnostic(install)}`,
    );
  }

  if (!qualifiedLockfile.equals(lockfileAfter)) {
    throw new Error(`${manager} install modified frozen lockfile ${performance.lockfile}`);
  }
  requireCleanFixture(fixtureRoot, "after dependency installation");
  const baselinePrepare = await runBaselinePrepare(
    project,
    fixtureRoot,
    args.timeoutMs,
    managerRunner,
  );
  validateTypecheckPerformanceTarget(project, fixtureRoot, { requireBaseline: true });
  // Both tools run against this tree, so the type environment is closed here
  // rather than in the divergence report — a repair only the baseline saw would
  // be a second way for the two sides to measure different programs.
  isolateFixture(project, fixtureRoot);
  requireCleanFixture(fixtureRoot, "after baseline preparation");
  const artifact = {
    schema: "vize.fixtureTypecheckDependencyInstall",
    version: 3,
    project: project.id,
    revision: project.revision,
    evidence: {
      commitSha,
      runtime: { name: "node", version: process.versions.node },
    },
    packageManager: { name: manager, version: detectedVersion },
    lockfile: {
      path: performance.lockfile,
      sizeBytes: lockfileBefore.byteLength,
      sha256: sha256(lockfileBefore),
    },
    qualificationLockfile: transformedLockfile
      ? {
          mode: "pnpm-injected-workspace-packages",
          sizeBytes: qualifiedLockfile.byteLength,
          sha256: sha256(qualifiedLockfile),
        }
      : null,
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

/**
 * Overlay unhoisted declared packages from both Vize and vue-tsc configs so
 * type-reference walks cannot climb into Vize (run 31979524200).
 */
export function isolationTsconfigPaths(project) {
  return typecheckUnitTsconfigs(project);
}

function isolateFixture(project, fixtureRoot) {
  const shadowed = [];
  const seen = new Set();
  for (const relativeConfig of isolationTsconfigPaths(project)) {
    const sourceConfig = resolve(fixtureRoot, relativeConfig);
    for (const entry of [
      ...isolateFixtureTypePackages(fixtureRoot, sourceConfig),
      ...isolateUniqueLocalTypePackages(fixtureRoot, sourceConfig),
    ]) {
      if (seen.has(entry.name)) continue;
      seen.add(entry.name);
      shadowed.push(entry);
    }
  }
  for (const isolate of [
    isolateUniqueVueRuntimePackages,
    isolateUniqueVueI18nPackages,
    isolateUniqueVueUsePackages,
    isolateUniqueUiLibraryPackages,
    isolateUniqueVueFormPackages,
    isolateUniqueVueQueryPackages,
    isolateUniqueVueEditorPackages,
  ]) {
    for (const entry of isolate(fixtureRoot)) {
      if (seen.has(entry.name)) continue;
      seen.add(entry.name);
      shadowed.push(entry);
    }
  }
  for (const entry of shadowed) {
    process.stdout.write(`Linked ${project.id} node_modules/${entry.name} -> ${entry.target}\n`);
  }
  for (const relativeConfig of isolationTsconfigPaths(project)) {
    const sourceConfig = resolve(fixtureRoot, relativeConfig);
    const overlay = applyIsolatedTypecheckOverlays(fixtureRoot, sourceConfig);
    if (overlay != null) {
      process.stdout.write(
        `Rewrote ${project.id} tsconfig overlay -> ${relative(fixtureRoot, overlay.path)}\n`,
      );
    }
  }
}

async function runBaselinePrepare(project, fixtureRoot, timeoutMs, managerRunner) {
  const command = project.typecheckPerformance.baseline?.prepare;
  if (command == null) return null;
  const commands = baselinePrepareCommands(command);
  if (commands == null) throw new Error("baseline prepare must be a command or a list of them");
  const inputOverlays = prepareTypecheckInputOverlays(
    repoRoot,
    fixtureRoot,
    project.revision,
    project.typecheckPerformance.baseline?.inputOverlays,
  );
  const startedAt = Date.now();
  let stdout = "";
  let stderr = "";
  // Commands run in order; each one sees the inputs the previous one generated.
  for (const [index, step] of commands.entries()) {
    const label = commands.length === 1 ? "baseline prepare" : `baseline prepare step ${index + 1}`;
    const prepared = await runBaselineCommand(step, fixtureRoot, timeoutMs, managerRunner);
    if (prepared.error != null) {
      throw new Error(`${label} failed to run: ${errorMessage(prepared.error)}`);
    }
    if (prepared.status !== 0) {
      throw new Error(
        `${label} exited with status ${prepared.status}${commandDiagnostic(prepared)}`,
      );
    }
    stdout += prepared.stdout ?? "";
    stderr += prepared.stderr ?? "";
  }
  const durationMs = Date.now() - startedAt;
  if (project.id === "misskey") validateMisskeyBuildOutputs(fixtureRoot);
  return {
    command,
    ...(inputOverlays.length > 0 ? { inputOverlays } : {}),
    durationMs,
    exitCode: 0,
    stdoutSha256: sha256(stdout),
    stderrSha256: sha256(stderr),
  };
}

async function runBaselineCommand(command, fixtureRoot, timeoutMs, managerRunner) {
  const options = {
    cwd: fixtureRoot,
    encoding: "utf8",
    env: { ...process.env, CI: "true", NUXT_TELEMETRY_DISABLED: "1" },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: timeoutMs,
  };
  if (command[0] === managerRunner.manager) {
    return runPackageManager(managerRunner, command.slice(1), options);
  }
  return runTypecheckCommand(command[0], command.slice(1), commandOptions(options));
}

function validateMisskeyBuildOutputs(fixtureRoot) {
  for (const packageName of ["misskey-js", "misskey-reversi", "misskey-bubble-game", "i18n"]) {
    const declaration = join(fixtureRoot, "packages", packageName, "built", "index.d.ts");
    if (!existsSync(declaration)) {
      throw new Error(`${packageName} declaration output is missing`);
    }
    const metadata = statSync(declaration);
    if (!metadata.isFile() || metadata.size === 0) {
      throw new Error(`${packageName} declaration output must be a nonempty regular file`);
    }
  }
  const i18nRuntime = join(fixtureRoot, "packages", "i18n", "built", "index.js");
  if (!existsSync(i18nRuntime)) {
    throw new Error("i18n runtime output is missing");
  }
  const runtimeMetadata = statSync(i18nRuntime);
  if (!runtimeMetadata.isFile() || runtimeMetadata.size === 0) {
    throw new Error("i18n runtime output must be a nonempty regular file");
  }
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

const ansiSgrPattern = new RegExp(`${String.fromCharCode(27)}\\[[0-9;]*m`, "gu");

function commandDiagnostic(result) {
  const lines = [result.stderr, result.stdout]
    .flatMap((stream) => String(stream ?? "").split(/\r?\n/u))
    .map((line) => line.replace(ansiSgrPattern, "").trim())
    .filter(Boolean);
  const line = lines.find((candidate) => candidate.includes("ERR_")) ?? lines[0];
  return line == null ? "" : `: ${line.slice(0, 500)}`;
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === resolve(process.argv[1])
  : false;
if (entrypoint) {
  try {
    await runTypecheckDependencyPrepare();
  } catch (error) {
    process.stderr.write(`${errorMessage(error)}\n`);
    process.exit(1);
  }
}

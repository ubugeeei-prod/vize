import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, statSync, writeFileSync } from "node:fs";
import { isAbsolute, join, relative, resolve } from "node:path";

import {
  installArguments,
  packageManagerRunner,
  runPackageManager,
} from "./typecheck-package-manager.ts";

type SetupProject = {
  id: string;
  revision: string;
  fixturePath: string;
  typecheckSetup?: {
    enabled?: boolean;
    workingDirectory?: string;
    packageManager: "npm" | "pnpm" | "yarn";
    packageManagerVersion: string;
    lockfile: string;
    prepare?: string[] | null;
    requiredOutputs: string[];
  };
};

type PrepareArguments = { outputDir: string; timeoutMs: number };

export async function prepareTypecheckSetupProject({
  args,
  commitSha,
  project,
  repoRoot,
}: {
  args: PrepareArguments;
  commitSha: string;
  project: SetupProject;
  repoRoot: string;
}) {
  const fixtureRoot = resolve(repoRoot, project.fixturePath);
  const setup = validateTypecheckSetupTarget(project, fixtureRoot);
  const workingDirectory = resolve(fixtureRoot, setup.workingDirectory);
  const lockfilePath = resolve(workingDirectory, setup.lockfile);
  const lockfileBefore = readFileSync(lockfilePath);
  requireCleanFixture(fixtureRoot, "before typecheck setup dependency installation");

  const managerRunner = packageManagerRunner(setup.packageManager, setup.packageManagerVersion);
  const probe = await runPackageManager(managerRunner, ["--version"], {
    cwd: workingDirectory,
    encoding: "utf8",
    timeout: 10_000,
  });
  if (probe.error != null || probe.status !== 0) {
    throw new Error(
      `${managerRunner.label} is not runnable: ${errorMessage(probe.error ?? probe.status)}`,
    );
  }
  const detectedVersion = (probe.stdout ?? "").trim();
  if (detectedVersion !== setup.packageManagerVersion) {
    throw new Error(
      `Detected ${setup.packageManager} version ${detectedVersion} does not match ${setup.packageManagerVersion}`,
    );
  }

  const installArgs = installArguments(
    setup.packageManager,
    workingDirectory,
    setup.packageManagerVersion,
  );
  const install = await runRecordedCommand({
    command: [setup.packageManager, ...installArgs],
    cwd: workingDirectory,
    label: `${managerRunner.label} install`,
    managerRunner,
    timeoutMs: args.timeoutMs,
  });
  const lockfileAfter = readFileSync(lockfilePath);
  if (!lockfileBefore.equals(lockfileAfter)) {
    throw new Error(
      `${setup.packageManager} install modified frozen lockfile ${relative(fixtureRoot, lockfilePath)}`,
    );
  }
  requireCleanFixture(fixtureRoot, "after typecheck setup dependency installation");

  const prepare =
    setup.prepare == null
      ? null
      : await runRecordedCommand({
          command: setup.prepare,
          cwd: workingDirectory,
          label: "typecheck setup prepare",
          managerRunner,
          timeoutMs: args.timeoutMs,
        });
  const requiredOutputs = setup.requiredOutputs.map((path) => {
    const absolutePath = resolve(workingDirectory, path);
    const metadata = statSync(absolutePath);
    if (!metadata.isFile() || metadata.size === 0) {
      throw new Error(`Typecheck setup output must be a nonempty regular file: ${path}`);
    }
    const bytes = readFileSync(absolutePath);
    return {
      path: relative(fixtureRoot, absolutePath),
      sizeBytes: bytes.byteLength,
      sha256: sha256(bytes),
    };
  });
  requireCleanFixture(fixtureRoot, "after typecheck setup preparation");

  const artifact = {
    schema: "vize.fixtureTypecheckSetup",
    version: 1,
    project: project.id,
    revision: project.revision,
    evidence: {
      commitSha,
      runtime: { name: "node", version: process.versions.node },
    },
    workingDirectory: relative(fixtureRoot, workingDirectory) || ".",
    packageManager: { name: setup.packageManager, version: detectedVersion },
    lockfile: {
      path: relative(fixtureRoot, lockfilePath),
      sizeBytes: lockfileAfter.byteLength,
      sha256: sha256(lockfileAfter),
    },
    install,
    prepare,
    requiredOutputs,
  };
  const artifactPath = join(args.outputDir, `${project.id}-typecheck-dependencies.json`);
  writeFileSync(artifactPath, `${JSON.stringify(artifact, null, 2)}\n`);
  process.stdout.write(`Wrote ${relative(repoRoot, artifactPath)}\n`);
  return artifact;
}

function validateTypecheckSetupTarget(project: SetupProject, fixtureRoot: string) {
  const setup = project.typecheckSetup;
  if (setup?.enabled !== true) {
    throw new Error(
      `Invalid typecheck setup target for ${String(project.id)}: enabled must be true`,
    );
  }
  const invalid = (message: string): never => {
    throw new Error(`Invalid typecheck setup target for ${String(project.id)}: ${message}`);
  };
  const workingDirectory = setup.workingDirectory ?? ".";
  if (workingDirectory !== ".") {
    requireNormalizedRelativePath(workingDirectory, "workingDirectory", invalid);
  }
  const resolvedWorkingDirectory = resolve(fixtureRoot, workingDirectory);
  if (!statSync(resolvedWorkingDirectory).isDirectory()) {
    invalid("workingDirectory must be a directory");
  }
  if (!["npm", "pnpm", "yarn"].includes(setup.packageManager)) {
    invalid("packageManager must be npm, pnpm, or yarn");
  }
  if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(setup.packageManagerVersion ?? "")) {
    invalid("packageManagerVersion must be an exact semantic version");
  }
  const expectedLockfile = {
    npm: "package-lock.json",
    pnpm: "pnpm-lock.yaml",
    yarn: "yarn.lock",
  }[setup.packageManager];
  if (setup.lockfile !== expectedLockfile) invalid(`lockfile must be ${expectedLockfile}`);
  const lockfilePath = resolve(resolvedWorkingDirectory, setup.lockfile);
  if (!statSync(lockfilePath).isFile()) invalid(`lockfile does not exist: ${setup.lockfile}`);
  if (setup.prepare != null) {
    if (
      !Array.isArray(setup.prepare) ||
      setup.prepare.length < 2 ||
      setup.prepare[0] !== setup.packageManager ||
      setup.prepare.some(
        (argument) =>
          typeof argument !== "string" || argument.length === 0 || argument.includes("\0"),
      )
    ) {
      invalid(`prepare must be a ${setup.packageManager} command argument array`);
    }
  }
  if (!Array.isArray(setup.requiredOutputs) || setup.requiredOutputs.length === 0) {
    invalid("requiredOutputs must be a non-empty array");
  }
  for (const output of setup.requiredOutputs) {
    requireNormalizedRelativePath(output, "requiredOutputs entry", invalid);
  }
  return { ...setup, workingDirectory };
}

function requireNormalizedRelativePath(
  value: unknown,
  label: string,
  invalid: (message: string) => never,
) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    isAbsolute(value) ||
    /^[A-Za-z]:[\\/]/.test(value) ||
    value.includes("\\") ||
    value.startsWith("./") ||
    value.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  ) {
    invalid(`${label} must be a normalized relative path`);
  }
}

async function runRecordedCommand({
  command,
  cwd,
  label,
  managerRunner,
  timeoutMs,
}: {
  command: string[];
  cwd: string;
  label: string;
  managerRunner: ReturnType<typeof packageManagerRunner>;
  timeoutMs: number;
}) {
  const startedAt = Date.now();
  const result = await runPackageManager(managerRunner, command.slice(1), {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      CI: "true",
      npm_config_ignore_scripts: "true",
      YARN_ENABLE_SCRIPTS: "false",
      NUXT_TELEMETRY_DISABLED: "1",
    },
    maxBuffer: 1024 * 1024 * 1024,
    timeout: timeoutMs,
  });
  if (result.error != null)
    throw new Error(`${label} failed to run: ${errorMessage(result.error)}`);
  if (result.status !== 0) throw new Error(`${label} exited with status ${result.status}`);
  return {
    command,
    durationMs: Date.now() - startedAt,
    exitCode: result.status,
    stdoutSha256: sha256(result.stdout ?? ""),
    stderrSha256: sha256(result.stderr ?? ""),
  };
}

function requireCleanFixture(fixtureRoot: string, phase: string) {
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

function sha256(value: string | NodeJS.ArrayBufferView) {
  return createHash("sha256").update(value).digest("hex");
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

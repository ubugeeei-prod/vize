import { existsSync } from "node:fs";

import type { PackagePath } from "./task-types.ts";
import { shellCommand } from "./task-shell.ts";

export const localVp = "./node_modules/.bin/vp";
export const vscodeExtensionPackageBin = (packageName: string, binName: string) =>
  `node ../../tools/vscode-vize/run-package-bin.mjs ${packageName} ${binName}`;

/**
 * Runs a command after changing into a package directory.
 *
 * The task catalog uses this for the few packages that must execute their own
 * package-manager scripts directly instead of going through `vp run --filter`.
 */
export const runInDirectory = (cwd: string, command: string) =>
  shellCommand(`cd ${cwd} && ${command}`);

export const runPackageScriptDirectly = (taskName: string, packages: readonly PackagePath[]) =>
  packages.map((pkg) => runInDirectory(pkg, `pnpm run ${taskName}`)).join(" && ");

/**
 * Ensures the VS Code extension package has the local binaries required by its
 * package-local tasks.
 */
export const installVscodeExtensionDependencies = runInDirectory(
  "editors/vscode",
  "if node ../../tools/vscode-vize/run-package-bin.mjs vite-plus vp --version >/dev/null 2>&1; then exit 0; fi && corepack pnpm install --ignore-workspace --lockfile-dir . --no-lockfile --prefer-offline --ignore-scripts",
);

/**
 * Runs one or more commands inside the VS Code extension package.
 *
 * The extension is intentionally isolated from the root workspace install, so
 * this helper performs a minimal package-local install before invoking tooling.
 * That keeps editor-extension tasks reproducible without making every root
 * install pay for VS Code extension dependencies.
 */
export const runInVscodeExtension = (...commands: string[]) =>
  `${installVscodeExtensionDependencies} && ${runInDirectory("editors/vscode", commands.join(" && "))}`;

/**
 * Builds a filtered `vp run` command for package groups.
 *
 * Package paths are typed as `./...` literals so task definitions cannot
 * accidentally target an absolute path or an unscoped shell fragment.
 */
export const runInPackages = (
  taskName: string,
  packages: readonly PackagePath[],
  options: {
    concurrencyLimit?: number;
  } = {},
) =>
  [
    ...(options.concurrencyLimit == null
      ? []
      : [`VP_RUN_CONCURRENCY_LIMIT=${options.concurrencyLimit}`]),
    "vp",
    "run",
    ...packages.map((pkg) => `--filter '${pkg}'`),
    taskName,
  ].join(" ");

export const runTask = (taskName: string) => `vp run --workspace-root ${taskName}`;
export const runTasks = (...taskNames: string[]) => taskNames.map(runTask).join(" && ");

const workspaceMoonHome = ".cache/moonbit";
const workspaceMoonBin = `${workspaceMoonHome}/bin/moon`;
const workspaceMoonRegistryIndex = `${workspaceMoonHome}/registry/index/.git`;
const moonToolsModule = "tools/moon";

export const moonCommandForEnvironment = (
  env: NodeJS.ProcessEnv = process.env,
  pathExists: (path: string) => boolean = existsSync,
) => {
  if (env.MOON_BIN != null && env.MOON_BIN !== "") {
    return env.MOON_BIN;
  }

  if (pathExists(workspaceMoonBin)) {
    return `env MOON_HOME=${workspaceMoonHome} MOON_BIN=${workspaceMoonBin} ${workspaceMoonBin}`;
  }

  return "moon";
};

export const moonRegistryUpdateGuardForEnvironment = (
  env: NodeJS.ProcessEnv = process.env,
  pathExists: (path: string) => boolean = existsSync,
) => {
  const hasExplicitMoonBin = env.MOON_BIN != null && env.MOON_BIN !== "";
  if (hasExplicitMoonBin || !pathExists(workspaceMoonBin)) {
    return null;
  }

  // The registry update runs from the repository root so the workspace-relative
  // MoonBit paths (MOON_HOME, MOON_BIN, and the moon binary itself) resolve. A
  // `cd` into the module directory would break those relative paths, which is
  // exactly what silently regressed the local `vp run release` path.
  return `( [ -d ${workspaceMoonRegistryIndex} ] || ${moonCommandForEnvironment(env, pathExists)} update )`;
};

export const moonRegistryRefreshCommandForEnvironment = (
  env: NodeJS.ProcessEnv = process.env,
  pathExists: (path: string) => boolean = existsSync,
) => `${moonCommandForEnvironment(env, pathExists)} update`;

const moonCommand = moonCommandForEnvironment();
const moonRegistryUpdateGuard = moonRegistryUpdateGuardForEnvironment();
const moonRegistryRefreshCommand = moonRegistryRefreshCommandForEnvironment();

const moonScriptCommand = (registrySetup: string | null, name: string, args: string[]) =>
  [
    ...(registrySetup == null ? [] : [registrySetup, "&&"]),
    moonCommand,
    "run",
    "-q",
    "--target",
    "native",
    `${moonToolsModule}/cmd/${name}`,
    "--",
    ...args,
  ].join(" ");

/**
 * Executes a repository MoonBit command package.
 *
 * The root task catalog treats MoonBit scripts as first-class automation. This
 * helper keeps the invocation uniform, prefers the workspace-local MoonBit
 * toolchain installed by the Nix shell, and forwards command arguments after
 * `--` so each package owns its own CLI parsing.
 */
export const moonScript = (name: string, ...args: string[]) =>
  moonScriptCommand(moonRegistryUpdateGuard, name, args);

/**
 * Executes a MoonBit command after refreshing the registry index.
 *
 * Release tasks use this slower path because resolving a newly pinned package
 * against an existing but stale index must fail before any release files are
 * changed. Regular development tasks keep the cheaper initialize-once path.
 */
export const moonScriptWithFreshRegistry = (name: string, ...args: string[]) =>
  moonScriptCommand(moonRegistryRefreshCommand, name, args);

export const devApp = (target?: string) =>
  target == null ? moonScript("dev_app") : moonScript("dev_app", target);

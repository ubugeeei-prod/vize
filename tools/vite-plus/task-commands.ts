import type { PackagePath } from "./task-types.ts";
import { shellQuote } from "./task-shell.ts";

export const localVp = "./node_modules/.bin/vp";

/**
 * Runs a command after changing into a package directory.
 *
 * The task catalog uses this for the few packages that must execute their own
 * package-manager scripts directly instead of going through `vp run --filter`.
 */
export const runInDirectory = (cwd: string, command: string) =>
  `sh -c ${shellQuote(`cd ${cwd} && ${command}`)}`;

export const runPackageScriptDirectly = (taskName: string, packages: readonly PackagePath[]) =>
  packages.map((pkg) => runInDirectory(pkg, `pnpm run ${taskName}`)).join(" && ");

/**
 * Ensures the VS Code extension package has the local binaries required by its
 * package-local tasks.
 */
export const installVscodeExtensionDependencies = runInDirectory(
  "npm/vscode-vize",
  "if [ -x node_modules/.bin/vp ]; then exit 0; fi && mkdir -p node_modules/.bin && pnpm install --ignore-workspace --no-lockfile --prefer-offline",
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
  `${installVscodeExtensionDependencies} && ${runInDirectory("npm/vscode-vize", commands.join(" && "))}`;

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

const moonCommand = process.env.MOON_BIN ?? "env -u MOON_HOME moon";

/**
 * Executes a repository MoonBit script through native script mode.
 *
 * The root task catalog treats MoonBit scripts as first-class automation. This
 * helper keeps the invocation uniform, clears inherited `MOON_HOME` by default,
 * and forwards script arguments after `--` so each script owns its own CLI
 * parsing.
 */
export const moonScript = (name: string, ...args: string[]) =>
  [
    moonCommand,
    "run",
    "-q",
    "--target",
    "native",
    "-",
    "--",
    ...args,
    "<",
    `tools/moon/scripts/${name}.mbtx`,
  ].join(" ");

export const devApp = (target?: string) =>
  target == null ? moonScript("dev_app") : moonScript("dev_app", target);

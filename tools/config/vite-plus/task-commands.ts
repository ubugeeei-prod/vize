import type { PackagePath } from "./task-types.ts";
import { shellCommand, shellQuote } from "./task-shell.ts";

export const localVp = "./node_modules/.bin/vp";

const rustScript = (scriptPath: string, args: readonly string[]) =>
  ["rust-script", scriptPath, ...args].map(shellQuote).join(" ");

export const vscodeExtensionPackageBin = (
  packageName: string,
  binName: string,
  ...args: string[]
) =>
  rustScript("../../tools/commands/editors/vscode/run-package-bin.rs", [
    packageName,
    binName,
    ...args,
  ]);

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
  `if ${vscodeExtensionPackageBin("vite-plus", "vp")} --version >/dev/null 2>&1; then exit 0; fi && corepack pnpm install --ignore-workspace --lockfile-dir . --no-lockfile --prefer-offline --ignore-scripts`,
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

export const rustTool = (command: string, ...args: string[]) =>
  rustScript(`tools/commands/${command}.rs`, args);

export const rustToolFromVscodeExtension = (command: string, ...args: string[]) =>
  rustScript(`../../tools/commands/${command}.rs`, args);

/** Runs a TypeScript command under `tools/support` with Node's type stripping. */
export const nodeTool = (command: string, ...args: string[]) =>
  ["node", `tools/support/${command}.ts`, ...args].map(shellQuote).join(" ");

export const devApp = (target?: string) =>
  target == null ? nodeTool("tasks/dev-app") : nodeTool("tasks/dev-app", target);

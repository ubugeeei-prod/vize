import { steps, type TaskCommand } from "./task-plan.ts";
import type { PackageCommand } from "../../support/tasks/package-task.ts";
import type { PackagePath } from "./task-types.ts";
import { shellQuote } from "./task-shell.ts";

export const localVp = "./node_modules/.bin/vp";

const rustScript = (scriptPath: string, args: readonly string[]) =>
  ["rust-script", scriptPath, ...args].map(shellQuote).join(" ");

export const vscodeExtensionPackageBin = (
  packageName: string,
  binName: string,
  ...args: string[]
) =>
  ["node", "../../tools/commands/editors/vscode/run-package-bin.ts", packageName, binName, ...args]
    .map(shellQuote)
    .join(" ");

/**
 * Runs a command after changing into a package directory.
 *
 * The task catalog uses this for the few packages that must execute their own
 * package-manager scripts directly instead of going through `vp run --filter`.
 */
export const runInDirectory = (directory: string, ...commands: TaskCommand[]): PackageCommand[] =>
  steps(...commands).map((command) => ({ ...command, directory }));

export const runPackageScriptDirectly = (
  taskName: string,
  packages: readonly PackagePath[],
): PackageCommand[] =>
  packages.map((directory) => ({ program: "pnpm", arguments: ["run", taskName], directory }));

export const installVscodeExtensionDependencies =
  "node tools/commands/editors/vscode/ensure-dependencies.ts";

export const runInVscodeExtension = (...commands: TaskCommand[]): PackageCommand[] =>
  steps(installVscodeExtensionDependencies, runInDirectory("editors/vscode", ...commands));

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
): PackageCommand => ({
  program: "vp",
  arguments: ["run", ...packages.flatMap((pkg) => ["--filter", pkg]), taskName],
  ...(options.concurrencyLimit == null
    ? {}
    : { environment: { VP_RUN_CONCURRENCY_LIMIT: String(options.concurrencyLimit) } }),
});

export const runTask = (taskName: string) => `vp run --workspace-root ${taskName}`;
export const runTasks = (...taskNames: string[]) => steps(...taskNames.map(runTask));

export const rustTool = (command: string, ...args: string[]) =>
  rustScript(`tools/commands/${command}.rs`, args);

export const rustToolFromVscodeExtension = (command: string, ...args: string[]) =>
  rustScript(`../../tools/commands/${command}.rs`, args);

/** Runs a TypeScript command from the isolated VS Code extension package. */
export const nodeCommandFromVscodeExtension = (command: string, ...args: string[]) =>
  ["node", `../../tools/commands/${command}.ts`, ...args].map(shellQuote).join(" ");

/** Runs a TypeScript command under `tools/commands` with stable type stripping. */
export const nodeCommand = (command: string, ...args: string[]) =>
  ["node", `tools/commands/${command}.ts`, ...args].map(shellQuote).join(" ");

/** Runs a TypeScript command under `tools/support` with Node's type stripping. */
export const nodeTool = (command: string, ...args: string[]) =>
  ["node", `tools/support/${command}.ts`, ...args].map(shellQuote).join(" ");

export const devApp = (target?: string) =>
  target == null ? nodeTool("tasks/dev-app") : nodeTool("tasks/dev-app", target);

export const stageVscodeTypeScriptPlugin = nodeCommandFromVscodeExtension(
  "editors/vscode/sync-typescript-plugin",
  "stage",
);
const injectVscodeTypeScriptPlugin = nodeCommandFromVscodeExtension(
  "editors/vscode/sync-typescript-plugin",
  "inject",
  "dist/vize.vsix",
);
export const assertVscodePackage = rustToolFromVscodeExtension(
  "editors/vscode/assert-vsix-package",
  "dist/vize.vsix",
);
export const packageVscodeExtension = steps(
  stageVscodeTypeScriptPlugin,
  `${vscodeExtensionPackageBin("@vscode/vsce", "vsce")} package --no-dependencies --out dist/vize.vsix`,
  injectVscodeTypeScriptPlugin,
);

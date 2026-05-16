import { execFileSync, spawnSync } from "node:child_process";
import type { Plugin } from "vite";
import type { UserConfig } from "vite-plus";

export type TaskMap = NonNullable<NonNullable<UserConfig["run"]>["tasks"]>;
export type TaskConfig = TaskMap[string];
export type CacheableTaskConfig = Extract<TaskConfig, { cache?: true }>;
export type TaskInput = NonNullable<CacheableTaskConfig["input"]>;
export type PackagePath = `./${string}`;

/**
 * Preserves the exact task object shape while letting TypeScript validate every
 * task against Vite+'s public configuration type.
 *
 * Keeping this helper small but explicit gives each task group precise literal
 * keys without falling back to a broad `Record<string, unknown>` style. That is
 * important for the root config because the task catalog is assembled from many
 * modules and should fail at compile time when Vite+ changes its task schema.
 */
export const defineTasks = <const T extends TaskMap>(tasks: T): T => tasks;

export const localVp = "./node_modules/.bin/vp";

/**
 * Quotes a command fragment for POSIX shell interpolation.
 *
 * Vite+ tasks are command strings, so a few helpers intentionally compose shell
 * snippets. This function keeps that composition centralized and prevents
 * environment-derived paths from breaking commands that are later wrapped in
 * `sh -c`.
 */
const shellQuote = (command: string) => `'${command.replaceAll("'", `'"'"'`)}'`;

const darwinLibiconvLibraryPath = process.env.VIZE_DARWIN_LIBICONV_LIB;
const rustTaskEnvironment =
  darwinLibiconvLibraryPath == null
    ? []
    : [
        `export LIBRARY_PATH=${shellQuote(darwinLibiconvLibraryPath)}\${LIBRARY_PATH:+:$LIBRARY_PATH}`,
        `export RUSTFLAGS=${shellQuote(`-L native=${darwinLibiconvLibraryPath}`)}\${RUSTFLAGS:+ $RUSTFLAGS}`,
      ];

/**
 * Applies the optional macOS libiconv environment to Rust-oriented task
 * commands.
 *
 * The environment is injected only when explicitly requested so regular Linux
 * CI and developer machines keep the shortest possible command path. When the
 * variable is present, both Cargo and any nested Rust build script see the same
 * library search path.
 */
const withRustTaskEnvironment = (command: string) =>
  rustTaskEnvironment.length === 0
    ? command
    : `sh -c ${shellQuote(`${rustTaskEnvironment.join("; ")}; ${command}`)}`;

/**
 * Creates a cacheable Vite+ task while keeping Rust-specific environment
 * handling transparent to the task catalog.
 */
export const task = (
  command: string,
  options: {
    input?: TaskInput;
  } = {},
): TaskConfig => ({
  command: withRustTaskEnvironment(command),
  ...options,
});

/**
 * Creates an uncached task for commands whose effects are too broad or too
 * stateful to be represented by a stable input list.
 */
export const noCacheTask = (command: string): TaskConfig => ({
  cache: false as const,
  command: withRustTaskEnvironment(command),
});

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

const commandExists = (command: string) =>
  spawnSync("sh", ["-c", `command -v ${command}`], { stdio: "ignore" }).status === 0;

/**
 * Builds the root library artifact by delegating to the workspace build task.
 *
 * The root Vite build exists only as a stable Vite+ entry point for task
 * orchestration. The actual production artifacts still come from the workspace
 * build task, and this plugin chooses `nix develop` only when the local machine
 * needs Nix to provide missing native tools.
 */
export const rootBuildTaskPlugin = (): Plugin => ({
  name: "vize-root-build-task",
  apply: "build",
  closeBundle() {
    if (process.env.VIZE_SKIP_ROOT_BUILD_TASK === "1") {
      return;
    }

    const buildCommand = ["vp", "run", "--workspace-root", "build"];
    const command = commandExists("wasm-pack") || !commandExists("nix") ? "vp" : "nix";
    const args =
      command === "vp"
        ? buildCommand.slice(1)
        : ["--option", "warn-dirty", "false", "develop", "--command", ...buildCommand];

    execFileSync(command, args, {
      env: {
        ...process.env,
        VIZE_SKIP_ROOT_BUILD_TASK: "1",
      },
      stdio: "inherit",
    });
  },
});

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

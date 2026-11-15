import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import type { UserConfig } from "vite-plus";
import type { VizePlusOptions, VizeTask } from "./types.ts";

const tasks: VizeTask[] = [
  "editor:setup",
  "check",
  "typecheck",
  "lint",
  "lint:fix",
  "fmt",
  "fmt:check",
  "build",
  "pack",
  "dev",
  "preview",
  "test",
];
type Tasks = NonNullable<NonNullable<UserConfig["run"]>["tasks"]>;
type LockfileInput = string | { pattern: string; base: "workspace" };
type PackageManifest = {
  packageManager?: string;
  scripts?: Record<string, string>;
  workspaces?: unknown;
};
const readOnlyTasks = new Set<VizeTask>(["lint", "fmt:check", "check", "typecheck"]);
const lockfilesByPackageManager = {
  bun: ["bun.lock", "bun.lockb"],
  npm: ["npm-shrinkwrap.json", "package-lock.json"],
  pnpm: ["pnpm-lock.yaml"],
  yarn: ["yarn.lock"],
} as const;
const allLockfiles = Object.values(lockfilesByPackageManager).flat();

export function createTasks(
  existing: Tasks = {},
  names: VizePlusOptions["tasks"] = {},
  taskCache: VizePlusOptions["taskCache"] = false,
): Tasks {
  if (names === false) return {};
  // Package export resolution survives pack's shared chunks and nested installs.
  const runner = fileURLToPath(import.meta.resolve("@vizejs/vite-plugin/internal/vite-plus-task"));
  let scripts: Record<string, string> = {};
  let packageManager: string | undefined;
  try {
    const manifest = readManifest(process.cwd());
    scripts = manifest.scripts ?? {};
    packageManager = manifest.packageManager;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  const result: Tasks = {};
  let lockfiles: LockfileInput[] | undefined;
  for (const task of tasks) {
    if (names[task] === false) continue;
    const name = names[task] ?? (task in scripts ? `vize:${task}` : task);
    if (name in existing) continue;
    if (name in scripts || name in result) {
      throw new Error(
        `defineConfig task "${name}" already exists. Choose another name with options.tasks.`,
      );
    }
    // Only the package-owned absolute path enters the task shell command.
    const command = `node ${quote(runner)} ${task}`;
    result[name] =
      taskCache && readOnlyTasks.has(task)
        ? cachedTask(command, (lockfiles ??= lockfileInputs(packageManager)))
        : { command, cache: false };
  }
  return result;
}

function lockfileInputs(packageManager?: string): LockfileInput[] {
  const packageDirectory = process.cwd();
  const local = allLockfiles.filter((file) => existsSync(path.join(packageDirectory, file)));
  const workspaceDirectory = findWorkspaceDirectory(packageDirectory);
  const workspace = workspaceDirectory === packageDirectory ? local : [];
  const useWorkspace = workspaceDirectory !== packageDirectory && !local.length;
  if (useWorkspace) {
    workspace.push(
      ...allLockfiles.filter((file) => existsSync(path.join(workspaceDirectory, file))),
    );
  }
  const selectedPackageManager = useWorkspace
    ? (readManifest(workspaceDirectory).packageManager ?? packageManager)
    : packageManager;
  const declared = selectedPackageManager?.split("@", 1)[0];
  let selected: string[];
  if (declared && declared in lockfilesByPackageManager) {
    selected = [...lockfilesByPackageManager[declared as keyof typeof lockfilesByPackageManager]];
  } else {
    const present = useWorkspace ? workspace : local;
    // If installation has not created a lockfile yet, retain every supported
    // name so the first one becomes an input without a config change.
    selected = present.length > 0 ? present : [...allLockfiles];
  }
  return useWorkspace ? selected.map((pattern) => ({ pattern, base: "workspace" })) : selected;
}

function findWorkspaceDirectory(packageDirectory: string): string {
  let directory = packageDirectory;
  for (;;) {
    if (existsSync(path.join(directory, "pnpm-workspace.yaml"))) return directory;
    if (existsSync(path.join(directory, "package.json"))) {
      try {
        if (readManifest(directory).workspaces != null) return directory;
      } catch (error) {
        if (directory === packageDirectory) throw error;
      }
    }
    const parent = path.dirname(directory);
    if (parent === directory) return packageDirectory;
    directory = parent;
  }
}

function readManifest(directory: string): PackageManifest {
  return JSON.parse(readFileSync(path.join(directory, "package.json"), "utf8"));
}

function cachedTask(command: string, lockfiles: LockfileInput[]): Tasks[string] {
  const require = createRequire(path.resolve("package.json"));
  const { version } = JSON.parse(readFileSync(require.resolve("vite-plus/package.json"), "utf8"));
  const cache = {
    // Installed dependency contents belong to the selected package manager's
    // lockfile. Keep automatic source/config discovery without fingerprinting
    // package-manager and tool caches under any workspace's node_modules.
    input: [
      { auto: true },
      "!**/node_modules",
      "!**/node_modules/**",
      "!**/.vize/**",
      ...lockfiles,
    ],
    // Read-only checks have no outputs to restore. In particular, forwarding
    // --fix must never make a later cache hit overwrite authored source.
    output: [],
  };
  // Vite+ moved input/output inside cache in 1.0.0-rc.1. Select the
  // consumer's API rather than the Vite+ version used to build this package.
  const nested = Number(version.split(".")[0]) >= 1 && version !== "1.0.0-rc.0";
  const task = nested ? { command, cache } : { command, cache: true, ...cache };
  // The optional peer supplies only one of these two incompatible Task types
  // at build time; both provider shapes are covered by installed-runner tests.
  return task as Tasks[string];
}

function quote(value: string): string {
  if (process.platform === "win32") {
    if (/["%\r\n]/u.test(value)) throw new Error("Unsupported character in Vize installation path");
    return `"${value}"`;
  }
  return `'${value.replaceAll("'", "'\\''")}'`;
}

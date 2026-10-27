import { readFileSync } from "node:fs";
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
const readOnlyTasks = new Set<VizeTask>(["lint", "fmt:check", "check", "typecheck"]);

export function createTasks(
  existing: Tasks = {},
  names: VizePlusOptions["tasks"] = {},
  taskCache: VizePlusOptions["taskCache"] = false,
): Tasks {
  if (names === false) return {};
  // Package export resolution survives pack's shared chunks and nested installs.
  const runner = fileURLToPath(import.meta.resolve("@vizejs/vite-plugin/internal/vite-plus-task"));
  let scripts: Record<string, string> = {};
  try {
    scripts = JSON.parse(readFileSync(path.resolve("package.json"), "utf8")).scripts ?? {};
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }
  const result: Tasks = {};
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
      taskCache && readOnlyTasks.has(task) ? cachedTask(command) : { command, cache: false };
  }
  return result;
}

function cachedTask(command: string): Tasks[string] {
  const require = createRequire(path.resolve("package.json"));
  const { version } = JSON.parse(readFileSync(require.resolve("vite-plus/package.json"), "utf8"));
  const cache = {
    // Keep automatic tracking of source, imported config, dependencies and new
    // directory entries; only exclude the native tools' generated work files.
    input: [{ auto: true }, "!node_modules/.vize/**", "!**/.vize/**"],
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

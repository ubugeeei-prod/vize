import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { UserConfig } from "vite-plus";
import type { VizePlusOptions, VizeTask } from "./types.ts";

const tasks: VizeTask[] = [
  "check",
  "lint",
  "lint:fix",
  "fmt",
  "fmt:check",
  "build",
  "dev",
  "preview",
  "test",
];
type Tasks = NonNullable<NonNullable<UserConfig["run"]>["tasks"]>;

export function createTasks(existing: Tasks = {}, names: VizePlusOptions["tasks"] = {}): Tasks {
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
        `withVue task "${name}" already exists. Choose another name with options.tasks.`,
      );
    }
    // Only the package-owned absolute path enters the task shell command.
    result[name] = { command: `node ${quote(runner)} ${task}`, cache: false };
  }
  return result;
}

function quote(value: string): string {
  if (process.platform === "win32") {
    if (/["%\r\n]/u.test(value)) throw new Error("Unsupported character in Vize installation path");
    return `"${value}"`;
  }
  return `'${value.replaceAll("'", "'\\''")}'`;
}

import { readFileSync } from "node:fs";
import path from "node:path";
import { build } from "vite";
import { type ConfigValue } from "./repository-yaml.ts";
import { isProgramInvocation, isProgramCommand } from "./automation-commands.ts";

export async function packageTaskRuntime(root: string): Promise<string> {
  const result = await build({
    root,
    configFile: false,
    logLevel: "silent",
    build: {
      target: "node18",
      write: false,
      minify: false,
      reportCompressedSize: false,
      lib: { entry: path.join(root, "tools/support/tasks/package-task.ts"), formats: ["cjs"] },
      rolldownOptions: {
        platform: "node",
        external: [/^node:/u],
        output: { comments: { legal: true, annotation: false, jsdoc: false } },
      },
    },
  });
  const [bundle] = Array.isArray(result) ? result : [result];
  if (!bundle || !("output" in bundle)) throw new Error("Missing package task runtime");
  const [entry] = bundle.output;
  if (entry?.type !== "chunk") throw new Error("Missing package task entrypoint");
  return entry.code.trimEnd();
}

export function validatePackageScripts(
  root: string,
  directory: string,
  taskNames: readonly string[],
): void {
  const { scripts = {} } = JSON.parse(
    readFileSync(path.join(root, directory, "package.json"), "utf8"),
  ) as { scripts?: Record<string, string> };
  for (const name of taskNames) {
    if (!/^[\w:-]+$/u.test(name) || scripts[name] !== "node ./scripts/package-tasks.cjs " + name)
      throw new Error(directory + ": missing package script for " + name);
  }
  for (const [name, command] of Object.entries(scripts)) {
    if (!isProgramInvocation(command))
      throw new Error(directory + ": move " + name + " into the package's Pkl task definition");
    if (command.startsWith("node ./scripts/package-tasks.cjs ") && !taskNames.includes(name))
      throw new Error(directory + ": undefined package task " + name);
  }
}

export function renderPackageTasks(
  root: string,
  target: string,
  tasks: ConfigValue,
  runtime: string,
): string {
  if (!tasks || Array.isArray(tasks) || typeof tasks !== "object")
    throw new Error("Invalid package tasks for " + target);
  const directory = path.dirname(path.dirname(target));
  for (const [name, commands] of Object.entries(tasks)) {
    if (!Array.isArray(commands) || commands.length === 0)
      throw new Error("Empty package task: " + name);
    for (const command of commands) {
      if (
        !command ||
        typeof command !== "object" ||
        Array.isArray(command) ||
        typeof command.program !== "string" ||
        !Array.isArray(command.arguments) ||
        !command.arguments.every((arg) => typeof arg === "string") ||
        !isProgramCommand([command.program, ...command.arguments])
      )
        throw new Error(directory + " / " + name + ": expected a named program");
    }
  }
  validatePackageScripts(root, directory, Object.keys(tasks));
  return (
    "// Generated from tools/config/pkl/package-tasks/" +
    directory +
    "/tasks.pkl\n" +
    runtime +
    "\n\nconst tasks = " +
    JSON.stringify(
      tasks,
      (_key, value: unknown) => {
        if (value === null) return undefined;
        if (value && typeof value === "object" && Object.keys(value).length === 0) return undefined;
        return value;
      },
      2,
    ) +
    ";\nexports.tasks = tasks;\nif (require.main === module) process.exitCode = exports.runPackageTask(tasks, process.argv.slice(2));\n"
  );
}

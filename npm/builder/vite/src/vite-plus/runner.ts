import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { glob } from "tinyglobby";
import type { ConfigWithVizeTasks, VizeTask, VizeTaskConfig } from "./types.ts";
import { taskConfigKey } from "./types.ts";
import { loadConfig, resolveConfigExport } from "../config.ts";
import { resolveVitePlus } from "./runtime.ts";

export type Execute = (command: string, args: string[]) => Promise<number>;

/** Run every selected checker, even when an earlier tool reports diagnostics. */
export async function runTools(
  task: VizeTask,
  args: string[],
  metadata: VizeTaskConfig,
  vp: string,
  native: string,
  execute: Execute,
): Promise<number> {
  const { options } = metadata;
  const lint = task === "check" || task === "lint" || task === "lint:fix";
  const fmt = task === "check" || task === "fmt" || task === "fmt:check";
  const fix = task === "lint:fix" || args.includes("--fix");
  const formatCheck =
    task === "fmt:check" || (task === "check" && !fix) || args.includes("--check");
  if (task === "check" && args.some((arg) => arg.startsWith("-") && arg !== "--fix")) {
    throw new Error("Vize check tasks accept paths and --fix. Put tool options in withVize().");
  }
  let status = 0;
  async function run(binary: string, argv: string[]) {
    const code = await execute(process.execPath, [binary, ...argv]);
    if (code >= 128) throw Object.assign(new Error("Vize task interrupted"), { exitCode: code });
    status = Math.max(status, code);
  }
  async function vize(command: "check" | "lint" | "fmt", argv: string[]) {
    const env = { mode: "production", command };
    const config =
      metadata.config === undefined
        ? ((await loadConfig(process.cwd(), { env })) ?? {})
        : await resolveConfigExport(metadata.config, env);
    // Native relative paths and scoped entries are relative to the config's
    // directory. Keep the temporary file beside the project's config, not in /tmp.
    const file = path.resolve(`.vize-vp-${randomUUID()}.json`);
    try {
      await writeFile(file, JSON.stringify(config), { flag: "wx", mode: 0o600 });
      await run(native, [command, "--config", file, ...argv]);
    } finally {
      await rm(file, { force: true });
    }
  }
  if (task === "check" && options.check !== false)
    await vize(
      "check",
      args.filter((arg) => arg !== "--fix"),
    );
  if (lint) {
    const lintArgs = fix && !args.includes("--fix") ? ["--fix", ...args] : args;
    if (options.lint !== false) await vize("lint", lintArgs);
    await run(vp, ["lint", ...lintArgs]);
  }
  if (fmt) {
    const patterns = args.filter((arg) => !["--check", "--write", "-w", "--fix"].includes(arg));
    if (patterns.some((arg) => arg.startsWith("-"))) {
      throw new Error(
        "Vize fmt tasks accept paths and --check/--write. Put formatter options in withVize().",
      );
    }
    if (options.fmt !== false) {
      const files = await glob(patterns.length ? patterns : ["**/*.vue"], {
        expandDirectories: true,
        ignore: ["**/node_modules/**", "**/.git/**"],
      });
      const vue = files.filter((file) => file.endsWith(".vue"));
      // An empty pattern list would make native fmt format every file again.
      if (vue.length) await vize("fmt", [formatCheck ? "--check" : "--write", ...vue]);
    }
    await run(vp, ["fmt", formatCheck ? "--check" : "--write", ...patterns]);
  }
  return status;
}

export async function runTask(argv: string[]): Promise<number> {
  const [task, ...rest] = argv;
  const args = rest[0] === "--" ? rest.slice(1) : rest;
  const { require, binary: vp } = resolveVitePlus();
  if (["build", "dev", "preview", "test"].includes(task)) {
    return execute(process.execPath, [vp, task, ...args]);
  }
  if (!["check", "lint", "lint:fix", "fmt", "fmt:check"].includes(task)) {
    throw new Error(`Unknown Vize task: ${task}`);
  }
  const { loadConfigFromFile } = require("vite-plus") as typeof import("vite-plus");
  const loaded = await loadConfigFromFile({ command: "build", mode: "production" });
  const metadata = (loaded?.config as ConfigWithVizeTasks | undefined)?.[taskConfigKey];
  if (!metadata)
    throw new Error("The current vite.config must export withVize() or withVize().vp(...).");
  const vizeRequire = createRequire(import.meta.url);
  const native = path.resolve(path.dirname(vizeRequire.resolve("vize")), "../bin/vize");
  return runTools(task as VizeTask, args, metadata, vp, native, execute);
}

async function execute(command: string, args: string[]): Promise<number> {
  return new Promise((resolve) => {
    const child = spawn(command, args, { stdio: "inherit" });
    const forwardInt = () => child.kill("SIGINT");
    const forwardTerm = () => child.kill("SIGTERM");
    process.on("SIGINT", forwardInt);
    process.on("SIGTERM", forwardTerm);
    const cleanup = () => {
      process.off("SIGINT", forwardInt);
      process.off("SIGTERM", forwardTerm);
    };
    child.once("error", (error) => {
      cleanup();
      console.error(error.message);
      resolve(1);
    });
    child.once("exit", (code, signal) => {
      cleanup();
      resolve(code ?? (signal === "SIGINT" ? 130 : signal === "SIGTERM" ? 143 : 1));
    });
  });
}

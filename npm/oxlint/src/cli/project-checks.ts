import fs from "node:fs";
import path from "node:path";
import { randomUUID } from "node:crypto";

import type { ScopedConfig } from "./scoped-config.ts";

type Config = Record<string, unknown>;
const isRecord = (value: unknown): value is Config =>
  typeof value === "object" && value != null && !Array.isArray(value);
const isVize = (name: string) => name.startsWith("vize/");

export function needsOriginalProjectChecks(config: ScopedConfig, args: readonly string[]): boolean {
  const hasProject = (value: Config): boolean =>
    (Array.isArray(value.plugins) && value.plugins.includes("import")) ||
    (isRecord(value.options) && Boolean(value.options.typeAware || value.options.typeCheck)) ||
    (Array.isArray(value.extends) &&
      value.extends.some((parent) => isRecord(parent) && hasProject(parent))) ||
    (Array.isArray(value.overrides) &&
      value.overrides.some((row) => isRecord(row) && hasProject(row)));
  return (
    hasProject(config.value) ||
    args.some(
      (arg) =>
        ["--import-plugin", "--type-aware", "--type-check", "--tsconfig"].includes(arg) ||
        arg.startsWith("--tsconfig="),
    )
  );
}

/** Native/project checks retain original paths; only Vize executes on the bridge. */
export function splitProjectConfig(config: ScopedConfig) {
  const visit = (value: Config, bridge: boolean, override = false): Config => ({
    ...value,
    ...(Array.isArray(value.extends)
      ? { extends: value.extends.map((parent) => visit(parent as Config, bridge)) }
      : {}),
    ...(Array.isArray(value.overrides)
      ? { overrides: value.overrides.map((row) => visit(row as Config, bridge, true)) }
      : {}),
    rules: Object.fromEntries(
      Object.entries(isRecord(value.rules) ? value.rules : {}).flatMap(([name, setting]) =>
        bridge ? (isVize(name) ? [[name, setting]] : []) : [[name, isVize(name) ? "off" : setting]],
      ),
    ),
    ...(bridge
      ? {
          plugins: [],
          ...(!override
            ? {
                categories: Object.fromEntries(
                  [
                    "correctness",
                    ...Object.keys(isRecord(value.categories) ? value.categories : {}),
                  ].map((category) => [category, "off"]),
                ),
              }
            : {}),
          ...(isRecord(value.options)
            ? { options: { ...value.options, typeAware: false, typeCheck: false } }
            : {}),
        }
      : {}),
  });
  return {
    original: { ...config, value: visit(config.value, false) },
    bridge: { ...config, value: visit(config.value, true) },
  };
}

export function replaceConfigArgs(options: readonly string[], file: string): string[] {
  const args = [...options];
  let replaced = false;
  for (let index = 0; index < args.length && args[index] !== "--"; index += 1) {
    if (args[index] === "-c" || args[index] === "--config") {
      args[++index] = file;
      replaced = true;
    } else if (args[index].startsWith("--config=")) {
      args[index] = `--config=${file}`;
      replaced = true;
    }
  }
  if (!replaced) args.unshift("--config", file);
  return args;
}

export function bridgeProjectArgs(options: readonly string[]): string[] {
  if (
    options.some(
      (arg) =>
        [
          "-A",
          "-D",
          "-W",
          "--allow",
          "--deny",
          "--warn",
          "--max-warnings",
          "--type-check-only",
          "--report-unused-disable-directives",
          "--report-unused-disable-directives-severity",
          "--fix",
          "--fix-suggestions",
          "--fix-dangerously",
        ].includes(arg) ||
        /^(?:--allow|--deny|--warn|--max-warnings|--report-unused-disable-directives-severity)=/u.test(
          arg,
        ),
    )
  )
    throw new Error(
      "Original-path project checks require rule severity in the config; remove CLI severity/limit overrides and fix flags, and unused-disable reporting.",
    );
  const args: string[] = [];
  for (let index = 0; index < options.length; index += 1) {
    const arg = options[index];
    if (arg === "--tsconfig") index += 1;
    else if (
      !["--import-plugin", "--type-aware", "--type-check"].includes(arg) &&
      !arg.startsWith("--tsconfig=")
    )
      args.push(arg);
  }
  return args;
}

export function writeOriginalProjectConfig(config: ScopedConfig) {
  const file = path.join(
    path.dirname(config.file),
    `oxlint-vize-original-${randomUUID()}.${config.module ? "mts" : "json"}`,
  );
  fs.writeFileSync(
    file,
    `${config.module ? "export default " : ""}${JSON.stringify(config.value)}${config.module ? ";" : ""}\n`,
    { flag: "wx" },
  );
  return { file, cleanup: () => fs.unlinkSync(file) };
}

/**
 * Runs oxlint's type-aware rules on Vue SFC script blocks (#8937).
 *
 * oxlint's type-aware engine (tsgolint) never sees `.vue` files, so rules such
 * as `typescript/no-floating-promises` stay silent inside `<script setup>`.
 * After the normal run, this pass writes one position-preserving script shim
 * next to each SFC (`Foo.vue.ts`: every character outside the script blocks
 * is blanked, newlines are kept), lints only those shims with a configuration
 * that enables nothing but the user's type-aware rules, and rewrites the shim
 * paths back to the `.vue` files. Lines and columns already match the SFC, and
 * `import Foo from "./Foo.vue"` resolves to the shim the way TypeScript
 * resolves any extensionless sibling, so imported types flow between files.
 * Template expressions stay with Vize's own `type/*` rules.
 */
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { isStandaloneHtmlFile } from "../file-kinds.ts";
import { extractSfcBlocks } from "../sfc-blocks.ts";
import { withoutLintTargets } from "./args.ts";
import { rewriteReportedPaths } from "./output.ts";
import type { OxlintProcessResult } from "./scoped-selection.ts";
import { registerPathReplacementVariants } from "./workaround-files.ts";

type Run = (args: readonly string[]) => Promise<OxlintProcessResult>;

const CATEGORIES = [
  "correctness",
  "suspicious",
  "pedantic",
  "perf",
  "style",
  "restriction",
  "nursery",
];
/** Options that would re-enable or re-select rules, or change what is linted. */
const DROPPED_WITH_VALUE = new Set([
  "-A",
  "-D",
  "-W",
  "-c",
  "--allow",
  "--deny",
  "--warn",
  "--config",
  "--ignore-path",
  "--ignore-pattern",
  "--max-warnings",
]);
const DROPPED_FLAGS = new Set([
  "--fix",
  "--fix-suggestions",
  "--fix-dangerously",
  "--print-config",
  "--rules",
  "--type-aware",
  "--type-check",
  "--type-check-only",
  "--suppress-all",
  "--prune-suppressions",
]);

export interface ScriptShim {
  readonly text: string;
  readonly extension: ".ts" | ".tsx" | ".js" | ".jsx";
}

/** Whether the run asks oxlint for type-aware linting (flag or `options.typeAware`). */
export function requestsTypeAware(args: readonly string[], cwd: string): boolean {
  const end = args.includes("--") ? args.indexOf("--") : args.length;
  const options = args.slice(0, end);
  if (options.includes("--type-aware")) return true;
  let configPath = path.join(cwd, ".oxlintrc.json");
  for (let index = 0; index < options.length; index += 1) {
    if (options[index] === "-c" || options[index] === "--config")
      configPath = path.resolve(cwd, options[index + 1] ?? "");
    else if (options[index].startsWith("--config="))
      configPath = path.resolve(cwd, options[index].slice("--config=".length));
  }
  if (!configPath.endsWith(".json") || !fs.existsSync(configPath)) return false;
  try {
    const config = JSON.parse(stripJsonComments(fs.readFileSync(configPath, "utf8"))) as {
      options?: { typeAware?: unknown };
    };
    return config.options?.typeAware === true;
  } catch {
    return false;
  }
}

/** Blank everything outside the script blocks, keeping every offset in place. */
export function buildScriptShim(source: string): ScriptShim | null {
  const lineStarts = [0];
  for (let index = 0; index < source.length; index += 1)
    if (source.charCodeAt(index) === 10) lineStarts.push(index + 1);
  const chars = source.replace(/[^\r\n]/gu, " ").split("");
  let lang = "";
  let found = false;
  for (const block of extractSfcBlocks(source)) {
    if (block.kind !== "script" && block.kind !== "script-setup") continue;
    const start = lineStarts[block.contentStart.line - 1] + block.contentStart.column - 1;
    const openTag = source.slice(source.lastIndexOf("<script", start), start);
    if (/\bsrc\s*=/u.test(openTag)) continue;
    const blockLang = /\blang\s*=\s*["']?([a-z]+)/iu.exec(openTag)?.[1]?.toLowerCase() ?? "js";
    if (lang === "" || blockLang.startsWith("t")) lang = blockLang;
    for (let offset = 0; offset < block.content.length; offset += 1)
      chars[start + offset] = block.content[offset];
    found = true;
  }
  if (!found) return null;
  const extension =
    lang === "tsx" ? ".tsx" : lang === "ts" ? ".ts" : lang === "jsx" ? ".jsx" : ".js";
  return { text: chars.join(""), extension };
}

/** The user's resolved config, reduced to its enabled type-aware rules. */
export function typeAwareOnlyConfig(
  printed: Record<string, unknown>,
  typeAwareRules: ReadonlySet<string>,
): Record<string, unknown> | null {
  const rules: Record<string, unknown> = {};
  for (const [name, value] of Object.entries((printed.rules ?? {}) as Record<string, unknown>)) {
    const severity = Array.isArray(value) ? value[0] : value;
    if (!typeAwareRules.has(name) || severity === "off" || severity === "allow" || severity === 0)
      continue;
    rules[name] = value;
  }
  if (Object.keys(rules).length === 0) return null;
  return {
    plugins: printed.plugins ?? [],
    categories: Object.fromEntries(CATEGORIES.map((category) => [category, "off"])),
    rules,
    settings: printed.settings ?? {},
    env: printed.env ?? {},
    globals: printed.globals ?? {},
  };
}

/** Lint the SFC script shims with the user's type-aware rules; undefined when nothing applies. */
export async function runVueTypeAwarePass(
  cwd: string,
  entrypoint: string,
  forwardedArgs: readonly string[],
  vueFiles: readonly string[],
  run: Run,
): Promise<OxlintProcessResult | undefined> {
  const sfcFiles = vueFiles.filter((file) => file.endsWith(".vue") && !isStandaloneHtmlFile(file));
  if (sfcFiles.length === 0) return undefined;
  const options = withoutLintTargets(forwardedArgs);
  const configOptions = keepConfigSelection(options);
  const ruleList = await run([entrypoint, "--rules", "--format", "json"]);
  const printed = await run([entrypoint, ...configOptions, "--print-config"]);
  if (ruleList.status !== 0 || printed.status !== 0)
    throw new Error(`oxlint could not list its rules or print the config: ${printed.stderr}`);
  const typeAwareRules = new Set(
    (JSON.parse(ruleList.stdout) as { scope: string; value: string; type_aware?: boolean }[])
      .filter((rule) => rule.type_aware === true)
      .map((rule) => `${rule.scope}/${rule.value}`),
  );
  const config = typeAwareOnlyConfig(
    JSON.parse(printed.stdout) as Record<string, unknown>,
    typeAwareRules,
  );
  if (config == null) return undefined;

  const created: string[] = [];
  const replacements = new Map<string, string>();
  const configDir = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-vize-type-aware-"));
  const cleanup = () => {
    for (const file of created) fs.rmSync(file, { force: true });
    fs.rmSync(configDir, { force: true, recursive: true });
  };
  process.once("exit", cleanup);
  try {
    for (const file of sfcFiles) {
      const shim = buildScriptShim(fs.readFileSync(file, "utf8"));
      if (shim == null) continue;
      const shimPath = `${file}${shim.extension}`;
      if (fs.existsSync(shimPath)) continue;
      fs.writeFileSync(shimPath, shim.text, { flag: "wx" });
      created.push(shimPath);
      registerPathReplacementVariants(replacements, cwd, shimPath, file);
    }
    if (created.length === 0) return undefined;
    const configPath = path.join(configDir, ".oxlintrc.json");
    fs.writeFileSync(configPath, JSON.stringify(config));
    const result = await run([
      entrypoint,
      ...passOptions(options),
      "-c",
      configPath,
      "--type-aware",
      ...created,
    ]);
    const stdout = rewriteReportedPaths(result.stdout, replacements);
    const stderr = rewriteReportedPaths(result.stderr, replacements);
    return { ...result, stdout, stderr, rawStdout: undefined, rawStderr: undefined };
  } finally {
    cleanup();
    process.removeListener("exit", cleanup);
  }
}

function keepConfigSelection(options: readonly string[]): string[] {
  const kept: string[] = [];
  for (let index = 0; index < options.length; index += 1) {
    const option = options[index];
    if (option === "-c" || option === "--config" || option === "--tsconfig")
      kept.push(option, options[++index]);
    else if (option.startsWith("--config=") || option.startsWith("--tsconfig=")) kept.push(option);
    else if (option === "--disable-nested-config") kept.push(option);
  }
  return kept;
}

function passOptions(options: readonly string[]): string[] {
  const kept: string[] = [];
  for (let index = 0; index < options.length; index += 1) {
    const option = options[index];
    if (option === "--") break;
    const name = option.includes("=") ? option.slice(0, option.indexOf("=")) : option;
    if (DROPPED_WITH_VALUE.has(name)) {
      if (!option.includes("=")) index += 1;
      continue;
    }
    if (DROPPED_FLAGS.has(name)) continue;
    kept.push(option);
  }
  return kept;
}

function stripJsonComments(text: string): string {
  let result = "";
  let inString = false;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    if (inString) {
      result += char;
      if (char === "\\") result += text[++index] ?? "";
      else if (char === '"') inString = false;
    } else if (char === '"') {
      inString = true;
      result += char;
    } else if (char === "/" && text[index + 1] === "/") {
      while (index < text.length && text[index] !== "\n") index += 1;
      result += "\n";
    } else if (char === "/" && text[index + 1] === "*") {
      index = text.indexOf("*/", index + 2);
      if (index === -1) break;
      index += 1;
    } else result += char;
  }
  return result;
}

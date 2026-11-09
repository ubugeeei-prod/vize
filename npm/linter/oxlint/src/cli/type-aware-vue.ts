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

/** Where the pass configuration is evaluated: the user's config directory and each SFC's shim. */
export interface ShimContext {
  /** Directory override globs are relative to (the directory of the user's config). */
  readonly baseDir: string;
  /** Absolute path of each linted `.vue` file mapped to the absolute path of its script shim. */
  readonly shims: ReadonlyMap<string, string>;
}

/** Escapes glob metacharacters so a literal relative path can be used as an override glob. */
function literalGlob(relativePath: string): string {
  return relativePath.replace(/[\\*?[\]{}()!]/gu, "\\$&");
}

/** Whether a config glob matches `relativePath`; slash-less globs also match the basename. */
function matchesConfigGlob(relativePath: string, glob: string): boolean {
  if (typeof path.matchesGlob !== "function") return false;
  return (
    path.matchesGlob(relativePath, glob) ||
    (!glob.includes("/") && path.matchesGlob(path.basename(relativePath), glob))
  );
}

function stringList(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string")
    : [];
}

/**
 * Per-file overrides of the user's config, reduced to the type-aware rules they
 * set. Unlike the root rules, `off`/`allow` entries are kept: they are exactly
 * the carve-outs (for example "no floating promises in tests") that must still
 * apply to the shim of a matching `.vue` file.
 *
 * Globs are evaluated here against the original `.vue` path, relative to the
 * user's config directory, and each override is re-targeted to the exact shims
 * of the files it matched. Rewriting the globs instead would let a TypeScript
 * override such as `**\/*.ts` leak onto `Foo.vue.ts`, and would lose
 * `excludeFiles`.
 */
function typeAwareOverrides(
  overrides: unknown,
  typeAwareRules: ReadonlySet<string>,
  context: ShimContext | undefined,
): Record<string, unknown>[] {
  if (!Array.isArray(overrides) || context == null) return [];
  const toRelative = (file: string) =>
    path.relative(context.baseDir, file).split(path.sep).join("/");
  const kept: Record<string, unknown>[] = [];
  for (const override of overrides as Record<string, unknown>[]) {
    const rules = Object.fromEntries(
      Object.entries((override.rules ?? {}) as Record<string, unknown>).filter(([name]) =>
        typeAwareRules.has(name),
      ),
    );
    if (Object.keys(rules).length === 0) continue;
    const include = stringList(override.files);
    const exclude = stringList(override.excludeFiles);
    const files: string[] = [];
    for (const [original, shim] of context.shims) {
      const relative = toRelative(original);
      if (relative.startsWith("../")) continue;
      if (!include.some((glob) => matchesConfigGlob(relative, glob))) continue;
      if (exclude.some((glob) => matchesConfigGlob(relative, glob))) continue;
      files.push(literalGlob(toRelative(shim)));
    }
    if (files.length > 0) kept.push({ files, rules });
  }
  return kept;
}

/** The user's resolved config, reduced to its enabled type-aware rules. */
export function typeAwareOnlyConfig(
  printed: Record<string, unknown>,
  typeAwareRules: ReadonlySet<string>,
  context?: ShimContext,
): Record<string, unknown> | null {
  const rules: Record<string, unknown> = {};
  for (const [name, value] of Object.entries((printed.rules ?? {}) as Record<string, unknown>)) {
    const severity = Array.isArray(value) ? value[0] : value;
    if (!typeAwareRules.has(name) || severity === "off" || severity === "allow" || severity === 0)
      continue;
    rules[name] = value;
  }
  const overrides = typeAwareOverrides(printed.overrides, typeAwareRules, context);
  // Overrides may switch rules on for some files only, so they count as well.
  const overridesEnable = overrides.some((override) =>
    Object.values(override.rules as Record<string, unknown>).some((value) => {
      const severity = Array.isArray(value) ? value[0] : value;
      return severity !== "off" && severity !== "allow" && severity !== 0;
    }),
  );
  if (Object.keys(rules).length === 0 && !overridesEnable) return null;
  return {
    plugins: printed.plugins ?? [],
    categories: Object.fromEntries(CATEGORIES.map((category) => [category, "off"])),
    rules,
    ...(overrides.length > 0 ? { overrides } : {}),
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
  const printedConfig = JSON.parse(printed.stdout) as Record<string, unknown>;
  if (!enablesTypeAwareRule(printedConfig, typeAwareRules)) return undefined;

  const created: string[] = [];
  const shims = new Map<string, string>();
  const replacements = new Map<string, string>();
  // Override globs are relative to the user's config, so the pass configuration is
  // written next to it (under a unique name) instead of in the system temp directory.
  const baseDir = configBaseDir(configOptions, cwd);
  const configPath = path.join(baseDir, `.oxlintrc.vize-type-aware-${process.pid}.json`);
  const cleanup = () => {
    for (const file of created) fs.rmSync(file, { force: true });
    fs.rmSync(configPath, { force: true });
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
      shims.set(file, shimPath);
      registerPathReplacementVariants(replacements, cwd, shimPath, file);
    }
    if (created.length === 0) return undefined;
    const config = typeAwareOnlyConfig(printedConfig, typeAwareRules, { baseDir, shims });
    if (config == null) return undefined;
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

/** Whether any root rule or override turns a type-aware rule on. */
function enablesTypeAwareRule(
  printed: Record<string, unknown>,
  typeAwareRules: ReadonlySet<string>,
): boolean {
  const enabled = (rules: unknown) =>
    Object.entries((rules ?? {}) as Record<string, unknown>).some(([name, value]) => {
      const severity = Array.isArray(value) ? value[0] : value;
      return typeAwareRules.has(name) && !["off", "allow", 0].includes(severity as string | number);
    });
  const overrides = Array.isArray(printed.overrides)
    ? (printed.overrides as { rules?: unknown }[])
    : [];
  return enabled(printed.rules) || overrides.some((override) => enabled(override.rules));
}

/** Directory of the user's `-c` config, or the working directory for the discovered one. */
function configBaseDir(configOptions: readonly string[], cwd: string): string {
  for (let index = 0; index < configOptions.length; index += 1) {
    const option = configOptions[index];
    if (option === "-c" || option === "--config")
      return path.dirname(path.resolve(cwd, configOptions[index + 1] ?? ""));
    if (option.startsWith("--config="))
      return path.dirname(path.resolve(cwd, option.slice("--config=".length)));
  }
  return cwd;
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

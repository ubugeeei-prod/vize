/**
 * Oxlint artifact emission for the engine-neutral Nuxt lint plan.
 *
 * The plan deliberately keeps eslint-compatible Patina rule ids. Oxlint loads
 * Patina as the `vize` JavaScript plugin, so this boundary is the one place
 * those ids gain their `vize/` prefix.
 */
import path from "node:path";

import type { NuxtLintConfigItem, NuxtLintRuleConfig } from "@vizejs/nuxt-lint-config";

interface OxlintOverride {
  files: string[];
  excludeFiles?: string[];
  globals?: Record<string, "readonly" | "writable">;
  rules?: Record<string, NuxtLintRuleConfig>;
}

interface NuxtOxlintConfig {
  plugins: ["vue"];
  jsPlugins: Array<string | { name: "vize"; specifier: string }>;
  settings: { vize: { preset: "incremental"; generatedBy?: "@vizejs/nuxt" } };
  ignorePatterns?: string[];
  globals?: Record<string, "readonly" | "writable">;
  rules?: Record<string, NuxtLintRuleConfig>;
  overrides?: OxlintOverride[];
}

const PATINA_RULE_NAMESPACES = new Set([
  "a11y",
  "css",
  "ecosystem",
  "html",
  "musea",
  "nuxt",
  "petite-vue",
  "script",
  "ssr",
  "type",
  "vapor",
  "vize",
  "vue",
]);

function toOxlintRuleId(ruleId: string): string {
  const separator = ruleId.indexOf("/");
  const namespace = separator < 0 ? ruleId : ruleId.slice(0, separator);
  return PATINA_RULE_NAMESPACES.has(namespace) ? `vize/${ruleId}` : ruleId;
}

function prefixVizeRules(
  rules: Readonly<Record<string, NuxtLintRuleConfig>> | undefined,
): Record<string, NuxtLintRuleConfig> | undefined {
  if (!rules) return undefined;

  return Object.fromEntries(
    Object.entries(rules).map(([ruleId, severity]) => [toOxlintRuleId(ruleId), severity]),
  );
}

function mergeGlobals(
  target: Record<string, "readonly" | "writable">,
  source: Readonly<Record<string, "readonly" | "writable">> | undefined,
): void {
  if (!source) return;
  for (const [name, access] of Object.entries(source)) {
    // Assignment invokes Object.prototype's legacy `__proto__` setter. Nuxt
    // import aliases are arbitrary identifiers, so use a data property and
    // preserve even that spelling as an actual global.
    Object.defineProperty(target, name, {
      configurable: true,
      enumerable: true,
      value: access,
      writable: true,
    });
  }
}

function mergeRules(
  target: Record<string, NuxtLintRuleConfig>,
  source: Readonly<Record<string, NuxtLintRuleConfig>> | undefined,
): void {
  Object.assign(target, prefixVizeRules(source));
}

/**
 * Plan globs are relative to the Nuxt root. Oxlint resolves `overrides` and
 * `ignorePatterns` from the generated config's directory. That directory
 * must contain the plan root: Oxlint cannot ignore files outside it.
 */
export interface NuxtOxlintGlobBase {
  rootDir: string;
  configDir: string;
  /** Mark the module's reserved generated file; custom artifacts remain unmarked. */
  owner?: "@vizejs/nuxt";
}

/** Render the complete generated oxlint config, including its trailing newline. */
export function renderNuxtOxlintConfig(
  items: readonly NuxtLintConfigItem[],
  pluginSpecifier: string,
  globBase?: NuxtOxlintGlobBase,
  projectJsPlugins: readonly string[] = [],
): string {
  if (globBase) {
    const root = path.relative(globBase.configDir, globBase.rootDir);
    if (root === ".." || root.startsWith(`..${path.sep}`) || path.isAbsolute(root)) {
      throw new Error("Generated oxlint config must be in the lint root or an ancestor directory");
    }
  }
  const rebase = (glob: string, ignore = false) =>
    globBase ? rebaseGlob(glob, globBase, ignore) : glob;
  const ignorePatterns: string[] = [];
  const globals: Record<string, "readonly" | "writable"> = {};
  const rules: Record<string, NuxtLintRuleConfig> = {};
  const overrides: OxlintOverride[] = [];

  for (const item of items) {
    if (item.files) {
      const override: OxlintOverride = { files: item.files.map((glob) => rebase(glob)) };
      if (item.ignores) override.excludeFiles = item.ignores.map((glob) => rebase(glob));
      if (item.globals) override.globals = { ...item.globals };
      if (item.rules || item.nativeRules) {
        override.rules = { ...prefixVizeRules(item.rules), ...item.nativeRules };
      }
      overrides.push(override);
      continue;
    }

    if (item.ignores) ignorePatterns.push(...item.ignores.map((glob) => rebase(glob, true)));
    mergeGlobals(globals, item.globals);
    mergeRules(rules, item.rules);
    Object.assign(rules, item.nativeRules);
  }

  const config: NuxtOxlintConfig = {
    plugins: ["vue"],
    jsPlugins: [
      { name: "vize", specifier: pluginSpecifier },
      ...projectJsPlugins.map((specifier) => rebaseJsPlugin(specifier, globBase?.configDir)),
    ],
    settings: {
      vize: { preset: "incremental", ...(globBase?.owner ? { generatedBy: globBase.owner } : {}) },
    },
  };
  if (ignorePatterns.length > 0) config.ignorePatterns = ignorePatterns;
  if (Object.keys(globals).length > 0) config.globals = globals;
  if (Object.keys(rules).length > 0) config.rules = rules;
  if (overrides.length > 0) config.overrides = overrides;

  return `${JSON.stringify(config, null, 2)}\n`;
}

function rebaseJsPlugin(specifier: string, configDir: string | undefined): string {
  if (!configDir || !path.isAbsolute(specifier)) return specifier;
  const relative = path.relative(configDir, specifier).split(path.sep).join("/");
  return relative.startsWith(".") ? relative : `./${relative}`;
}

function rebaseGlob(glob: string, base: NuxtOxlintGlobBase, ignore: boolean): string {
  const negated = glob.startsWith("!");
  const body = negated ? glob.slice(1) : glob;
  const target = path.resolve(base.rootDir, body);
  let relative = path.relative(base.configDir, target);
  if (relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    if (ignore)
      throw new Error(`Oxlint cannot ignore a path outside its config directory: ${glob}`);
    // Overrides receive an absolute path for files outside the config directory.
    // Oxlint uses that same absolute path when it cannot strip the config root.
    relative = target;
  }
  const pattern = relative.split(path.sep).join("/") || ".";
  return negated ? `!${pattern}` : pattern;
}

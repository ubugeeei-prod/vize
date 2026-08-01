/**
 * Oxlint artifact emission for the engine-neutral Nuxt lint plan.
 *
 * The plan deliberately keeps eslint-compatible Patina rule ids. Oxlint loads
 * Patina as the `vize` JavaScript plugin, so this boundary is the one place
 * those ids gain their `vize/` prefix.
 */
import type { NuxtLintConfigItem, NuxtLintSeverity } from "./plan.ts";

interface OxlintOverride {
  files: string[];
  excludeFiles?: string[];
  globals?: Record<string, "readonly" | "writable">;
  rules?: Record<string, NuxtLintSeverity>;
}

interface NuxtOxlintConfig {
  plugins: ["vue"];
  jsPlugins: [{ name: "vize"; specifier: string }];
  settings: { vize: { preset: "incremental" } };
  ignorePatterns?: string[];
  globals?: Record<string, "readonly" | "writable">;
  rules?: Record<string, NuxtLintSeverity>;
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
  rules: Readonly<Record<string, NuxtLintSeverity>> | undefined,
): Record<string, NuxtLintSeverity> | undefined {
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
  target: Record<string, NuxtLintSeverity>,
  source: Readonly<Record<string, NuxtLintSeverity>> | undefined,
): void {
  Object.assign(target, prefixVizeRules(source));
}

/** Render the complete generated oxlint config, including its trailing newline. */
export function renderNuxtOxlintConfig(
  items: readonly NuxtLintConfigItem[],
  pluginSpecifier: string,
): string {
  const ignorePatterns: string[] = [];
  const globals: Record<string, "readonly" | "writable"> = {};
  const rules: Record<string, NuxtLintSeverity> = {};
  const overrides: OxlintOverride[] = [];

  for (const item of items) {
    if (item.files) {
      const override: OxlintOverride = { files: [...item.files] };
      if (item.ignores) override.excludeFiles = [...item.ignores];
      if (item.globals) override.globals = { ...item.globals };
      if (item.rules) override.rules = prefixVizeRules(item.rules);
      overrides.push(override);
      continue;
    }

    if (item.ignores) ignorePatterns.push(...item.ignores);
    mergeGlobals(globals, item.globals);
    mergeRules(rules, item.rules);
  }

  const config: NuxtOxlintConfig = {
    plugins: ["vue"],
    jsPlugins: [{ name: "vize", specifier: pluginSpecifier }],
    settings: { vize: { preset: "incremental" } },
  };
  if (ignorePatterns.length > 0) config.ignorePatterns = ignorePatterns;
  if (Object.keys(globals).length > 0) config.globals = globals;
  if (Object.keys(rules).length > 0) config.rules = rules;
  if (overrides.length > 0) config.overrides = overrides;

  return `${JSON.stringify(config, null, 2)}\n`;
}

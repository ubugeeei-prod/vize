import type { ESLint, Linter } from "eslint";
import * as typescriptParser from "@typescript-eslint/parser";
import * as vueParser from "vue-eslint-parser";
import bridge, { configs as ruleConfigs, createVizeRuleConfig } from "oxlint-plugin-vize/eslint";
import packageJson from "../package.json" with { type: "json" };

export type VizeConfigOptions = NonNullable<Parameters<typeof createVizeRuleConfig>[0]>;

// Keep the actual rule objects: collection, options, diagnostics and UTF-8 edits
// have one implementation shared with the existing public ESLint bridge.
const plugin: ESLint.Plugin & { configs: Record<keyof typeof ruleConfigs, Linter.Config> } = {
  meta: { name: packageJson.name, version: packageJson.version },
  rules: bridge.rules as unknown as NonNullable<ESLint.Plugin["rules"]>,
  configs: {} as Record<keyof typeof ruleConfigs, Linter.Config>,
};

function flatConfig(name: string, rules: Linter.RulesRecord): Linter.Config {
  return {
    name: `vize/${name}`,
    files: ["**/*.vue"],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: typescriptParser,
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: { jsx: true },
        extraFileExtensions: [".vue"],
      },
    },
    plugins: { vize: plugin },
    settings: { vize: { preset: "incremental" } },
    rules: { ...rules },
  };
}

/** A complete flat config; rule selection and severities use native metadata. */
export function createVizeConfig(options: VizeConfigOptions = {}): Linter.Config {
  return flatConfig("custom", createVizeRuleConfig(options));
}

for (const name of Object.keys(ruleConfigs) as (keyof typeof ruleConfigs)[]) {
  plugin.configs[name] = flatConfig(name, ruleConfigs[name]);
}

export { createVizeRuleConfig, ruleConfigs };
export default plugin;

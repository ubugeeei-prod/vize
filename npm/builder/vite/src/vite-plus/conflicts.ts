import type { UserConfig } from "vite-plus";
import type { VizePlusOptions } from "./types.ts";
import { availableVueRules } from "./runtime.ts";

// Native Patina implements these Vue/script rules. Keep other Oxlint
// diagnostics (including its JS/TS rules inside SFCs) enabled.
export const overlappingVueRules = [
  "component-definition-name-casing",
  "define-emits-declaration",
  "define-props-declaration",
  "define-props-destructuring",
  "no-arrow-functions-in-watch",
  "no-async-in-computed-properties",
  "no-deprecated-data-object-declaration",
  "no-deprecated-destroyed-lifecycle",
  "no-deprecated-events-api",
  "no-deprecated-props-default-this",
  "no-dupe-keys",
  "no-export-in-script-setup",
  "no-import-compiler-macros",
  "no-multiple-slot-args",
  "no-required-prop-with-default",
  "no-reserved-component-names",
  "no-reserved-keys",
  "no-reserved-props",
  "no-side-effects-in-computed-properties",
  "prefer-import-from-vue",
  "prop-name-casing",
  "require-default-prop",
  "require-prop-type-constructor",
  "require-prop-types",
  "require-typed-ref",
  "return-in-computed-property",
  "return-in-emits-validator",
  "valid-define-emits",
  "valid-define-options",
  "valid-define-props",
  "valid-next-tick",
] as const;

export function configureTools(config: UserConfig, options: VizePlusOptions): UserConfig {
  if (options.conflicts === false) return config;
  const supported = options.lint === false ? new Set<string>() : availableVueRules();
  return {
    ...config,
    lint:
      options.lint === false
        ? config.lint
        : {
            ...config.lint,
            ignorePatterns: [
              ...new Set([
                ...(config.lint?.ignorePatterns ?? []),
                "**/node_modules/**",
                "**/.vize/**",
              ]),
            ],
            plugins: [
              ...new Set([
                "vue" as const,
                ...(config.lint?.plugins ?? (["eslint", "typescript", "unicorn", "oxc"] as const)),
              ]),
            ],
            rules: {
              ...Object.fromEntries(
                overlappingVueRules
                  .filter((rule) => supported.has(rule))
                  .map((rule) => [`vue/${rule}`, "off" as const]),
              ),
              ...config.lint?.rules,
            },
          },
    fmt:
      options.fmt === false
        ? config.fmt
        : {
            ...config.fmt,
            ignorePatterns: [
              ...new Set([
                ...(config.fmt?.ignorePatterns ?? []),
                "**/node_modules/**",
                "**/.vize/**",
                "**/*.vue",
              ]),
            ],
          },
  };
}

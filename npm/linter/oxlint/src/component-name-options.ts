import { hasOnlyKeys, isRecord, optionalBooleanField } from "./plugin-option-guards.js";
import type { ComponentNameInTemplateCasingOption, PatinaRuleOptions } from "./model.js";

interface Policy {
  casing?: ComponentNameInTemplateCasingOption;
  registeredComponentsOnly?: boolean;
  globals?: string[];
  ignores?: string[];
}

function isCasing(value: unknown): value is ComponentNameInTemplateCasingOption {
  return value === "PascalCase" || value === "kebab-case";
}

function isPolicy(value: unknown, canonical: boolean): value is Policy {
  return (
    isRecord(value) &&
    hasOnlyKeys(
      value,
      canonical
        ? ["casing", "registeredComponentsOnly", "globals", "ignores"]
        : ["registeredComponentsOnly", "globals", "ignores"],
    ) &&
    (value.casing === undefined || (canonical && isCasing(value.casing))) &&
    optionalBooleanField(value.registeredComponentsOnly) &&
    (value.globals === undefined ||
      (Array.isArray(value.globals) &&
        value.globals.every(
          (name) => typeof name === "string" && (canonical || !/^\/(.+)\/(.*)$/u.test(name)),
        ))) &&
    isIgnores(value.ignores)
  );
}

/** Unique exact tag names or `/pattern/flags` entries, as in eslint-plugin-vue. */
function isIgnores(value: unknown): value is string[] | undefined {
  return (
    value === undefined ||
    (Array.isArray(value) &&
      value.every((name, index) => typeof name === "string" && value.indexOf(name) === index))
  );
}

/** Preserve the original style field and forward the complete authored tuple policy. */
export function getComponentNameCasingOptions(
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  if (options.length === 1 && isCasing(options[0]))
    return { componentNameInTemplateCasing: options[0] };
  if (options.length === 2 && isCasing(options[0]) && isPolicy(options[1], false))
    return {
      ruleOptions: {
        "vue/component-name-in-template-casing": {
          casing: options[0],
          ...structuredClone(options[1]),
        },
      },
    };
  if (options.length === 1 && isPolicy(options[0], true))
    return {
      ruleOptions: { "vue/component-name-in-template-casing": structuredClone(options[0]) },
    };
  return undefined;
}

/** The canonical policy is one item; only the official style can take a second item. */
export function componentNameCasingSchema(): Record<string, unknown> {
  const casing = { enum: ["PascalCase", "kebab-case"] };
  const properties = {
    registeredComponentsOnly: { type: "boolean" },
    globals: { type: "array", items: { type: "string" } },
    ignores: { type: "array", items: { type: "string" }, uniqueItems: true },
  };
  return {
    type: "array",
    oneOf: [
      {
        items: [
          casing,
          {
            type: "object",
            properties: {
              ...properties,
              globals: {
                type: "array",
                items: { type: "string", not: { pattern: "^/(.+)/(.*)$" } },
              },
            },
            additionalProperties: false,
          },
        ],
        maxItems: 2,
      },
      {
        items: [
          { type: "object", properties: { casing, ...properties }, additionalProperties: false },
        ],
        minItems: 1,
        maxItems: 1,
      },
    ],
  };
}

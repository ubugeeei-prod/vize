import { hasOnlyKeys, isRecord } from "./plugin-option-guards.js";
import type { PatinaRuleOptions } from "./model.js";

export type PropNameCasingStyle = "camelCase" | "snake_case";
export interface PropNameCasingOptions {
  casing?: PropNameCasingStyle;
}

function isStyle(value: unknown): value is PropNameCasingStyle {
  return value === "camelCase" || value === "snake_case";
}

/** The upstream style tuple and the CLI's complete object share one native policy. */
export function getPropNameCasingOptions(
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  if (options.length !== 1) return undefined;
  const first = options[0];
  if (isStyle(first)) return { ruleOptions: { "vue/prop-name-casing": { casing: first } } };
  if (
    isRecord(first) &&
    hasOnlyKeys(first, ["casing"]) &&
    (first.casing === undefined || isStyle(first.casing))
  )
    return { ruleOptions: { "vue/prop-name-casing": structuredClone(first) } };
  return undefined;
}

/** ignoreProps is outside the native policy; unknown keys/extra tuple options refuse. */
export function propNameCasingSchema(): Record<string, unknown> {
  return {
    oneOf: [
      { enum: ["camelCase", "snake_case"] },
      {
        type: "object",
        properties: { casing: { enum: ["camelCase", "snake_case"] } },
        additionalProperties: false,
      },
    ],
  };
}

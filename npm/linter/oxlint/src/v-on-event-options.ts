import type { HyphenationStyle, PatinaRuleOptions } from "./model.js";

export interface VOnEventHyphenationOptions {
  style?: HyphenationStyle;
  ignore?: string[];
  autofix?: boolean;
}

export function isVOnEventHyphenationOptions(value: unknown): value is VOnEventHyphenationOptions {
  if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
  if (!Object.keys(value).every((key) => key === "style" || key === "ignore" || key === "autofix"))
    return false;
  const option = value as Record<string, unknown>;
  if (option.style !== undefined && option.style !== "always" && option.style !== "never")
    return false;
  if (option.autofix !== undefined && typeof option.autofix !== "boolean") return false;
  const ignore = option.ignore;
  return (
    ignore === undefined ||
    (Array.isArray(ignore) &&
      ignore.every(
        (entry, index) =>
          typeof entry === "string" &&
          !/^\s*$/u.test(entry) &&
          !entry.endsWith(":exit") &&
          ignore.indexOf(entry) === index,
      ))
  );
}

/** Match the canonical CLI shape while retaining the original style string. */
export function vOnEventHyphenationSchema(): Record<string, unknown> {
  return {
    oneOf: [
      { enum: ["always", "never"] },
      {
        type: "object",
        properties: {
          style: { enum: ["always", "never"] },
          autofix: { type: "boolean" },
          ignore: {
            type: "array",
            uniqueItems: true,
            items: { type: "string", pattern: "^(?!\\s*$)(?![\\s\\S]*:exit$)" },
          },
        },
        additionalProperties: false,
      },
    ],
  };
}

/** Carry the supported official tuple without changing canonical/default policy. */
export function getVOnEventHyphenationOptions(
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  const first = options[0];
  const isStyle = first === "always" || first === "never";
  if (options.length === 1 && isStyle) return { vOnEventHyphenation: first };
  if (options.length === 1 && isVOnEventHyphenationOptions(first))
    return { ruleOptions: { "vue/v-on-event-hyphenation": structuredClone(first) } };
  const second = options[1];
  if (
    options.length === 2 &&
    isStyle &&
    isVOnEventHyphenationOptions(second) &&
    !Object.hasOwn(second, "style")
  )
    return {
      ruleOptions: {
        "vue/v-on-event-hyphenation": { style: first, ...structuredClone(second) },
      },
    };
  return undefined;
}

/** Canonical object policies remain one item; the official style accepts a policy. */
export function vOnEventHyphenationRuleSchema(): Record<string, unknown> {
  const style = { enum: ["always", "never"] };
  const policy = {
    type: "object",
    properties: {
      autofix: { type: "boolean" },
      ignore: {
        type: "array",
        uniqueItems: true,
        items: { type: "string", pattern: "^(?!\\s*$)(?![\\s\\S]*:exit$)" },
      },
    },
    additionalProperties: false,
  };
  return {
    type: "array",
    oneOf: [
      { items: [style, policy], maxItems: 2 },
      {
        items: [{ ...policy, properties: { style, ...policy.properties } }],
        minItems: 1,
        maxItems: 1,
      },
    ],
  };
}

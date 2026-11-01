import type { HyphenationStyle, PatinaRuleOptions } from "./model.js";

export interface AttributeHyphenationOptions {
  style?: HyphenationStyle;
  ignore?: string[];
}

function isIgnore(value: unknown): value is string[] | undefined {
  return (
    value === undefined ||
    (Array.isArray(value) &&
      value.every(
        (entry, index) =>
          typeof entry === "string" &&
          !/^\s*$/u.test(entry) &&
          !entry.endsWith(":exit") &&
          value.indexOf(entry) === index,
      ))
  );
}

function isOptions(value: unknown, allowStyle: boolean): value is AttributeHyphenationOptions {
  if (value === null || typeof value !== "object" || Array.isArray(value)) return false;
  const options = value as Record<string, unknown>;
  if (!Object.keys(options).every((key) => key === "ignore" || (allowStyle && key === "style")))
    return false;
  if (
    allowStyle &&
    options.style !== undefined &&
    options.style !== "always" &&
    options.style !== "never"
  )
    return false;
  return isIgnore(options.ignore);
}

/** Preserve the old style tuple; accept upstream's second ignore object and the canonical object. */
export function getAttributeHyphenationOptions(
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  if (options.length > 2) return undefined;
  const first = options[0],
    second = options[1];
  if (first === "always" || first === "never") {
    if (second === undefined) return { attributeHyphenation: first };
    if (!isOptions(second, false)) return undefined;
    return { attributeHyphenation: first, attributeHyphenationIgnore: [...(second.ignore ?? [])] };
  }
  if (isOptions(first, true) && second !== undefined) {
    throw new TypeError(
      "attribute-hyphenation accepts a canonical object or a style/ignore tuple, not both",
    );
  }
  if (second === undefined && isOptions(first, true)) {
    return { ruleOptions: { "vue/attribute-hyphenation": structuredClone(first) } };
  }
  return undefined;
}

function ignoreSchema(): Record<string, unknown> {
  return {
    type: "array",
    uniqueItems: true,
    items: {
      type: "string",
      pattern: "^(?!\\s*$)(?![\\s\\S]*:exit$)",
    },
  };
}

export function attributeHyphenationSchema(): Record<string, unknown>[] {
  return [
    {
      oneOf: [
        { enum: ["always", "never"] },
        {
          type: "object",
          properties: { style: { enum: ["always", "never"] }, ignore: ignoreSchema() },
          additionalProperties: false,
        },
      ],
    },
    { type: "object", properties: { ignore: ignoreSchema() }, additionalProperties: false },
  ];
}

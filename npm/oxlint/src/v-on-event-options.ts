import type { HyphenationStyle } from "./model.js";

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

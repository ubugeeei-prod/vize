import { hasOnlyKeys, isRecord, isString, optionalBooleanField } from "./plugin-option-guards.js";
import type { PatinaRuleOptions } from "./model.js";

export interface BlockLangBlockOptions {
  lang?: string | string[];
  allowNoLang?: boolean;
  required?: boolean;
}

export interface BlockLangRuleOptions {
  script?: BlockLangBlockOptions;
  template?: BlockLangBlockOptions;
  style?: BlockLangBlockOptions;
}

export interface MaxLinesObjectOptions {
  max?: number;
  skipComments?: boolean;
  skipBlankLines?: boolean;
}

export type MaxLinesRuleOptions = number | MaxLinesObjectOptions;

export interface MaxLinesPerBlockRuleOptions {
  style?: number;
  template?: number;
  script?: number;
  skipBlankLines?: boolean;
}

const BLOCK_KEYS = ["script", "template", "style"] as const;
const BLOCK_FIELD_KEYS = ["lang", "allowNoLang", "required"] as const;

function isWholeNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0;
}

function isLangName(value: unknown): value is string {
  return isString(value) && value.length > 0;
}

function isLangList(value: unknown): value is string | string[] {
  return isLangName(value) || (Array.isArray(value) && value.length > 0 && value.every(isLangName));
}

function isBlockOptions(value: unknown): value is BlockLangBlockOptions {
  if (!isRecord(value) || !hasOnlyKeys(value, BLOCK_FIELD_KEYS)) return false;
  return (
    (value.lang === undefined || isLangList(value.lang)) &&
    optionalBooleanField(value.allowNoLang) &&
    optionalBooleanField(value.required)
  );
}

/** eslint-plugin-vue's `vue/block-lang` object, forwarded unchanged. */
export function getBlockLangOptions(options: readonly unknown[]): PatinaRuleOptions | undefined {
  if (options.length !== 1) return undefined;
  const first = options[0];
  if (!isRecord(first) || !hasOnlyKeys(first, BLOCK_KEYS)) return undefined;
  const blockLang: BlockLangRuleOptions = {};
  for (const key of BLOCK_KEYS) {
    const value = first[key];
    if (value === undefined) continue;
    if (!isBlockOptions(value)) return undefined;
    blockLang[key] = structuredClone(value);
  }
  return { ruleOptions: { "vue/block-lang": blockLang } };
}

export function blockLangSchema(): Record<string, unknown> {
  const lang = {
    oneOf: [
      { type: "string", minLength: 1 },
      { type: "array", items: { type: "string", minLength: 1 }, minItems: 1 },
    ],
  };
  const block = {
    type: "object",
    properties: {
      lang,
      allowNoLang: { type: "boolean" },
      required: { type: "boolean" },
    },
    additionalProperties: false,
  };
  return {
    type: "object",
    properties: { script: block, template: block, style: block },
    additionalProperties: false,
  };
}

/** ESLint `max-lines`: a maximum, or `{ max, skipComments, skipBlankLines }`. */
export function getMaxLinesOptions(options: readonly unknown[]): PatinaRuleOptions | undefined {
  if (options.length !== 1) return undefined;
  const first = options[0];
  if (isWholeNumber(first)) return { ruleOptions: { "max-lines": first } };
  if (
    isRecord(first) &&
    hasOnlyKeys(first, ["max", "skipComments", "skipBlankLines"]) &&
    (first.max === undefined || isWholeNumber(first.max)) &&
    optionalBooleanField(first.skipComments) &&
    optionalBooleanField(first.skipBlankLines)
  ) {
    const maxLines: MaxLinesObjectOptions = {};
    if (isWholeNumber(first.max)) maxLines.max = first.max;
    if (first.skipComments === true) maxLines.skipComments = true;
    if (first.skipComments === false) maxLines.skipComments = false;
    if (first.skipBlankLines === true) maxLines.skipBlankLines = true;
    if (first.skipBlankLines === false) maxLines.skipBlankLines = false;
    return { ruleOptions: { "max-lines": maxLines } };
  }
  return undefined;
}

export function maxLinesSchema(): Record<string, unknown> {
  return {
    oneOf: [
      { type: "integer", minimum: 0 },
      {
        type: "object",
        properties: {
          max: { type: "integer", minimum: 0 },
          skipComments: { type: "boolean" },
          skipBlankLines: { type: "boolean" },
        },
        additionalProperties: false,
      },
    ],
  };
}

/** `vue/max-lines-per-block` limits, matching the native option object. */
export function getMaxLinesPerBlockOptions(
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  if (options.length !== 1) return undefined;
  const first = options[0];
  if (
    !isRecord(first) ||
    !hasOnlyKeys(first, ["style", "template", "script", "skipBlankLines"]) ||
    !["style", "template", "script"].every(
      (key) => first[key] === undefined || isWholeNumber(first[key]),
    ) ||
    !optionalBooleanField(first.skipBlankLines)
  ) {
    return undefined;
  }
  const perBlock: MaxLinesPerBlockRuleOptions = {};
  if (isWholeNumber(first.style)) perBlock.style = first.style;
  if (isWholeNumber(first.template)) perBlock.template = first.template;
  if (isWholeNumber(first.script)) perBlock.script = first.script;
  if (typeof first.skipBlankLines === "boolean") perBlock.skipBlankLines = first.skipBlankLines;
  return { ruleOptions: { "vue/max-lines-per-block": perBlock } };
}

export function maxLinesPerBlockSchema(): Record<string, unknown> {
  const limit = { type: "integer", minimum: 0 };
  return {
    type: "object",
    properties: {
      style: limit,
      template: limit,
      script: limit,
      skipBlankLines: { type: "boolean" },
    },
    additionalProperties: false,
  };
}

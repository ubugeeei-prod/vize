import { hasOnlyKeys, isRecord, isString } from "./plugin-option-guards.js";
import type { PatinaRuleOptions, RestrictedHtmlElementEntry, RestrictedVOnEntry } from "./model.js";

const HTML_ELEMENTS = "vue/no-restricted-html-elements";
const V_ON = "vue/no-restricted-v-on";

function isOptionalString(value: unknown): value is string | undefined {
  return value === undefined || isString(value);
}

function isStringList(value: unknown): value is string[] {
  return Array.isArray(value) && value.every(isString);
}

/** A name, or `{ element, message }` as eslint-plugin-vue writes it. */
function isRestrictedHtmlElementEntry(value: unknown): value is RestrictedHtmlElementEntry {
  return (
    isString(value) ||
    (isRecord(value) &&
      hasOnlyKeys(value, ["element", "message"]) &&
      (isString(value.element) || isStringList(value.element)) &&
      isOptionalString(value.message))
  );
}

/** A name, or `{ argument, modifiers, element, message }`. A `null` argument is not supported. */
function isRestrictedVOnEntry(value: unknown): value is RestrictedVOnEntry {
  return (
    isString(value) ||
    (isRecord(value) &&
      hasOnlyKeys(value, ["argument", "modifiers", "element", "message"]) &&
      isString(value.argument) &&
      (value.modifiers === undefined || isStringList(value.modifiers)) &&
      isOptionalString(value.element) &&
      isOptionalString(value.message))
  );
}

/** The official options are the entry list itself, which is the native list form. */
export function getRestrictedHtmlElementsOptions(
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  return options.length > 0 && options.every(isRestrictedHtmlElementEntry)
    ? { ruleOptions: { [HTML_ELEMENTS]: structuredClone([...options]) } }
    : undefined;
}

export function getRestrictedVOnOptions(
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  return options.length > 0 && options.every(isRestrictedVOnEntry)
    ? { ruleOptions: { [V_ON]: structuredClone([...options]) } }
    : undefined;
}

const name = { type: "string" };
const names = { type: "array", items: name };

export function restrictedHtmlElementsSchema(): Record<string, unknown> {
  return {
    type: "array",
    items: {
      oneOf: [
        name,
        {
          type: "object",
          properties: { element: { oneOf: [name, names] }, message: name },
          required: ["element"],
          additionalProperties: false,
        },
      ],
    },
  };
}

export function restrictedVOnSchema(): Record<string, unknown> {
  return {
    type: "array",
    items: {
      oneOf: [
        name,
        {
          type: "object",
          properties: { argument: name, modifiers: names, element: name, message: name },
          required: ["argument"],
          additionalProperties: false,
        },
      ],
    },
  };
}

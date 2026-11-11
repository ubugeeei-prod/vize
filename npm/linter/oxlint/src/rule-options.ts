import {
  hasOnlyKeys,
  isRecord,
  isString,
  optionField,
  optionalBooleanField,
} from "./plugin-option-guards.js";
import { getVOnEventHyphenationOptions } from "./v-on-event-options.js";
import { getAttributeHyphenationOptions } from "./attribute-options.js";
import { getPropNameCasingOptions } from "./prop-name-options.js";
import { getComponentNameCasingOptions } from "./component-name-options.js";
import type {
  CustomEventNameCasingOption,
  DefineMacrosOrderOption,
  HtmlQuoteStyle,
  HtmlSelfClosingOption,
  MustacheInterpolationSpacingStyle,
  NoMutatingPropsOption,
  PatinaRuleOptions,
  SfcElementOrderOption,
  VBindStyle,
  VOnStyle,
  VSlotStyle,
} from "./model.js";

export function getRuleOptions(
  ruleName: string,
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  const firstOption = options[0];
  switch (ruleName) {
    case "vue/prop-name-casing":
      return getPropNameCasingOptions(options);
    case "vue/component-name-in-template-casing":
      return getComponentNameCasingOptions(options);
    case "script/custom-event-name-casing":
      if (isCustomEventNameCasingOption(firstOption)) {
        return { customEventNameCasing: firstOption };
      }
      if (isRecord(firstOption)) return cliRuleOptions(ruleName, firstOption);
      break;
    case "vue/no-mutating-props":
      if (isNoMutatingPropsOption(firstOption)) {
        return { noMutatingProps: firstOption };
      }
      break;
    case "script/define-macros-order":
      if (isDefineMacrosOrderOption(firstOption)) {
        return { defineMacrosOrder: firstOption };
      }
      break;
    case "vue/sfc-element-order":
      if (isSfcElementOrderOption(firstOption)) {
        return { sfcElementOrder: firstOption };
      }
      break;
    case "vue/html-self-closing":
      if (isHtmlSelfClosingOption(firstOption)) {
        return { htmlSelfClosing: firstOption };
      }
      break;
    case "vue/html-quotes": {
      if (!isHtmlQuoteStyle(firstOption)) break;
      const secondOption = options[1];
      if (secondOption === undefined) {
        return { ruleOptions: { "vue/html-quotes": firstOption } };
      }
      if (
        isRecord(secondOption) &&
        hasOnlyKeys(secondOption, ["avoidEscape"]) &&
        optionalBooleanField(secondOption.avoidEscape)
      ) {
        return {
          ruleOptions: {
            "vue/html-quotes": {
              style: firstOption,
              avoidEscape: secondOption.avoidEscape ?? false,
            },
          },
        };
      }
      break;
    }
    case "vue/mustache-interpolation-spacing":
      if (isMustacheInterpolationSpacingStyle(firstOption)) {
        return { ruleOptions: { "vue/mustache-interpolation-spacing": firstOption } };
      }
      break;
    case "vue/v-on-event-hyphenation":
      return getVOnEventHyphenationOptions(options);
    case "vue/v-on-style":
      if (isVOnStyle(firstOption)) {
        return { ruleOptions: { "vue/v-on-style": firstOption } };
      }
      break;
    case "vue/v-slot-style":
      if (isVSlotStyle(firstOption)) {
        return { ruleOptions: { "vue/v-slot-style": structuredClone(firstOption) } };
      }
      break;
    case "vue/v-bind-style":
      if (isVBindStyle(firstOption)) {
        return { ruleOptions: { "vue/v-bind-style": firstOption } };
      }
      break;
    case "vue/attribute-hyphenation":
      return getAttributeHyphenationOptions(options);
    case "vue/no-restricted-syntax":
      if (options.every(isSupportedRestrictedSyntaxEntry)) {
        return {
          ruleOptions: {
            "vue/no-restricted-syntax": { selectors: [...options] },
          },
        };
      }
      break;
    case "vue/quote-props":
      if (firstOption === "as-needed") {
        return { ruleOptions: { "vue/quote-props": firstOption } };
      }
      break;
    case "vue/sort-keys": {
      if (firstOption !== undefined && firstOption !== "asc" && firstOption !== "desc") break;
      const secondOption = options[1];
      if (
        secondOption !== undefined &&
        (!isRecord(secondOption) ||
          !hasOnlyKeys(secondOption, ["caseSensitive", "natural"]) ||
          !optionalBooleanField(secondOption.caseSensitive) ||
          !optionalBooleanField(secondOption.natural))
      ) {
        break;
      }
      return {
        ruleOptions: {
          "vue/sort-keys": {
            order: firstOption ?? "asc",
            ...(isRecord(secondOption) ? structuredClone(secondOption) : {}),
          },
        },
      };
    }
    case "script/define-props-destructuring":
    case "html/no-empty-palpable-content":
    case "type/strict-boolean-expressions":
    case "script/no-restricted-globals":
    case "script/no-restricted-members":
    case "vue/warn-custom-directive":
    case "vue/no-bare-strings-in-template":
    case "a11y/anchor-has-content":
    case "a11y/label-has-for":
    case "vue/enforce-style-attribute":
    case "vue/require-component-registration":
    case "vue/valid-v-on":
    case "vue/valid-v-slot":
    case "musea/prefer-design-tokens":
      if (isRecord(firstOption)) return cliRuleOptions(ruleName, firstOption);
      break;
  }
  return undefined;
}

const UNSUPPORTED_PSEUDO_CLASS =
  /:(?!(?:matches|is|not|has|nth-child|nth-last-child)\(|(?:first-child|last-child|statement|expression|declaration|function|pattern)\b)/u;

/** A string or `{ selector, message }` entry within Patina's esquery subset. */
function isSupportedRestrictedSyntaxEntry(
  value: unknown,
): value is string | { selector: string; message?: string } {
  const selector = isString(value)
    ? value
    : isRecord(value) &&
        hasOnlyKeys(value, ["selector", "message"]) &&
        isString(value.selector) &&
        (value.message === undefined || isString(value.message))
      ? value.selector
      : undefined;
  return (
    selector !== undefined &&
    selector.trim() !== "" &&
    !UNSUPPORTED_PSEUDO_CLASS.test(selector.replace(/(["'])(?:\\.|(?!\1).)*\1|\/(?:\\.|[^/])*\//gu, ""))
  );
}

function isDefineMacrosOrderOption(value: unknown): value is DefineMacrosOrderOption {
  if (!isRecord(value) || !hasOnlyKeys(value, ["order"])) {
    return false;
  }
  const order = value.order;
  return (
    order === undefined ||
    (Array.isArray(order) &&
      order.every(
        (name, index) => isString(name) && name.length > 0 && order.indexOf(name) === index,
      ))
  );
}

function cliRuleOptions(ruleName: string, option: Record<string, unknown>): PatinaRuleOptions {
  return { ruleOptions: { [ruleName]: structuredClone(option) } };
}

function isCustomEventNameCasingOption(value: unknown): value is CustomEventNameCasingOption {
  return value === "lowercase" || value === "camelCase" || value === "kebab-case";
}

function isNoMutatingPropsOption(value: unknown): value is NoMutatingPropsOption {
  if (!isRecord(value)) {
    return false;
  }
  return hasOnlyKeys(value, ["shallowOnly"]) && optionalBooleanField(value.shallowOnly);
}

function isSfcElementOrderOption(value: unknown): value is SfcElementOrderOption {
  if (!isRecord(value)) {
    return false;
  }
  return (
    hasOnlyKeys(value, ["order"]) &&
    (value.order === undefined ||
      (Array.isArray(value.order) && value.order.every(isSfcElementOrderGroup)))
  );
}

function isSfcElementOrderGroup(value: unknown): boolean {
  return typeof value === "string" || (Array.isArray(value) && value.every(isString));
}

function isHtmlSelfClosingOption(value: unknown): value is HtmlSelfClosingOption {
  if (!isRecord(value)) {
    return false;
  }
  return (
    hasOnlyKeys(value, ["html", "svg", "math"]) &&
    optionField(value.svg) &&
    optionField(value.math) &&
    (value.html === undefined ||
      (isRecord(value.html) &&
        hasOnlyKeys(value.html, ["void", "normal", "component"]) &&
        optionField(value.html.void) &&
        optionField(value.html.normal) &&
        optionField(value.html.component)))
  );
}

function isHtmlQuoteStyle(value: unknown): value is HtmlQuoteStyle {
  return value === "double" || value === "single";
}

function isMustacheInterpolationSpacingStyle(
  value: unknown,
): value is MustacheInterpolationSpacingStyle {
  return value === "always" || value === "never";
}

function isVOnStyle(value: unknown): value is VOnStyle {
  return value === "shorthand" || value === "longform";
}

function isVSlotStyle(value: unknown): value is VSlotStyle {
  if (value === "shorthand" || value === "longform") return true;
  if (!isRecord(value) || !hasOnlyKeys(value, ["atComponent", "default", "named"])) return false;
  const isSlotValue = (entry: unknown) =>
    entry === undefined || entry === "shorthand" || entry === "longform" || entry === "v-slot";
  return (
    isSlotValue(value.atComponent) &&
    isSlotValue(value.default) &&
    (value.named === undefined || value.named === "shorthand" || value.named === "longform")
  );
}

function isVBindStyle(value: unknown): value is VBindStyle {
  return value === "shorthand" || value === "longform";
}

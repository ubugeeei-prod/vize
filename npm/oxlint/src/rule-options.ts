import {
  hasOnlyKeys,
  isRecord,
  isString,
  optionField,
  optionalBooleanField,
} from "./plugin-option-guards.js";
import type {
  ComponentNameInTemplateCasingOption,
  CustomEventNameCasingOption,
  DefineMacrosOrderOption,
  HtmlSelfClosingOption,
  HyphenationStyle,
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
    case "vue/component-name-in-template-casing":
      if (isComponentNameInTemplateCasingOption(firstOption)) {
        return { componentNameInTemplateCasing: firstOption };
      }
      if (isRecord(firstOption)) return cliRuleOptions(ruleName, firstOption);
      break;
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
    case "vue/v-on-event-hyphenation":
      if (isHyphenationStyle(firstOption)) {
        return { vOnEventHyphenation: firstOption };
      }
      break;
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
      if (isHyphenationStyle(firstOption)) {
        return { attributeHyphenation: firstOption };
      }
      break;
    case "script/define-props-destructuring":
    case "html/no-empty-palpable-content":
    case "type/strict-boolean-expressions":
    case "script/no-restricted-globals":
    case "script/no-restricted-members":
    case "vue/require-component-registration":
    case "vue/valid-v-on":
    case "vue/valid-v-slot":
    case "musea/prefer-design-tokens":
      if (isRecord(firstOption)) return cliRuleOptions(ruleName, firstOption);
      break;
  }
  return undefined;
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

function isComponentNameInTemplateCasingOption(
  value: unknown,
): value is ComponentNameInTemplateCasingOption {
  return value === "PascalCase" || value === "kebab-case";
}

function isCustomEventNameCasingOption(value: unknown): value is CustomEventNameCasingOption {
  return value === "camelCase" || value === "kebab-case";
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

function isHyphenationStyle(value: unknown): value is HyphenationStyle {
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

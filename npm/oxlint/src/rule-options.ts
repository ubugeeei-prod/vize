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
  HtmlSelfClosingOption,
  HyphenationStyle,
  NoMutatingPropsOption,
  PatinaRuleOptions,
  SfcElementOrderOption,
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
      break;
    case "script/custom-event-name-casing":
      if (isCustomEventNameCasingOption(firstOption)) {
        return { customEventNameCasing: firstOption };
      }
      break;
    case "vue/no-mutating-props":
      if (isNoMutatingPropsOption(firstOption)) {
        return { noMutatingProps: firstOption };
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
    case "vue/attribute-hyphenation":
      if (isHyphenationStyle(firstOption)) {
        return { attributeHyphenation: firstOption };
      }
      break;
  }
  return undefined;
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

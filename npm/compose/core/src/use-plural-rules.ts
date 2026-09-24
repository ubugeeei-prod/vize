import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useLocale } from "./locale.ts";

/** CLDR plural categories. */
export type PluralCategory = Intl.LDMLPluralRule;

/**
 * One plural message: a template where `{count}` and `#` are replaced by the
 * locale-formatted count, or a function receiving the raw and formatted count.
 */
export type PluralMessage = string | ((count: number, formatted: string) => string);

/**
 * Messages per plural category. `other` is required because every locale
 * falls back to it; exact-value overrides such as `"=0"` take precedence
 * over categories.
 */
export type PluralMessages = {
  readonly [Category in Exclude<PluralCategory, "other">]?: PluralMessage;
} & {
  /** Required fallback message. */
  readonly other: PluralMessage;
} & {
  readonly [exact: `=${number}`]: PluralMessage | undefined;
};

/** Options for {@link usePluralRules}. */
export interface UsePluralRulesOptions extends Intl.PluralRulesOptions {
  /**
   * Locale used for plural selection and count formatting. Pass it
   * explicitly for hydration-stable server rendering.
   *
   * @default the browser language, otherwise "en"
   */
  readonly locale?: string | Intl.Locale;
}

/** Reactive plural selection returned by {@link usePluralRules}. */
export interface PluralRulesControls {
  /** Canonical locale in use. */
  readonly locale: ComputedRef<string>;

  /** Plural rules for the current locale and options. */
  readonly rules: ComputedRef<Intl.PluralRules>;

  /** Category of the current count. */
  readonly category: ComputedRef<PluralCategory>;

  /** Message for the current count with `{count}` / `#` interpolated. */
  readonly message: ComputedRef<string>;

  /**
   * Plural category of any count.
   *
   * @param count Count to classify.
   * @returns The CLDR category.
   */
  readonly select: (count: number) => PluralCategory;

  /**
   * Message for any count.
   *
   * @param count Count to render.
   * @returns The interpolated message.
   */
  readonly format: (count: number) => string;
}

function browserLanguage(): string | undefined {
  return typeof window === "undefined" ? undefined : window.navigator.language;
}

function renderMessage(message: PluralMessage, count: number, formatted: string): string {
  return typeof message === "function"
    ? message(count, formatted)
    : message.replaceAll("{count}", formatted).replaceAll("#", formatted);
}

/**
 * Choose and render a message by CLDR plural category (`Intl.PluralRules`).
 *
 * Messages are typed per category with a mandatory `other`, plus optional
 * exact overrides (`"=0": "No items"`). Cardinal (`1 item`) and ordinal
 * (`1st`, `2nd`) rules are supported via `type`. The count is formatted
 * with the same locale for interpolation. Everything is derived state:
 * no listeners or timers, deterministic during server rendering when the
 * locale is explicit. Invalid options surface as the platform `RangeError`.
 *
 * @example
 * ```ts
 * const { message } = usePluralRules(count, { "=0": "No files", one: "# file", other: "# files" }, { locale: "en" });
 * ```
 *
 * @param count Reactive count.
 * @param messages Reactive messages per category.
 * @param options Reactive locale and `Intl.PluralRules` options.
 * @default options {}
 * @returns Reactive category, message, and selection helpers.
 */
export function usePluralRules(
  count: MaybeRefOrGetter<number>,
  messages: MaybeRefOrGetter<PluralMessages>,
  options: MaybeRefOrGetter<UsePluralRulesOptions> = {},
): PluralRulesControls {
  const localeControls = useLocale(() => toValue(options).locale, { detect: browserLanguage });
  const rules = computed(() => {
    const { locale: _locale, ...ruleOptions } = toValue(options);
    return new Intl.PluralRules(localeControls.locale.value, ruleOptions);
  });
  const numberFormat = computed(() => {
    const { minimumFractionDigits, maximumFractionDigits } = toValue(options);
    return localeControls.number({
      ...(minimumFractionDigits === undefined ? {} : { minimumFractionDigits }),
      ...(maximumFractionDigits === undefined ? {} : { maximumFractionDigits }),
    });
  });
  const select = (value: number): PluralCategory => rules.value.select(value);
  const format = (value: number): string => {
    const table = toValue(messages);
    const exact: `=${number}` = `=${value}`;
    const message = table[exact] ?? table[select(value)] ?? table.other;
    return renderMessage(message, value, numberFormat.value.format(value));
  };
  return {
    locale: localeControls.locale,
    rules,
    category: computed(() => select(toValue(count))),
    message: computed(() => format(toValue(count))),
    select,
    format,
  };
}

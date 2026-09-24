import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useLocale } from "./locale.ts";

/** Options for {@link useListFormat}. */
export interface UseListFormatOptions extends Intl.ListFormatOptions {
  /**
   * Locale used for formatting. Pass it explicitly for hydration-stable
   * server rendering.
   *
   * @default the browser language, otherwise "en"
   */
  readonly locale?: string | Intl.Locale;
}

/** Reactive list formatting returned by {@link useListFormat}. */
export interface ListFormatControls {
  /** Canonical locale in use. */
  readonly locale: ComputedRef<string>;

  /** Cached formatter for the current locale and options. */
  readonly formatter: ComputedRef<Intl.ListFormat>;

  /** Formatted list, for example `"apples, pears, and plums"`. */
  readonly formatted: ComputedRef<string>;

  /**
   * Format any list with the current formatter.
   *
   * @param items Items to join.
   * @returns The formatted list.
   */
  readonly format: (items: Iterable<string>) => string;

  /**
   * Format any list into element and literal parts (for rich rendering).
   *
   * @param items Items to join.
   * @returns The formatted parts.
   */
  readonly formatToParts: (
    items: Iterable<string>,
  ) => { type: "element" | "literal"; value: string }[];
}

function browserLanguage(): string | undefined {
  return typeof window === "undefined" ? undefined : window.navigator.language;
}

/**
 * Join strings into a locale-aware list with `Intl.ListFormat`
 * (`"a, b, and c"`, `"a、b、c"`, `"a, b or c"`).
 *
 * Follows reactive items and options; formatters are cached per locale and
 * option set through {@link useLocale}. Invalid options surface as the
 * platform's `RangeError`. No listeners or timers are created.
 *
 * @example
 * ```ts
 * const { formatted } = useListFormat(names, { locale: "en", type: "disjunction" });
 * ```
 *
 * @param items Reactive list of strings.
 * @param options Reactive locale and `Intl.ListFormat` options.
 * @default options {}
 * @returns Reactive formatted output and formatting helpers.
 */
export function useListFormat(
  items: MaybeRefOrGetter<readonly string[]>,
  options: MaybeRefOrGetter<UseListFormatOptions> = {},
): ListFormatControls {
  const localeControls = useLocale(() => toValue(options).locale, { detect: browserLanguage });
  const formatter = computed(() => {
    const { locale: _locale, ...formatOptions } = toValue(options);
    return localeControls.list(formatOptions);
  });
  return {
    locale: localeControls.locale,
    formatter,
    formatted: computed(() => formatter.value.format(toValue(items))),
    format: (list) => formatter.value.format(list),
    formatToParts: (list) => formatter.value.formatToParts(list),
  };
}

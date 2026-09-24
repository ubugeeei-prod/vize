import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useLocale } from "./locale.ts";

/** Numeric values accepted by `Intl.NumberFormat`. */
export type FormattableNumber = number | bigint;

/** Options for {@link useNumberFormat}. */
export interface UseNumberFormatOptions extends Intl.NumberFormatOptions {
  /**
   * Locale used for formatting. Pass it explicitly for server rendering:
   * without it the server falls back to `"en"` while the browser detects
   * `navigator.language`, which can make hydration mismatch.
   *
   * @default the browser language, otherwise "en"
   */
  readonly locale?: string | Intl.Locale;
}

/** Reactive number formatting returned by {@link useNumberFormat}. */
export interface NumberFormatControls {
  /** Canonical locale in use. */
  readonly locale: ComputedRef<string>;

  /** Cached formatter for the current locale and options. */
  readonly formatter: ComputedRef<Intl.NumberFormat>;

  /** Formatted `value`, or `""` while the value is `null`/`undefined`. */
  readonly formatted: ComputedRef<string>;

  /**
   * Format any number with the current formatter.
   *
   * @param value Number to format.
   * @returns The formatted string.
   */
  readonly format: (value: FormattableNumber) => string;

  /**
   * Format any number into locale-aware parts.
   *
   * @param value Number to format.
   * @returns The formatted parts.
   */
  readonly formatToParts: (value: FormattableNumber) => Intl.NumberFormatPart[];

  /**
   * Format a numeric range (for example `"3–5"`).
   *
   * @param start Range start.
   * @param end Range end.
   * @returns The formatted range.
   */
  readonly formatRange: (start: FormattableNumber, end: FormattableNumber) => string;
}

function formatRangeWith(
  formatter: Intl.NumberFormat,
  start: FormattableNumber,
  end: FormattableNumber,
): string {
  // `formatRange` is ES2023; older engines fall back to joining both ends.
  const native: unknown = Reflect.get(formatter, "formatRange");
  if (typeof native === "function") {
    const result: unknown = Reflect.apply(native, formatter, [start, end]);
    if (typeof result === "string") return result;
  }
  return `${formatter.format(start)}\u2013${formatter.format(end)}`;
}

function browserLanguage(): string | undefined {
  return typeof window === "undefined" ? undefined : window.navigator.language;
}

/**
 * Format numbers reactively with `Intl.NumberFormat`.
 *
 * The formatter follows the reactive options (including `locale`) and is
 * cached per locale and option set through {@link useLocale}. Without an
 * explicit locale the browser language is used, falling back to `"en"`
 * (always the case on the server). Invalid options surface as the
 * platform's `RangeError`/`TypeError` on first read. No listeners or timers
 * are created, so nothing needs cleanup.
 *
 * @example
 * ```ts
 * const price = ref(1234.5);
 * const { formatted } = useNumberFormat(price, { locale: "de-DE", style: "currency", currency: "EUR" });
 * formatted.value; // "1.234,50 €"
 * ```
 *
 * @param value Reactive number; `null`/`undefined` format as `""`.
 * @param options Reactive locale and `Intl.NumberFormat` options.
 * @default value undefined
 * @default options {}
 * @returns Reactive formatted output and formatter helpers.
 */
export function useNumberFormat(
  value?: MaybeRefOrGetter<FormattableNumber | null | undefined>,
  options: MaybeRefOrGetter<UseNumberFormatOptions> = {},
): NumberFormatControls {
  const localeControls = useLocale(() => toValue(options).locale, { detect: browserLanguage });
  const formatter = computed(() => {
    const { locale: _locale, ...formatOptions } = toValue(options);
    return localeControls.number(formatOptions);
  });
  const formatted = computed(() => {
    const current = value === undefined ? undefined : toValue(value);
    return current === null || current === undefined ? "" : formatter.value.format(current);
  });
  return {
    locale: localeControls.locale,
    formatter,
    formatted,
    format: (input) => formatter.value.format(input),
    formatToParts: (input) => formatter.value.formatToParts(input),
    formatRange: (start, end) => formatRangeWith(formatter.value, start, end),
  };
}

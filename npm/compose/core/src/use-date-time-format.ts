import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useLocale } from "./locale.ts";

/** Values accepted by `Intl.DateTimeFormat`: a `Date` or Unix milliseconds. */
export type FormattableDate = Date | number;

/** Options for {@link useDateTimeFormat}. */
export interface UseDateTimeFormatOptions extends Intl.DateTimeFormatOptions {
  /**
   * Locale used for formatting. Pass it explicitly (and a `timeZone`) for
   * server rendering: the server falls back to `"en"` and its own time zone.
   *
   * @default the browser language, otherwise "en"
   */
  readonly locale?: string | Intl.Locale;
}

/** Reactive date formatting returned by {@link useDateTimeFormat}. */
export interface DateTimeFormatControls {
  /** Canonical locale in use. */
  readonly locale: ComputedRef<string>;

  /** Cached formatter for the current locale and options. */
  readonly formatter: ComputedRef<Intl.DateTimeFormat>;

  /**
   * Formatted `value`, or `""` while the value is `null`/`undefined` or an
   * invalid date.
   */
  readonly formatted: ComputedRef<string>;

  /**
   * Format any date with the current formatter.
   *
   * @param value Date or Unix milliseconds.
   * @returns The formatted string.
   * @throws {RangeError} For an invalid date, as `Intl.DateTimeFormat` does.
   */
  readonly format: (value: FormattableDate) => string;

  /**
   * Format any date into locale-aware parts.
   *
   * @param value Date or Unix milliseconds.
   * @returns The formatted parts.
   */
  readonly formatToParts: (value: FormattableDate) => Intl.DateTimeFormatPart[];

  /**
   * Format a date range, collapsing shared fields (for example `"Jan 3 – 5, 2026"`).
   *
   * @param start Range start.
   * @param end Range end.
   * @returns The formatted range.
   */
  readonly formatRange: (start: FormattableDate, end: FormattableDate) => string;
}

function browserLanguage(): string | undefined {
  return typeof window === "undefined" ? undefined : window.navigator.language;
}

function isValidDate(value: FormattableDate): boolean {
  return !Number.isNaN(value instanceof Date ? value.getTime() : value);
}

/**
 * Format dates and times reactively with `Intl.DateTimeFormat`.
 *
 * The formatter follows the reactive options (including `locale`) and is
 * cached per locale and option set through {@link useLocale}. For
 * hydration-stable output pass both `locale` and `timeZone`: otherwise the
 * server uses `"en"` and its own zone. Invalid options surface as the
 * platform's `RangeError` on first read; invalid dates render as `""`.
 * No listeners or timers are created — combine with a clock composable for
 * live-updating output.
 *
 * @example
 * ```ts
 * const { formatted } = useDateTimeFormat(date, { locale: "en-GB", dateStyle: "long", timeZone: "UTC" });
 * ```
 *
 * @param value Reactive date; `null`/`undefined` format as `""`.
 * @param options Reactive locale and `Intl.DateTimeFormat` options.
 * @default value undefined
 * @default options {}
 * @returns Reactive formatted output and formatter helpers.
 */
export function useDateTimeFormat(
  value?: MaybeRefOrGetter<FormattableDate | null | undefined>,
  options: MaybeRefOrGetter<UseDateTimeFormatOptions> = {},
): DateTimeFormatControls {
  const localeControls = useLocale(() => toValue(options).locale, { detect: browserLanguage });
  const formatter = computed(() => {
    const { locale: _locale, ...formatOptions } = toValue(options);
    return localeControls.dateTime(formatOptions);
  });
  const formatted = computed(() => {
    const current = value === undefined ? undefined : toValue(value);
    if (current === null || current === undefined || !isValidDate(current)) return "";
    return formatter.value.format(current);
  });
  return {
    locale: localeControls.locale,
    formatter,
    formatted,
    format: (input) => formatter.value.format(input),
    formatToParts: (input) => formatter.value.formatToParts(input),
    formatRange: (start, end) => formatter.value.formatRange(start, end),
  };
}

import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useLocale } from "./locale.ts";

/** Units selected by {@link selectRelativeTimeUnit}. */
export type RelativeTimeUnitSelection =
  | "second"
  | "minute"
  | "hour"
  | "day"
  | "week"
  | "month"
  | "year";

/** A signed amount in one relative-time unit. */
export interface RelativeTimeValue {
  /** Signed, rounded amount; negative values are in the past. */
  readonly value: number;

  /** Unit the amount is expressed in. */
  readonly unit: RelativeTimeUnitSelection;
}

/** Options for {@link useRelativeTimeFormat}. */
export interface UseRelativeTimeFormatOptions extends Intl.RelativeTimeFormatOptions {
  /**
   * Locale used for formatting. Pass it explicitly for hydration-stable
   * server rendering.
   *
   * @default the browser language, otherwise "en"
   */
  readonly locale?: string | Intl.Locale;
}

/** Options for {@link RelativeTimeFormatControls.formatFrom}. */
export interface RelativeTimeFromOptions {
  /**
   * Reference time in Unix milliseconds. Pass it explicitly to keep server
   * and client output identical.
   *
   * @default Date.now()
   */
  readonly now?: number;
}

/** Reactive relative-time formatting returned by {@link useRelativeTimeFormat}. */
export interface RelativeTimeFormatControls {
  /** Canonical locale in use. */
  readonly locale: ComputedRef<string>;

  /** Cached formatter for the current locale and options. */
  readonly formatter: ComputedRef<Intl.RelativeTimeFormat>;

  /** Formatted `value` in `unit`, or `""` while either is missing. */
  readonly formatted: ComputedRef<string>;

  /**
   * Format an amount in a unit.
   *
   * @param value Signed amount; negative is in the past.
   * @param unit Relative-time unit.
   * @returns The formatted string.
   */
  readonly format: (value: number, unit: Intl.RelativeTimeFormatUnit) => string;

  /**
   * Format an amount in a unit into parts.
   *
   * @param value Signed amount.
   * @param unit Relative-time unit.
   * @returns The formatted parts.
   */
  readonly formatToParts: (
    value: number,
    unit: Intl.RelativeTimeFormatUnit,
  ) => Intl.RelativeTimeFormatPart[];

  /**
   * Format a signed millisecond difference using the best-fitting unit
   * (see {@link selectRelativeTimeUnit}).
   *
   * @param diffMs Signed difference; negative is in the past.
   * @returns The formatted string.
   */
  readonly formatDiff: (diffMs: number) => string;

  /**
   * Format a date relative to a reference time using the best-fitting unit.
   *
   * @param date Date or Unix milliseconds.
   * @param options Reference time.
   * @default options {}
   * @returns The formatted string.
   */
  readonly formatFrom: (date: Date | number, options?: RelativeTimeFromOptions) => string;
}

// Literal millisecond values keep module evaluation free of computation, so
// bundlers can drop this module entirely when it is unused.
const SECOND = 1000;
const MINUTE = 60_000;
const HOUR = 3_600_000;
const DAY = 86_400_000;
const WEEK = 604_800_000;
/** Gregorian average month: 30.436875 days. */
const MONTH = 2_629_746_000;
/** Gregorian average year: 365.2425 days. */
const YEAR = 31_556_952_000;

const thresholds: readonly (readonly [number, number, RelativeTimeUnitSelection])[] = [
  [MINUTE, SECOND, "second"],
  [HOUR, MINUTE, "minute"],
  [DAY, HOUR, "hour"],
  [WEEK, DAY, "day"],
  [MONTH, WEEK, "week"],
  [YEAR, MONTH, "month"],
];

/**
 * Pick the largest unit that expresses a millisecond difference as a
 * non-zero rounded amount: seconds below a minute, minutes below an hour,
 * hours below a day, days below a week, weeks below a month, months below
 * a year, and years otherwise (Gregorian average month/year lengths).
 *
 * @param diffMs Signed difference; negative is in the past.
 * @returns The rounded amount and its unit.
 * @throws {RangeError} `[VIZE_COMPOSE_RELATIVE_TIME_INVALID_DIFF]` for a
 * non-finite difference.
 */
export function selectRelativeTimeUnit(diffMs: number): RelativeTimeValue {
  if (!Number.isFinite(diffMs)) {
    throw new RangeError(
      `[VIZE_COMPOSE_RELATIVE_TIME_INVALID_DIFF] the difference must be finite; received ${String(diffMs)}`,
    );
  }
  const magnitude = Math.abs(diffMs);
  for (const [limit, size, unit] of thresholds) {
    if (magnitude < limit) return { value: Math.round(diffMs / size) + 0, unit };
  }
  return { value: Math.round(diffMs / YEAR) + 0, unit: "year" };
}

function browserLanguage(): string | undefined {
  return typeof window === "undefined" ? undefined : window.navigator.language;
}

/**
 * Format relative times ("in 3 days", "yesterday") with `Intl.RelativeTimeFormat`.
 *
 * A pure formatting wrapper: it owns no clock or timer, so it never
 * re-renders on its own and stays deterministic during server rendering
 * (pair it with a clock composable for live "time ago" labels). The
 * formatter follows the reactive options and is cached per locale and
 * option set through {@link useLocale}; invalid options surface as the
 * platform's `RangeError`.
 *
 * @example
 * ```ts
 * const { formatted } = useRelativeTimeFormat(-1, "day", { locale: "en", numeric: "auto" });
 * formatted.value; // "yesterday"
 * ```
 *
 * @param value Reactive signed amount; `null`/`undefined` format as `""`.
 * @param unit Reactive unit.
 * @param options Reactive locale and `Intl.RelativeTimeFormat` options.
 * @default options {}
 * @returns Reactive formatted output and formatting helpers.
 */
export function useRelativeTimeFormat(
  value: MaybeRefOrGetter<number | null | undefined>,
  unit: MaybeRefOrGetter<Intl.RelativeTimeFormatUnit>,
  options: MaybeRefOrGetter<UseRelativeTimeFormatOptions> = {},
): RelativeTimeFormatControls {
  const localeControls = useLocale(() => toValue(options).locale, { detect: browserLanguage });
  const formatter = computed(() => {
    const { locale: _locale, ...formatOptions } = toValue(options);
    return localeControls.relativeTime(formatOptions);
  });
  const formatDiff = (diffMs: number): string => {
    const selected = selectRelativeTimeUnit(diffMs);
    return formatter.value.format(selected.value, selected.unit);
  };
  return {
    locale: localeControls.locale,
    formatter,
    formatted: computed(() => {
      const current = toValue(value);
      return current === null || current === undefined
        ? ""
        : formatter.value.format(current, toValue(unit));
    }),
    format: (amount, amountUnit) => formatter.value.format(amount, amountUnit),
    formatToParts: (amount, amountUnit) => formatter.value.formatToParts(amount, amountUnit),
    formatDiff,
    formatFrom: (date, fromOptions = {}) => {
      const time = date instanceof Date ? date.getTime() : date;
      return formatDiff(time - (fromOptions.now ?? Date.now()));
    },
  };
}

import { toUtcDate } from "./plain-date.ts";
import type { PlainDate, PlainYearMonth, Weekday } from "./plain-date.ts";

/** Width of weekday labels rendered in calendar column headers. */
export type CalendarWeekdayFormat = "narrow" | "short" | "long";

/** Localized label for one calendar column. */
export interface CalendarWeekdayLabel {
  /** Day of week, where `0` is Sunday. */
  readonly weekday: Weekday;

  /** Visible column label in the requested width. */
  readonly label: string;

  /** Full weekday name for `abbr` and assistive technology. */
  readonly longLabel: string;
}

/** Locale inputs shared by calendar formatters. */
export interface CalendarLocaleOptions {
  /** BCP 47 locale. */
  readonly locale: string;

  /**
   * Intl calendar used for display only (for example `japanese` or `buddhist`).
   * Grid structure always follows ISO months.
   */
  readonly calendar?: string | undefined;

  /** Intl numbering system for day numbers and years. */
  readonly numberingSystem?: string | undefined;
}

/** Memoized Intl formatters for one locale configuration. */
export interface CalendarFormatters {
  /** Day-of-month label shown inside a cell. */
  readonly day: (date: PlainDate) => string;

  /** Full accessible date label, for example `Friday, September 25, 2026`. */
  readonly fullDate: (date: PlainDate) => string;

  /** Month and year label, for example `September 2026`. */
  readonly monthYear: (month: PlainYearMonth) => string;

  /** Month range label for multi-month views, for example `September – October 2026`. */
  readonly monthRange: (first: PlainYearMonth, last: PlainYearMonth) => string;

  /** Standalone month name used by month pickers. */
  readonly monthName: (month: number) => string;

  /** Year label used by year pickers. */
  readonly year: (year: number) => string;

  /** Weekday labels in column order. */
  readonly weekdays: (
    weekStartsOn: Weekday,
    format: CalendarWeekdayFormat,
  ) => readonly CalendarWeekdayLabel[];
}

/*
 * First day of week per CLDR `weekData`. The table is embedded instead of
 * reading `Intl.Locale#getWeekInfo()` because that API is missing from some
 * engines; a shared table keeps server and browser output byte-identical.
 */
const sundayFirstRegions = new Set(
  "AG AS AU BD BR BS BT BW BZ CA CN CO DM DO ET GT GU HK HN ID IL IN JM JP KE KH KR LA MH MM MO MT MX MZ NI NP PA PE PH PK PR PT PY SA SG SV TH TT TW UM US VE VI WS YE ZA ZW".split(
    " ",
  ),
);
const saturdayFirstRegions = new Set("AE AF BH DJ DZ EG IQ IR JO KW LY OM QA SD SY".split(" "));
const fridayFirstRegions = new Set(["MV"]);
const firstDayKeywords: Readonly<Record<string, Weekday>> = {
  sun: 0,
  mon: 1,
  tue: 2,
  wed: 3,
  thu: 4,
  fri: 5,
  sat: 6,
};
const sundayReference: PlainDate = Object.freeze({ year: 2023, month: 1, day: 1 });

/**
 * Locale-preferred first day of week.
 *
 * Honors the `-u-fw-` Unicode extension (for example `en-US-u-fw-mon`) and
 * otherwise resolves the likely region from CLDR week data, defaulting to Monday.
 */
export function resolveWeekStart(locale: string): Weekday {
  try {
    const firstDay = /-u(?:-[a-z0-9]{2,8})*?-fw-([a-z]{3})/iu.exec(locale)?.[1]?.toLowerCase();
    const keyword = firstDay === undefined ? undefined : firstDayKeywords[firstDay];
    if (keyword !== undefined) return keyword;
    const region = new Intl.Locale(locale).maximize().region ?? "";
    if (sundayFirstRegions.has(region)) return 0;
    if (saturdayFirstRegions.has(region)) return 6;
    if (fridayFirstRegions.has(region)) return 5;
  } catch {
    // Invalid locale tags use the ISO 8601 default.
  }
  return 1;
}

/** Normalize a consumer week start, falling back to the locale preference. */
export function normalizeWeekday(value: number | null | undefined, locale: string): Weekday {
  if (typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= 6) {
    return value as Weekday;
  }
  return resolveWeekStart(locale);
}

function createFormatter(
  options: CalendarLocaleOptions,
  format: Intl.DateTimeFormatOptions,
): Intl.DateTimeFormat {
  const base: Intl.DateTimeFormatOptions = { ...format, timeZone: "UTC" };
  if (options.calendar) base.calendar = options.calendar;
  if (options.numberingSystem) base.numberingSystem = options.numberingSystem;
  try {
    return new Intl.DateTimeFormat(options.locale, base);
  } catch {
    return new Intl.DateTimeFormat("en-US", { ...format, timeZone: "UTC" });
  }
}

/**
 * Create Intl formatters for calendar labels.
 *
 * Dates are formatted as UTC midnight with `timeZone: "UTC"`, so labels never
 * depend on the host time zone.
 */
export function createCalendarFormatters(options: CalendarLocaleOptions): CalendarFormatters {
  const dayFormatter = createFormatter(options, { day: "numeric" });
  const fullFormatter = createFormatter(options, { dateStyle: "full" });
  const monthYearFormatter = createFormatter(options, { month: "long", year: "numeric" });
  const monthFormatter = createFormatter(options, { month: "long" });
  const yearFormatter = createFormatter(options, { year: "numeric" });
  const weekdayFormatters = new Map<CalendarWeekdayFormat | "full", Intl.DateTimeFormat>();
  const weekdayFormatter = (format: CalendarWeekdayFormat | "full") => {
    let formatter = weekdayFormatters.get(format);
    if (!formatter) {
      formatter = createFormatter(options, { weekday: format === "full" ? "long" : format });
      weekdayFormatters.set(format, formatter);
    }
    return formatter;
  };
  const firstOfMonth = (month: PlainYearMonth) =>
    toUtcDate({ year: month.year, month: month.month, day: 1 });

  return {
    day: (date) => dayFormatter.format(toUtcDate(date)),
    fullDate: (date) => fullFormatter.format(toUtcDate(date)),
    monthYear: (month) => monthYearFormatter.format(firstOfMonth(month)),
    monthRange: (first, last) =>
      first.year === last.year && first.month === last.month
        ? monthYearFormatter.format(firstOfMonth(first))
        : monthYearFormatter.formatRange(firstOfMonth(first), firstOfMonth(last)),
    monthName: (month) => monthFormatter.format(toUtcDate({ year: 2001, month, day: 1 })),
    year: (year) => yearFormatter.format(toUtcDate({ year, month: 7, day: 1 })),
    weekdays: (weekStartsOn, format) =>
      Array.from({ length: 7 }, (_, offset): CalendarWeekdayLabel => {
        const weekday = ((weekStartsOn + offset) % 7) as Weekday;
        const date = toUtcDate({ ...sundayReference, day: sundayReference.day + weekday });
        return {
          weekday,
          label: weekdayFormatter(format).format(date),
          longLabel: weekdayFormatter("full").format(date),
        };
      }),
  };
}

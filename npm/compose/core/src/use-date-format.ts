import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** Date input accepted by {@link formatDate} and {@link useDateFormat}. */
export type DateFormatInput = Date | number | string;

/**
 * Tokens understood by the pattern form of {@link formatDate}.
 *
 * | Token | Output | Token | Output |
 * | --- | --- | --- | --- |
 * | `YYYY` | 2026 | `YY` | 26 |
 * | `M` / `MM` | 1 / 01 | `MMM` / `MMMM` | Jan / January (localized) |
 * | `D` / `DD` | 5 / 05 | `d` | 0–6, Sunday is 0 |
 * | `dd` / `ddd` / `dddd` | S / Sun / Sunday (localized) | `H` / `HH` | 0–23 |
 * | `h` / `hh` | 1–12 | `m` / `mm`, `s` / `ss` | minutes, seconds |
 * | `SSS` | milliseconds | `A` / `a` | AM / am (localized) |
 * | `Z` / `ZZ` | +09:00 / +0900 | `[text]` | literal text |
 */
export type DateFormatToken =
  | "YYYY"
  | "YY"
  | "M"
  | "MM"
  | "MMM"
  | "MMMM"
  | "D"
  | "DD"
  | "d"
  | "dd"
  | "ddd"
  | "dddd"
  | "H"
  | "HH"
  | "h"
  | "hh"
  | "m"
  | "mm"
  | "s"
  | "ss"
  | "SSS"
  | "A"
  | "a"
  | "Z"
  | "ZZ";

/** Options for {@link formatDate}. */
export interface FormatDateOptions {
  /**
   * BCP 47 locale(s) for names and `Intl` option formats. Defaults to a fixed
   * locale (not the host default) so server and client output agree.
   *
   * @default "en"
   */
  readonly locale?: string | readonly string[];

  /**
   * IANA time zone the date is rendered in. Defaults to UTC (not the host
   * zone) for SSR determinism; pass the user's zone explicitly.
   *
   * @default "UTC"
   */
  readonly timeZone?: string;
}

/** Options for {@link useDateFormat}. */
export interface UseDateFormatOptions {
  /**
   * Reactive BCP 47 locale(s).
   *
   * @default "en"
   */
  readonly locale?: MaybeRefOrGetter<string | readonly string[]>;

  /**
   * Reactive IANA time zone.
   *
   * @default "UTC"
   */
  readonly timeZone?: MaybeRefOrGetter<string>;
}

const tokenPattern =
  /\[([^\]]*)]|YYYY|YY|MMMM|MMM|MM|M|DD|D|dddd|ddd|dd|d|HH|H|hh|h|mm|m|ss|s|SSS|A|a|ZZ|Z/g;

const weekdayIndex: Readonly<Record<string, number>> = {
  Sun: 0,
  Mon: 1,
  Tue: 2,
  Wed: 3,
  Thu: 4,
  Fri: 5,
  Sat: 6,
};

function toDate(input: DateFormatInput): Date {
  const date = input instanceof Date ? new Date(input.getTime()) : new Date(input);
  if (Number.isNaN(date.getTime())) {
    throw new RangeError(
      `[VIZE_COMPOSE_DATE_FORMAT_INVALID_DATE] expected a valid date; received ${String(input)}`,
    );
  }
  return date;
}

function pad(value: number, length: number): string {
  return String(value).padStart(length, "0");
}

function partsOf(formatter: Intl.DateTimeFormat, date: Date): Map<string, string> {
  return new Map(formatter.formatToParts(date).map((part) => [part.type, part.value]));
}

/**
 * Format a date with a token pattern or `Intl.DateTimeFormat` options.
 *
 * Pure and deterministic: locale and time zone default to fixed values
 * (`"en"`, `"UTC"`) rather than host settings, so the same input renders
 * identically on server and client. Calendar fields are computed in the
 * requested time zone through `Intl`, so no host-zone arithmetic leaks in.
 *
 * @example
 * ```ts
 * formatDate(0, "YYYY-MM-DD HH:mm:ss"); // "1970-01-01 00:00:00"
 * formatDate(0, "dddd, MMMM D [at] h:mm A", { locale: "en", timeZone: "Asia/Tokyo" });
 * formatDate(0, { dateStyle: "long" }, { locale: "ja" }); // "1970年1月1日"
 * ```
 *
 * @param input Date to format.
 * @param format Token pattern (see {@link DateFormatToken}) or `Intl` options.
 * @param options Locale and time zone.
 * @default format "HH:mm:ss"
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_DATE_FORMAT_INVALID_DATE` for
 * invalid dates, and `RangeError` from `Intl` for invalid locales or zones.
 * @returns The formatted string.
 */
export function formatDate(
  input: DateFormatInput,
  format: string | Intl.DateTimeFormatOptions = "HH:mm:ss",
  options: FormatDateOptions = {},
): string {
  const date = toDate(input);
  const locales =
    typeof options.locale === "string" ? options.locale : [...(options.locale ?? ["en"])];
  const timeZone = options.timeZone ?? "UTC";

  if (typeof format !== "string") {
    return new Intl.DateTimeFormat(locales, { ...format, timeZone }).format(date);
  }

  const numeric = partsOf(
    new Intl.DateTimeFormat("en-US", {
      timeZone,
      hourCycle: "h23",
      year: "numeric",
      month: "numeric",
      day: "numeric",
      hour: "numeric",
      minute: "numeric",
      second: "numeric",
      weekday: "short",
      timeZoneName: "longOffset",
    }),
    date,
  );
  const field = (type: string): number => Number(numeric.get(type) ?? "0");
  const localized = (formatOptions: Intl.DateTimeFormatOptions): string =>
    new Intl.DateTimeFormat(locales, { ...formatOptions, timeZone }).format(date);

  const year = field("year");
  const month = field("month");
  const day = field("day");
  const hour = field("hour") % 24;
  const minute = field("minute");
  const second = field("second");
  const offset = (numeric.get("timeZoneName") ?? "GMT").replace("GMT", "") || "+00:00";

  const meridiem = (): string =>
    partsOf(
      new Intl.DateTimeFormat(locales, { hour: "numeric", hourCycle: "h12", timeZone }),
      date,
    ).get("dayPeriod") ?? (hour < 12 ? "AM" : "PM");

  const render = (token: string): string => {
    switch (token) {
      case "YYYY":
        return pad(year, 4);
      case "YY":
        return pad(year % 100, 2);
      case "M":
        return String(month);
      case "MM":
        return pad(month, 2);
      case "MMM":
        return localized({ month: "short" });
      case "MMMM":
        return localized({ month: "long" });
      case "D":
        return String(day);
      case "DD":
        return pad(day, 2);
      case "d":
        return String(weekdayIndex[numeric.get("weekday") ?? "Sun"] ?? 0);
      case "dd":
        return localized({ weekday: "narrow" });
      case "ddd":
        return localized({ weekday: "short" });
      case "dddd":
        return localized({ weekday: "long" });
      case "H":
        return String(hour);
      case "HH":
        return pad(hour, 2);
      case "h":
        return String(hour % 12 || 12);
      case "hh":
        return pad(hour % 12 || 12, 2);
      case "m":
        return String(minute);
      case "mm":
        return pad(minute, 2);
      case "s":
        return String(second);
      case "ss":
        return pad(second, 2);
      case "SSS":
        return pad(date.getUTCMilliseconds(), 3);
      case "A":
        return meridiem();
      case "a":
        return meridiem().toLowerCase();
      case "Z":
        return offset;
      default:
        return offset.replace(":", "");
    }
  };

  return format.replaceAll(
    tokenPattern,
    (match: string, literal: string | undefined) => literal ?? render(match),
  );
}

/**
 * Reactive formatted date.
 *
 * Wraps {@link formatDate} in a computed ref; the date, the format, the
 * locale, and the time zone are all reactive. Deterministic defaults (`"en"`,
 * `"UTC"`) keep server and client output identical. No timers, no globals,
 * nothing to dispose.
 *
 * @example
 * ```ts
 * const label = useDateFormat(() => event.value.startsAt, "ddd, MMM D HH:mm", {
 *   locale: () => locale.value,
 *   timeZone: "Europe/Paris",
 * });
 * ```
 *
 * @param input Reactive date.
 * @param format Reactive token pattern or `Intl` options.
 * @param options Reactive locale and time zone.
 * @default format "HH:mm:ss"
 * @default options {}
 * @throws `RangeError` on read for invalid dates, locales, or zones.
 * @returns Computed formatted string.
 */
export function useDateFormat(
  input: MaybeRefOrGetter<DateFormatInput>,
  format: MaybeRefOrGetter<string | Intl.DateTimeFormatOptions> = "HH:mm:ss",
  options: UseDateFormatOptions = {},
): ComputedRef<string> {
  return computed(() =>
    formatDate(toValue(input), toValue(format), {
      locale: toValue(options.locale) ?? "en",
      timeZone: toValue(options.timeZone) ?? "UTC",
    }),
  );
}

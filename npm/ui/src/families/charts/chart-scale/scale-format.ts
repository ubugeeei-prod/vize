import { precisionFixed, tickStep } from "./scale-ticks.ts";
import type {
  NumberTickFormatOptions,
  TimeTickFormatOptions,
  TimeTickGranularity,
} from "./scale-types.ts";

const defaultLocale = "en-US";

/**
 * Build a localized formatter for ticks of `[start, stop]`.
 *
 * Precision follows d3's `precisionFixed(tickStep)`, so adjacent ticks are
 * always distinguishable without trailing noise; with the default `en-US`
 * locale the output equals d3's `",f"` format except that `Intl` uses an ASCII
 * hyphen-minus rather than `−`.
 */
export function numberTickFormat(
  start: number,
  stop: number,
  count: number,
  options: NumberTickFormatOptions = {},
): (value: number) => string {
  const step = tickStep(start, stop, count);
  const percent = options.format?.style === "percent";
  const fixed = Number.isFinite(step) ? precisionFixed(step) : 0;
  const digits = Math.min(20, Math.max(0, percent ? fixed - 2 : fixed));
  const formatter = new Intl.NumberFormat(options.locale ?? defaultLocale, {
    maximumFractionDigits: digits,
    minimumFractionDigits: digits,
    ...options.format,
  });
  return (value) => formatter.format(value);
}

/** Default `Intl.DateTimeFormat` options per time tick granularity. */
export const defaultTimeTickFormats: Readonly<
  Record<TimeTickGranularity, Intl.DateTimeFormatOptions>
> = Object.freeze({
  millisecond: { second: "2-digit", fractionalSecondDigits: 3 },
  second: { hour: "numeric", minute: "2-digit", second: "2-digit" },
  minute: { hour: "numeric", minute: "2-digit" },
  hour: { hour: "numeric" },
  day: { weekday: "short", day: "numeric" },
  week: { month: "short", day: "numeric" },
  month: { month: "long" },
  year: { year: "numeric" },
});

/**
 * Build a granularity-aware time formatter. `granularity` decides which calendar
 * unit a tick represents (as d3's multi-scale time format does); each unit is
 * formatted with `Intl.DateTimeFormat` in `timeZone`, so labels are localized
 * and identical on server and client.
 */
export function timeTickFormat(
  timeZone: string,
  granularity: (date: Date) => TimeTickGranularity,
  options: TimeTickFormatOptions = {},
): (date: Date) => string {
  const locale = options.locale ?? defaultLocale;
  const cache = new Map<TimeTickGranularity, Intl.DateTimeFormat>();
  const formatterFor = (unit: TimeTickGranularity): Intl.DateTimeFormat => {
    const cached = cache.get(unit);
    if (cached !== undefined) return cached;
    const formatter = new Intl.DateTimeFormat(locale, {
      ...(options.formats?.[unit] ?? defaultTimeTickFormats[unit]),
      timeZone,
    });
    cache.set(unit, formatter);
    return formatter;
  };
  return (date) => formatterFor(granularity(date)).format(date);
}

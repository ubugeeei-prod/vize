import { precisionFixed } from "../chart-scale/chart-scale.ts";
import type { TimeScale } from "../chart-scale/chart-scale.ts";
import type { ChartAxisScale } from "./chart-types.ts";

/** Whether a scale is a chart-scale time scale, so labels can use calendar formatting. */
export function isTimeScale(scale: unknown): scale is TimeScale {
  return typeof scale === "function" && "kind" in scale && scale.kind === "time";
}

/**
 * Default axis label formatter.
 *
 * Time scales use their zone-aware `Intl` tick format; numeric ticks use
 * `Intl.NumberFormat` with just enough fraction digits to tell adjacent ticks
 * apart; every other value is stringified.
 */
export function defaultTickFormatter<Tick>(
  scale: ChartAxisScale<Tick>,
  ticks: readonly Tick[],
  locale: string,
): (tick: Tick) => string {
  if (isTimeScale(scale)) {
    const format = scale.tickFormat({ locale });
    return (tick) => (tick instanceof Date ? format(tick) : String(tick));
  }
  const numbers = ticks.filter((tick): tick is Tick & number => typeof tick === "number");
  if (numbers.length > 0) {
    const first = numbers[0] ?? 0;
    const second = numbers[1];
    const step = second === undefined ? first : Math.abs(second - first);
    const digits = step === 0 || !Number.isFinite(step) ? 0 : Math.min(20, precisionFixed(step));
    const formatter = new Intl.NumberFormat(locale, {
      maximumFractionDigits: digits,
      minimumFractionDigits: digits,
    });
    return (tick) => (typeof tick === "number" ? formatter.format(tick) : String(tick));
  }
  return (tick) => (tick instanceof Date ? tick.toISOString() : String(tick));
}

/** Default cell formatter for the accessible data table. */
export function defaultCellFormatter(
  locale: string,
): (value: string | number | Date | null | undefined) => string {
  const numbers = new Intl.NumberFormat(locale);
  const dates = new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeZone: "UTC" });
  return (value) => {
    if (value === null || value === undefined) return "";
    if (typeof value === "number") return numbers.format(value);
    if (value instanceof Date) return dates.format(value);
    return value;
  };
}

import { createContinuousCore } from "./scale-continuous.ts";
import { timeTickFormat } from "./scale-format.ts";
import { timeTickInterval, zoneIntervals } from "./scale-time-interval.ts";
import { tickStep } from "./scale-ticks.ts";
import type {
  TimeScale,
  TimeScaleOptions,
  TimeTickFormatOptions,
  TimeTickGranularity,
} from "./scale-types.ts";

const defaultDomain: readonly number[] = Object.freeze([
  Date.UTC(2000, 0, 1),
  Date.UTC(2000, 0, 2),
]);
const unitRange: readonly number[] = Object.freeze([0, 1]);

function epochOf(value: Date | number | null | undefined): number {
  if (value === null || value === undefined) return Number.NaN;
  return value instanceof Date ? value.getTime() : +value;
}

function identity(value: number): number {
  return value;
}

/**
 * Create an immutable, calendar-aware time scale.
 *
 * Ticks, nice domains, and labels are computed in `timeZone` (default `"UTC"`),
 * never in the host's local zone, so server rendering and hydration agree. With
 * `"UTC"` the ticks equal d3's `scaleUtc`; with an IANA zone they follow the
 * same calendar rules as d3's local `scaleTime` would in that zone.
 *
 * @example
 * ```ts
 * const x = scaleTime({ domain: [start, end], range: [0, 640], timeZone: "Asia/Tokyo" });
 * const format = x.tickFormat({ locale: "ja-JP" });
 * x.ticks(6).map(format);
 * ```
 */
export function scaleTime(options: TimeScaleOptions = {}): TimeScale {
  const intervals = zoneIntervals(options.timeZone ?? "UTC");
  const timeZone = intervals.clock.timeZone;
  let domain = Object.freeze((options.domain ?? defaultDomain).map(epochOf));
  const niceCount =
    options.nice === true ? 10 : options.nice === false ? null : (options.nice ?? null);
  if (niceCount !== null) {
    const first = domain[0] ?? 0;
    const last = domain.at(-1) ?? 0;
    const interval = timeTickInterval(intervals, first, last, niceCount, tickStep);
    if (interval !== null) {
      const next = [...domain];
      const [i0, i1] = last < first ? [next.length - 1, 0] : [0, next.length - 1];
      next[i0] = interval.floor(Math.min(first, last));
      next[i1] = interval.ceil(Math.max(first, last));
      domain = Object.freeze(next);
    }
  }
  const range = Object.freeze([...(options.range ?? unitRange)]);
  const core = createContinuousCore({
    clamp: options.clamp ?? false,
    domain,
    range,
    round: options.round ?? false,
    transform: identity,
    unknown: options.unknown ?? Number.NaN,
    untransform: identity,
  });
  const current: TimeScaleOptions = { ...options, domain, range, nice: false, timeZone };
  const first = domain[0] ?? 0;
  const last = domain.at(-1) ?? 0;

  const tickGranularity = (date: Date): TimeTickGranularity => {
    const t = date.getTime();
    if (intervals.second.floor(t) < t) return "millisecond";
    if (intervals.minute.floor(t) < t) return "second";
    if (intervals.hour.floor(t) < t) return "minute";
    if (intervals.day.floor(t) < t) return "hour";
    if (intervals.month.floor(t) < t) return intervals.week.floor(t) < t ? "day" : "week";
    if (intervals.year.floor(t) < t) return "month";
    return "year";
  };

  const scale = Object.assign(
    (value: Date | number | null | undefined) => core.map(epochOf(value)),
    {
      kind: "time" as const,
      timeZone,
      range,
      clamp: options.clamp ?? false,
      round: options.round ?? false,
      invert: (value: number) => new Date(core.invert(value)),
      ticks: (count = 10) => {
        const reverse = last < first;
        const [start, stop] = reverse ? [last, first] : [first, last];
        const interval = timeTickInterval(intervals, start, stop, count, tickStep);
        const values = interval === null ? [] : interval.range(start, stop + 1);
        const dates = values.map((epoch) => new Date(epoch));
        return reverse ? dates.reverse() : dates;
      },
      tickGranularity,
      tickFormat: (format: TimeTickFormatOptions = {}) =>
        timeTickFormat(timeZone, tickGranularity, format),
      nice: (count = 10) => scaleTime({ ...current, nice: count }),
      with: (next: TimeScaleOptions) => scaleTime({ ...current, ...next }),
      options: () => ({ ...current }),
    },
  ) as TimeScale;
  // Date is mutable even in a readonly array. Keep the epoch snapshot private
  // so changing a returned domain value cannot change later copies or `with()`.
  Object.defineProperty(scale, "domain", {
    enumerable: true,
    get: () => Object.freeze(domain.map((epoch) => new Date(epoch))),
  });
  return scale;
}

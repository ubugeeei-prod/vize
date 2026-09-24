// Calendar intervals mirror d3-time 3 (ISC licensed), but run against an explicit
// time zone instead of the host's local zone. Each epoch timestamp is viewed as a
// "wall" timestamp (the zone's local clock reading encoded as UTC milliseconds),
// floored or offset on the wall clock, then converted back. With `"UTC"` the
// conversion is the identity and results equal d3's UTC intervals exactly.

const second = 1000;
const minute = second * 60;
const hour = minute * 60;
const day = hour * 24;
const week = day * 7;
const month = day * 30;
const year = day * 365;

/** Epoch ↔ wall-clock conversion for one IANA time zone. */
export interface ZoneClock {
  readonly timeZone: string;
  readonly utc: boolean;
  readonly toWall: (epoch: number) => number;
  readonly fromWall: (wall: number) => number;
}

const partFormatters = new Map<string, Intl.DateTimeFormat>();

function partsFormatter(timeZone: string): Intl.DateTimeFormat {
  const cached = partFormatters.get(timeZone);
  if (cached !== undefined) return cached;
  const formatter = new Intl.DateTimeFormat("en-US", {
    timeZone,
    hourCycle: "h23",
    year: "numeric",
    month: "numeric",
    day: "numeric",
    hour: "numeric",
    minute: "numeric",
    second: "numeric",
    era: "short",
  });
  partFormatters.set(timeZone, formatter);
  return formatter;
}

function zoneOffset(formatter: Intl.DateTimeFormat, epoch: number): number {
  const fields: Record<string, number> = {};
  let bce = false;
  for (const part of formatter.formatToParts(epoch)) {
    if (part.type === "era") bce = part.value.startsWith("B");
    else if (part.type !== "literal") fields[part.type] = Number(part.value);
  }
  const yearValue = fields.year ?? 1970;
  const wall = Date.UTC(
    bce ? 1 - yearValue : yearValue,
    (fields.month ?? 1) - 1,
    fields.day ?? 1,
    fields.hour ?? 0,
    fields.minute ?? 0,
    fields.second ?? 0,
  );
  const whole = Math.floor(epoch / second) * second;
  return wall - whole;
}

/**
 * Create a wall-clock converter for `timeZone`.
 *
 * @throws RangeError when `timeZone` is not a supported IANA zone.
 */
export function createZoneClock(timeZone: string): ZoneClock {
  const canonical = new Intl.DateTimeFormat("en-US", { timeZone }).resolvedOptions().timeZone;
  if (canonical === "UTC" || canonical === "Etc/UTC" || canonical === "Etc/GMT") {
    return { timeZone: canonical, utc: true, toWall: (epoch) => epoch, fromWall: (wall) => wall };
  }
  const formatter = partsFormatter(canonical);
  const offset = (epoch: number) => zoneOffset(formatter, epoch);
  return {
    timeZone: canonical,
    utc: false,
    toWall: (epoch) => epoch + offset(epoch),
    // Resolve the offset at the wall time, then re-check once so instants near a
    // transition pick the offset that is actually in effect, like local Dates do.
    fromWall: (wall) => {
      const guess = wall - offset(wall);
      const corrected = wall - offset(guess);
      return offset(corrected) === wall - corrected ? corrected : guess;
    },
  };
}

/** A calendar interval over epoch milliseconds. */
export interface TimeInterval {
  readonly floor: (epoch: number) => number;
  readonly offset: (epoch: number, step: number) => number;
  readonly ceil: (epoch: number) => number;
  readonly range: (start: number, stop: number) => number[];
  readonly every: (step: number) => TimeInterval | null;
}

interface IntervalSpec {
  readonly floor: (epoch: number) => number;
  readonly offset: (epoch: number, step: number) => number;
  readonly field?: (epoch: number) => number;
  readonly every?: (step: number) => TimeInterval | null;
}

function createInterval(spec: IntervalSpec): TimeInterval {
  const ceil = (epoch: number) => spec.floor(spec.offset(spec.floor(epoch - 1), 1));
  const interval: TimeInterval = {
    floor: spec.floor,
    offset: (epoch, step) => spec.offset(epoch, Math.floor(step)),
    ceil,
    range: (start, stop) => {
      const values: number[] = [];
      let current = ceil(start);
      if (!(current < stop)) return values;
      let previous: number;
      do {
        values.push((previous = current));
        current = spec.floor(spec.offset(current, 1));
      } while (previous < current && current < stop);
      return values;
    },
    every: (stepInput) => {
      if (spec.every !== undefined) return spec.every(stepInput);
      const step = Math.floor(stepInput);
      if (!Number.isFinite(step) || !(step > 0)) return null;
      if (!(step > 1)) return interval;
      const field = spec.field;
      if (field === undefined) return null;
      return filterInterval(spec, (epoch) => field(epoch) % step === 0);
    },
  };
  return interval;
}

function filterInterval(spec: IntervalSpec, test: (epoch: number) => boolean): TimeInterval {
  return createInterval({
    floor: (input) => {
      let epoch = spec.floor(input);
      while (!test(epoch)) epoch = spec.floor(epoch - 1);
      return epoch;
    },
    offset: (input, step) => {
      let epoch = input;
      if (step < 0) {
        for (let remaining = step; remaining < 0; remaining++) {
          do epoch = spec.offset(epoch, -1);
          while (!test(epoch));
        }
      } else {
        for (let remaining = step; remaining > 0; remaining--) {
          do epoch = spec.offset(epoch, 1);
          while (!test(epoch));
        }
      }
      return epoch;
    },
  });
}

function millisecondEvery(stepInput: number): TimeInterval | null {
  const step = Math.floor(stepInput);
  if (!Number.isFinite(step) || !(step > 0)) return null;
  return createInterval({
    floor: (epoch) => Math.floor(epoch / step) * step,
    offset: (epoch, count) => epoch + count * step,
  });
}

function mod(value: number, divisor: number): number {
  return ((value % divisor) + divisor) % divisor;
}

/** Calendar intervals for one zone, matching d3's local or UTC intervals. */
export interface ZoneIntervals {
  readonly clock: ZoneClock;
  readonly millisecond: TimeInterval;
  readonly second: TimeInterval;
  readonly minute: TimeInterval;
  readonly hour: TimeInterval;
  readonly day: TimeInterval;
  readonly week: TimeInterval;
  readonly month: TimeInterval;
  readonly year: TimeInterval;
}

const intervalCache = new Map<string, ZoneIntervals>();

/** Build (and cache) the calendar intervals for `timeZone`. */
export function zoneIntervals(timeZone: string): ZoneIntervals {
  const cached = intervalCache.get(timeZone);
  if (cached !== undefined) return cached;
  const clock = createZoneClock(timeZone);
  const { toWall, fromWall } = clock;
  const wallDate = (epoch: number) => new Date(toWall(epoch));
  const floorWall = (epoch: number, unit: number) => epoch - mod(toWall(epoch), unit);

  const yearInterval = (every: number) =>
    createInterval({
      floor: (epoch) => {
        const date = wallDate(epoch);
        const y = Math.floor(date.getUTCFullYear() / every) * every;
        date.setUTCFullYear(y, 0, 1);
        date.setUTCHours(0, 0, 0, 0);
        return fromWall(date.getTime());
      },
      offset: (epoch, step) => {
        const date = wallDate(epoch);
        date.setUTCFullYear(date.getUTCFullYear() + step * every);
        return fromWall(date.getTime());
      },
    });

  const intervals: ZoneIntervals = {
    clock,
    millisecond: createInterval({
      floor: (epoch) => epoch,
      offset: (epoch, step) => epoch + step,
      every: millisecondEvery,
    }),
    second: createInterval({
      floor: (epoch) => epoch - mod(epoch, second),
      offset: (epoch, step) => epoch + step * second,
      field: (epoch) => new Date(epoch).getUTCSeconds(),
    }),
    minute: createInterval({
      floor: (epoch) => floorWall(epoch, minute),
      offset: (epoch, step) => epoch + step * minute,
      field: (epoch) => wallDate(epoch).getUTCMinutes(),
    }),
    hour: createInterval({
      floor: (epoch) => floorWall(epoch, hour),
      offset: (epoch, step) => epoch + step * hour,
      field: (epoch) => wallDate(epoch).getUTCHours(),
    }),
    day: createInterval({
      floor: (epoch) => fromWall(toWall(epoch) - mod(toWall(epoch), day)),
      offset: (epoch, step) => fromWall(toWall(epoch) + step * day),
      field: clock.utc
        ? (epoch) => Math.floor(epoch / day)
        : (epoch) => wallDate(epoch).getUTCDate() - 1,
    }),
    week: createInterval({
      floor: (epoch) => {
        const wall = toWall(epoch);
        const midnight = wall - mod(wall, day);
        return fromWall(midnight - new Date(midnight).getUTCDay() * day);
      },
      offset: (epoch, step) => fromWall(toWall(epoch) + step * week),
    }),
    month: createInterval({
      floor: (epoch) => {
        const date = wallDate(epoch);
        return fromWall(Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), 1));
      },
      offset: (epoch, step) => {
        const date = wallDate(epoch);
        date.setUTCMonth(date.getUTCMonth() + step);
        return fromWall(date.getTime());
      },
      field: (epoch) => wallDate(epoch).getUTCMonth(),
    }),
    year: createInterval({
      floor: (epoch) => yearInterval(1).floor(epoch),
      offset: (epoch, step) => yearInterval(1).offset(epoch, step),
      every: (stepInput) => {
        const step = Math.floor(stepInput);
        return !Number.isFinite(step) || !(step > 0) ? null : yearInterval(step);
      },
    }),
  };
  intervalCache.set(timeZone, intervals);
  return intervals;
}

/** Choose the tick interval d3's `timeTickInterval` would pick for `[start, stop]`. */
export function timeTickInterval(
  intervals: ZoneIntervals,
  start: number,
  stop: number,
  count: number,
  tickStep: (start: number, stop: number, count: number) => number,
): TimeInterval | null {
  const table: readonly (readonly [TimeInterval, number, number])[] = [
    [intervals.second, 1, second],
    [intervals.second, 5, 5 * second],
    [intervals.second, 15, 15 * second],
    [intervals.second, 30, 30 * second],
    [intervals.minute, 1, minute],
    [intervals.minute, 5, 5 * minute],
    [intervals.minute, 15, 15 * minute],
    [intervals.minute, 30, 30 * minute],
    [intervals.hour, 1, hour],
    [intervals.hour, 3, 3 * hour],
    [intervals.hour, 6, 6 * hour],
    [intervals.hour, 12, 12 * hour],
    [intervals.day, 1, day],
    [intervals.day, 2, 2 * day],
    [intervals.week, 1, week],
    [intervals.month, 1, month],
    [intervals.month, 3, 3 * month],
    [intervals.year, 1, year],
  ];
  const target = Math.abs(stop - start) / count;
  let i = 0;
  while (i < table.length && (table[i]?.[2] ?? Infinity) <= target) i++;
  if (i === table.length) {
    return intervals.year.every(tickStep(start / year, stop / year, count));
  }
  if (i === 0) return intervals.millisecond.every(Math.max(tickStep(start, stop, count), 1));
  const before = table[i - 1];
  const after = table[i];
  if (before === undefined || after === undefined) return null;
  const [interval, step] = target / before[2] < after[2] / target ? before : after;
  return interval.every(step);
}

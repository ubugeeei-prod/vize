/**
 * Wall-clock date and time without a time zone, in the ISO calendar.
 * `Temporal.PlainDateTime` (ISO calendar) satisfies {@link PlainDateTime}
 * structurally.
 */
import {
  addDays,
  compareDates,
  formatIsoDate,
  normalizePlainDate,
  parseIsoDate,
} from "../calendar/plain-date.ts";
import type { PlainDate } from "../calendar/plain-date.ts";
import {
  formatIsoTime,
  normalizePlainTime,
  parseIsoTime,
  toSecondOfDay,
} from "../time-field/plain-time.ts";
import type { PlainTime, TimeGranularity } from "../time-field/plain-time.ts";

/** A calendar date plus a wall-clock time. */
export interface PlainDateTime extends PlainDate, PlainTime {}

/** Loose input accepted by {@link normalizePlainDateTime}; `second` defaults to `0`. */
export interface PlainDateTimeLike extends PlainDate {
  readonly hour: number;
  readonly minute: number;
  readonly second?: number;
}

/** Copy any date-time record into a frozen {@link PlainDateTime}; invalid input returns `null`. */
export function normalizePlainDateTime(
  value: PlainDateTimeLike | null | undefined,
): PlainDateTime | null {
  const date = normalizePlainDate(value);
  const time = value ? normalizePlainTime(value) : null;
  if (!date || !time) return null;
  return Object.freeze({ ...date, ...time });
}

/**
 * Create a frozen {@link PlainDateTime}.
 *
 * @throws `RangeError` tagged `VIZE_UI_PLAIN_DATE_TIME_INVALID` for impossible values.
 */
export function createPlainDateTime(
  year: number,
  month: number,
  day: number,
  hour = 0,
  minute = 0,
  second = 0,
): PlainDateTime {
  const value = normalizePlainDateTime({ year, month, day, hour, minute, second });
  if (!value) {
    throw new RangeError(
      `VIZE_UI_PLAIN_DATE_TIME_INVALID: ${year}-${month}-${day}T${hour}:${minute}:${second} is not a date-time`,
    );
  }
  return value;
}

/** Combine a date and a time. */
export function combineDateTime(date: PlainDate, time: PlainTime): PlainDateTime {
  return Object.freeze({
    year: date.year,
    month: date.month,
    day: date.day,
    hour: time.hour,
    minute: time.minute,
    second: time.second,
  });
}

/** Date part of a date-time. */
export function toPlainDate(value: PlainDate): PlainDate {
  return Object.freeze({ year: value.year, month: value.month, day: value.day });
}

/** Time part of a date-time. */
export function toPlainTime(value: PlainTime): PlainTime {
  return Object.freeze({ hour: value.hour, minute: value.minute, second: value.second });
}

/** Order two date-times. */
export function compareDateTimes(left: PlainDateTime, right: PlainDateTime): -1 | 0 | 1 {
  const date = compareDates(left, right);
  if (date !== 0) return date;
  const time = toSecondOfDay(left) - toSecondOfDay(right);
  return time === 0 ? 0 : time < 0 ? -1 : 1;
}

/** Whether two nullable date-times are equal. */
export function isSameDateTime(
  left: PlainDateTime | null | undefined,
  right: PlainDateTime | null | undefined,
): boolean {
  if (!left || !right) return !left && !right;
  return compareDateTimes(left, right) === 0;
}

/** Add a signed number of minutes, rolling over days. */
export function addMinutes(value: PlainDateTime, minutes: number): PlainDateTime {
  const total = value.hour * 60 + value.minute + Math.trunc(minutes);
  const dayOffset = Math.floor(total / 1_440);
  const minuteOfDay = total - dayOffset * 1_440;
  return combineDateTime(addDays(value, dayOffset), {
    hour: Math.floor(minuteOfDay / 60),
    minute: minuteOfDay % 60,
    second: value.second,
  });
}

/** Signed minutes from `from` to `to`, ignoring seconds. */
export function minutesBetween(from: PlainDateTime, to: PlainDateTime): number {
  const days = Math.round(
    (Date.UTC(to.year, to.month - 1, to.day) - Date.UTC(from.year, from.month - 1, from.day)) /
      86_400_000,
  );
  return days * 1_440 + (to.hour * 60 + to.minute) - (from.hour * 60 + from.minute);
}

/** Format as `YYYY-MM-DDTHH:MM`, or with seconds for `second` granularity (HTML `datetime-local`). */
export function formatIsoDateTime(
  value: PlainDateTime,
  granularity: TimeGranularity = "minute",
): string {
  return `${formatIsoDate(value)}T${formatIsoTime(value, granularity === "second" ? "second" : "minute")}`;
}

/** Parse `YYYY-MM-DDTHH:MM[:SS]` (a space separator is accepted); anything else returns `null`. */
export function parseIsoDateTime(value: string): PlainDateTime | null {
  const [datePart, timePart, extra] = value.trim().split(/[T ]/u);
  if (datePart === undefined || timePart === undefined || extra !== undefined) return null;
  const date = parseIsoDate(datePart);
  const time = parseIsoTime(timePart);
  return date && time ? combineDateTime(date, time) : null;
}

const zonedDateTimeFormatters = new Map<string, Intl.DateTimeFormat>();

/** Wall-clock date-time of an instant in an IANA time zone; unknown zones return `null`. */
export function fromEpochMillisecondsDateTime(
  epochMilliseconds: number,
  timeZone: string,
): PlainDateTime | null {
  if (!Number.isFinite(epochMilliseconds)) return null;
  let formatter = zonedDateTimeFormatters.get(timeZone);
  if (!formatter) {
    try {
      formatter = new Intl.DateTimeFormat("en-US", {
        timeZone,
        calendar: "gregory",
        numberingSystem: "latn",
        era: "short",
        year: "numeric",
        month: "numeric",
        day: "numeric",
        hour: "numeric",
        minute: "numeric",
        second: "numeric",
        hourCycle: "h23",
      });
    } catch {
      return null;
    }
    zonedDateTimeFormatters.set(timeZone, formatter);
  }
  const fields: Record<string, number> = {};
  let era = "";
  for (const part of formatter.formatToParts(epochMilliseconds)) {
    if (part.type === "era") era = part.value;
    else if (part.type !== "literal") fields[part.type] = Number(part.value);
  }
  const year = fields.year ?? Number.NaN;
  return normalizePlainDateTime({
    year: /^b/iu.test(era) ? 1 - year : year,
    month: fields.month ?? Number.NaN,
    day: fields.day ?? Number.NaN,
    hour: (fields.hour ?? Number.NaN) % 24,
    minute: fields.minute ?? Number.NaN,
    second: fields.second ?? Number.NaN,
  });
}

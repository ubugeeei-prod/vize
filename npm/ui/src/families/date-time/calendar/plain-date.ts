/**
 * Timezone-free ISO calendar date model shared by the date-time family.
 *
 * Every value is a plain `{ year, month, day }` record in the proleptic
 * Gregorian (ISO 8601) calendar. Arithmetic runs on integer epoch days, so no
 * operation ever constructs a `Date` in the host time zone and results are
 * identical on the server, in every browser, and across daylight-saving
 * transitions. `Temporal.PlainDate` instances (ISO calendar) satisfy
 * {@link PlainDate} structurally and are accepted wherever a date is read.
 */

/** A calendar date without time or time zone, in the ISO 8601 calendar. */
export interface PlainDate {
  /** ISO year. Year `0` is 1 BCE; negative years are supported. */
  readonly year: number;

  /** ISO month from `1` (January) through `12` (December). */
  readonly month: number;

  /** Day of month from `1` through the month length. */
  readonly day: number;
}

/** A calendar month without a day, in the ISO 8601 calendar. */
export interface PlainYearMonth {
  /** ISO year. */
  readonly year: number;

  /** ISO month from `1` through `12`. */
  readonly month: number;
}

/** An inclusive date range whose `start` is never after its `end`. */
export interface DateRange {
  /** First selected date, inclusive. */
  readonly start: PlainDate;

  /** Last selected date, inclusive. */
  readonly end: PlainDate;
}

/** Predicate that marks individual dates, for example unavailable dates. */
export type DateMatcher = (date: PlainDate) => boolean;

/** Day of week where `0` is Sunday and `6` is Saturday. */
export type Weekday = 0 | 1 | 2 | 3 | 4 | 5 | 6;

/** Result of {@link compareDates}. */
export type DateOrder = -1 | 0 | 1;

/** Smallest year accepted by the model; matches the ECMAScript `Date` range. */
export const PLAIN_DATE_MIN_YEAR = -271_820;

/** Largest year accepted by the model; matches the ECMAScript `Date` range. */
export const PLAIN_DATE_MAX_YEAR = 275_759;

const millisecondsPerDay = 86_400_000;
const isoDatePattern = /^([+-]\d{6}|\d{4})-(\d{2})-(\d{2})$/u;

/** Whether `year` is a Gregorian leap year. */
export function isLeapYear(year: number): boolean {
  return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;
}

/** Number of days in the given ISO month, or `0` when the month is invalid. */
export function daysInMonth(year: number, month: number): number {
  if (!Number.isInteger(month) || month < 1 || month > 12) return 0;
  if (month === 2) return isLeapYear(year) ? 29 : 28;
  return month === 4 || month === 6 || month === 9 || month === 11 ? 30 : 31;
}

/** Whether a value is a structurally valid ISO date inside the supported year range. */
export function isValidPlainDate(value: unknown): value is PlainDate {
  if (typeof value !== "object" || value === null) return false;
  const { year, month, day } = value as Partial<Record<keyof PlainDate, unknown>>;
  return (
    typeof year === "number" &&
    typeof month === "number" &&
    typeof day === "number" &&
    Number.isInteger(year) &&
    year >= PLAIN_DATE_MIN_YEAR &&
    year <= PLAIN_DATE_MAX_YEAR &&
    Number.isInteger(day) &&
    day >= 1 &&
    day <= daysInMonth(year, month)
  );
}

/**
 * Create a frozen {@link PlainDate}.
 *
 * @throws `RangeError` tagged `VIZE_UI_PLAIN_DATE_INVALID` for impossible dates.
 */
export function createPlainDate(year: number, month: number, day: number): PlainDate {
  const value = { year, month, day };
  if (!isValidPlainDate(value)) {
    throw new RangeError(
      `VIZE_UI_PLAIN_DATE_INVALID: ${String(year)}-${String(month)}-${String(day)} is not an ISO date`,
    );
  }
  return Object.freeze(value);
}

/**
 * Copy any `{ year, month, day }` record, including `Temporal.PlainDate`, into
 * a frozen {@link PlainDate}. Invalid, missing, or out-of-range input returns `null`.
 */
export function normalizePlainDate(value: PlainDate | null | undefined): PlainDate | null {
  if (!isValidPlainDate(value)) return null;
  return Object.freeze({ year: value.year, month: value.month, day: value.day });
}

/** Convert a date to days since 1970-01-01 (negative before the epoch). */
export function toEpochDay(date: PlainDate): number {
  const year = date.year - (date.month <= 2 ? 1 : 0);
  const era = Math.floor(year / 400);
  const yearOfEra = year - era * 400;
  const shiftedMonth = date.month + (date.month > 2 ? -3 : 9);
  const dayOfYear = Math.floor((153 * shiftedMonth + 2) / 5) + date.day - 1;
  const dayOfEra =
    yearOfEra * 365 + Math.floor(yearOfEra / 4) - Math.floor(yearOfEra / 100) + dayOfYear;
  return era * 146_097 + dayOfEra - 719_468;
}

/** Convert days since 1970-01-01 back into a frozen {@link PlainDate}. */
export function fromEpochDay(epochDay: number): PlainDate {
  const shifted = Math.trunc(epochDay) + 719_468;
  const era = Math.floor(shifted / 146_097);
  const dayOfEra = shifted - era * 146_097;
  const yearOfEra = Math.floor(
    (dayOfEra -
      Math.floor(dayOfEra / 1_460) +
      Math.floor(dayOfEra / 36_524) -
      Math.floor(dayOfEra / 146_096)) /
      365,
  );
  const dayOfYear =
    dayOfEra - (365 * yearOfEra + Math.floor(yearOfEra / 4) - Math.floor(yearOfEra / 100));
  const shiftedMonth = Math.floor((5 * dayOfYear + 2) / 153);
  const day = dayOfYear - Math.floor((153 * shiftedMonth + 2) / 5) + 1;
  const month = shiftedMonth < 10 ? shiftedMonth + 3 : shiftedMonth - 9;
  const year = yearOfEra + era * 400 + (month <= 2 ? 1 : 0);
  return Object.freeze({ year, month, day });
}

/** Order two dates chronologically. */
export function compareDates(left: PlainDate, right: PlainDate): DateOrder {
  if (left.year !== right.year) return left.year < right.year ? -1 : 1;
  if (left.month !== right.month) return left.month < right.month ? -1 : 1;
  if (left.day !== right.day) return left.day < right.day ? -1 : 1;
  return 0;
}

/** Whether two nullable dates denote the same day. */
export function isSameDay(
  left: PlainDate | null | undefined,
  right: PlainDate | null | undefined,
): boolean {
  if (!left || !right) return !left && !right;
  return compareDates(left, right) === 0;
}

/** Whether two dates fall in the same ISO month. */
export function isSameMonth(left: PlainYearMonth, right: PlainYearMonth): boolean {
  return left.year === right.year && left.month === right.month;
}

/** Add a signed number of days. */
export function addDays(date: PlainDate, days: number): PlainDate {
  return fromEpochDay(toEpochDay(date) + Math.trunc(days));
}

/** Add a signed number of months, clamping the day to the target month length. */
export function addMonths(date: PlainDate, months: number): PlainDate {
  const month = shiftYearMonth(date, months);
  return Object.freeze({
    year: month.year,
    month: month.month,
    day: Math.min(date.day, daysInMonth(month.year, month.month)),
  });
}

/** Add a signed number of years, clamping February 29 in non-leap years. */
export function addYears(date: PlainDate, years: number): PlainDate {
  return addMonths(date, Math.trunc(years) * 12);
}

/** Shift a year-month by a signed number of months. */
export function shiftYearMonth(value: PlainYearMonth, months: number): PlainYearMonth {
  const index = value.year * 12 + (value.month - 1) + Math.trunc(months);
  const year = Math.floor(index / 12);
  return Object.freeze({ year, month: index - year * 12 + 1 });
}

/** Signed number of whole months from `from` to `to`. */
export function monthsBetween(from: PlainYearMonth, to: PlainYearMonth): number {
  return to.year * 12 + to.month - (from.year * 12 + from.month);
}

/** Signed number of days from `from` to `to`. */
export function daysBetween(from: PlainDate, to: PlainDate): number {
  return toEpochDay(to) - toEpochDay(from);
}

/** Day of week, where `0` is Sunday. */
export function dayOfWeek(date: PlainDate): Weekday {
  return ((((toEpochDay(date) + 4) % 7) + 7) % 7) as Weekday;
}

/** First date of the week containing `date` for a locale week start. */
export function startOfWeek(date: PlainDate, weekStartsOn: Weekday): PlainDate {
  return addDays(date, -((dayOfWeek(date) - weekStartsOn + 7) % 7));
}

/** Last date of the week containing `date` for a locale week start. */
export function endOfWeek(date: PlainDate, weekStartsOn: Weekday): PlainDate {
  return addDays(startOfWeek(date, weekStartsOn), 6);
}

/** First day of the month containing `value`. */
export function startOfMonth(value: PlainYearMonth): PlainDate {
  return Object.freeze({ year: value.year, month: value.month, day: 1 });
}

/** Last day of the month containing `value`. */
export function endOfMonth(value: PlainYearMonth): PlainDate {
  return Object.freeze({
    year: value.year,
    month: value.month,
    day: daysInMonth(value.year, value.month),
  });
}

/** Year-month that contains `date`. */
export function toYearMonth(date: PlainYearMonth): PlainYearMonth {
  return Object.freeze({ year: date.year, month: date.month });
}

/** Clamp a date into an optional inclusive `[min, max]` window. */
export function clampDate(
  date: PlainDate,
  min: PlainDate | null | undefined,
  max: PlainDate | null | undefined,
): PlainDate {
  if (min && compareDates(date, min) < 0) return min;
  if (max && compareDates(date, max) > 0) return max;
  return date;
}

/** Whether a date is inside an optional inclusive `[min, max]` window. */
export function isDateWithin(
  date: PlainDate,
  min: PlainDate | null | undefined,
  max: PlainDate | null | undefined,
): boolean {
  return (!min || compareDates(date, min) >= 0) && (!max || compareDates(date, max) <= 0);
}

/** Order two dates into a frozen {@link DateRange}. */
export function createDateRange(first: PlainDate, second: PlainDate): DateRange {
  const [start, end] = compareDates(first, second) <= 0 ? [first, second] : [second, first];
  return Object.freeze({ start: normalizeDate(start), end: normalizeDate(end) });
}

/** Copy and order a `{ start, end }` record; invalid input returns `null`. */
export function normalizeDateRange(value: DateRange | null | undefined): DateRange | null {
  if (typeof value !== "object" || value === null) return null;
  const start = normalizePlainDate(value.start);
  const end = normalizePlainDate(value.end);
  return start && end ? createDateRange(start, end) : null;
}

/** Whether `date` lies inside an inclusive range. */
export function isDateInRange(date: PlainDate, range: DateRange | null | undefined): boolean {
  return range ? isDateWithin(date, range.start, range.end) : false;
}

/** Whether two nullable ranges cover the same days. */
export function isSameRange(
  left: DateRange | null | undefined,
  right: DateRange | null | undefined,
): boolean {
  if (!left || !right) return !left && !right;
  return isSameDay(left.start, right.start) && isSameDay(left.end, right.end);
}

/** Format an ISO 8601 calendar date such as `2026-09-25` (extended years use `±YYYYYY`). */
export function formatIsoDate(date: PlainDate): string {
  const month = String(date.month).padStart(2, "0");
  const day = String(date.day).padStart(2, "0");
  const year =
    date.year >= 0 && date.year <= 9_999
      ? String(date.year).padStart(4, "0")
      : `${date.year < 0 ? "-" : "+"}${String(Math.abs(date.year)).padStart(6, "0")}`;
  return `${year}-${month}-${day}`;
}

/** Parse an ISO 8601 calendar date. Anything else, including impossible dates, returns `null`. */
export function parseIsoDate(value: string): PlainDate | null {
  const match = isoDatePattern.exec(value.trim());
  if (!match) return null;
  const year = Number(match[1]);
  if (Object.is(year, -0)) return null;
  return normalizePlainDate({ year, month: Number(match[2]), day: Number(match[3]) });
}

/** Read the calendar fields of a `Date` in the host's local time zone. */
export function fromLocalDate(date: Date): PlainDate | null {
  if (Number.isNaN(date.getTime())) return null;
  return normalizePlainDate({
    year: date.getFullYear(),
    month: date.getMonth() + 1,
    day: date.getDate(),
  });
}

/** Read the calendar fields of a `Date` in UTC. */
export function fromUtcDate(date: Date): PlainDate | null {
  if (Number.isNaN(date.getTime())) return null;
  return fromEpochDay(Math.floor(date.getTime() / millisecondsPerDay));
}

/** Create a `Date` at local midnight of `date` in the host time zone. */
export function toLocalDate(date: PlainDate): Date {
  const result = new Date(0);
  result.setFullYear(date.year, date.month - 1, date.day);
  result.setHours(0, 0, 0, 0);
  return result;
}

/** Create a `Date` at UTC midnight of `date`; formatting it with `timeZone: "UTC"` is stable everywhere. */
export function toUtcDate(date: PlainDate): Date {
  return new Date(toEpochDay(date) * millisecondsPerDay);
}

const zonedDateFormatters = new Map<string, Intl.DateTimeFormat>();

/**
 * Calendar date of an instant in an IANA time zone.
 *
 * This is the only operation that depends on a clock value; callers own
 * where the epoch milliseconds come from. Unknown time zones return `null`.
 */
export function fromEpochMilliseconds(
  epochMilliseconds: number,
  timeZone: string,
): PlainDate | null {
  if (!Number.isFinite(epochMilliseconds)) return null;
  let formatter = zonedDateFormatters.get(timeZone);
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
      });
    } catch {
      return null;
    }
    zonedDateFormatters.set(timeZone, formatter);
  }
  const fields = { era: "", year: Number.NaN, month: Number.NaN, day: Number.NaN };
  for (const part of formatter.formatToParts(epochMilliseconds)) {
    if (part.type === "era") fields.era = part.value;
    else if (part.type === "year" || part.type === "month" || part.type === "day") {
      fields[part.type] = Number(part.value);
    }
  }
  const year = /^b/iu.test(fields.era) ? 1 - fields.year : fields.year;
  return normalizePlainDate({ year, month: fields.month, day: fields.day });
}

function normalizeDate(date: PlainDate): PlainDate {
  return Object.isFrozen(date) && Object.keys(date).length === 3
    ? date
    : Object.freeze({ year: date.year, month: date.month, day: date.day });
}

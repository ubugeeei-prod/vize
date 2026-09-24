/**
 * Wall-clock time without date or time zone. `Temporal.PlainTime` instances
 * satisfy {@link PlainTime} structurally.
 */
export interface PlainTime {
  /** Hour from `0` through `23`. */
  readonly hour: number;

  /** Minute from `0` through `59`. */
  readonly minute: number;

  /** Second from `0` through `59`. */
  readonly second: number;
}

/** Smallest unit a time field edits. */
export type TimeGranularity = "hour" | "minute" | "second";

/** Hour clock preference: `12` renders a day-period segment, `24` does not. */
export type HourCycle = 12 | 24;

function inRange(value: unknown, max: number): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= 0 && value <= max;
}

/** Whether a value is a structurally valid {@link PlainTime}. */
export function isValidPlainTime(value: unknown): value is PlainTime {
  if (typeof value !== "object" || value === null) return false;
  const { hour, minute, second } = value as Partial<Record<keyof PlainTime, unknown>>;
  return inRange(hour, 23) && inRange(minute, 59) && (second === undefined || inRange(second, 59));
}

/**
 * Create a frozen {@link PlainTime}.
 *
 * @throws `RangeError` tagged `VIZE_UI_PLAIN_TIME_INVALID` for impossible times.
 */
export function createPlainTime(hour: number, minute = 0, second = 0): PlainTime {
  const value = { hour, minute, second };
  if (!isValidPlainTime(value)) {
    throw new RangeError(
      `VIZE_UI_PLAIN_TIME_INVALID: ${String(hour)}:${String(minute)}:${String(second)} is not a time`,
    );
  }
  return Object.freeze(value);
}

/** Copy any `{ hour, minute, second? }` record into a frozen {@link PlainTime}; invalid input returns `null`. */
export function normalizePlainTime(
  value:
    | { readonly hour: number; readonly minute: number; readonly second?: number }
    | null
    | undefined,
): PlainTime | null {
  if (!isValidPlainTime(value)) return null;
  return Object.freeze({ hour: value.hour, minute: value.minute, second: value.second ?? 0 });
}

/** Seconds since midnight. */
export function toSecondOfDay(time: PlainTime): number {
  return time.hour * 3_600 + time.minute * 60 + time.second;
}

/** Order two times. */
export function compareTimes(left: PlainTime, right: PlainTime): -1 | 0 | 1 {
  const difference = toSecondOfDay(left) - toSecondOfDay(right);
  return difference === 0 ? 0 : difference < 0 ? -1 : 1;
}

/** Whether two nullable times are equal. */
export function isSameTime(
  left: PlainTime | null | undefined,
  right: PlainTime | null | undefined,
): boolean {
  if (!left || !right) return !left && !right;
  return compareTimes(left, right) === 0;
}

/** Drop units smaller than `granularity`. */
export function truncateTime(time: PlainTime, granularity: TimeGranularity): PlainTime {
  return Object.freeze({
    hour: time.hour,
    minute: granularity === "hour" ? 0 : time.minute,
    second: granularity === "second" ? time.second : 0,
  });
}

/** Format as HTML/ISO time: `HH:MM`, or `HH:MM:SS` when seconds are edited. */
export function formatIsoTime(time: PlainTime, granularity: TimeGranularity = "minute"): string {
  const pad = (value: number) => String(value).padStart(2, "0");
  const base = `${pad(time.hour)}:${pad(time.minute)}`;
  return granularity === "second" ? `${base}:${pad(time.second)}` : base;
}

/** Parse `HH:MM` or `HH:MM:SS` (fractional seconds are truncated); anything else returns `null`. */
export function parseIsoTime(value: string): PlainTime | null {
  const match = /^(\d{2}):(\d{2})(?::(\d{2})(?:\.\d+)?)?$/u.exec(value.trim());
  if (!match) return null;
  return normalizePlainTime({
    hour: Number(match[1]),
    minute: Number(match[2]),
    second: match[3] === undefined ? 0 : Number(match[3]),
  });
}

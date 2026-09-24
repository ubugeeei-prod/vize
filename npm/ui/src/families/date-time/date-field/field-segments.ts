import { daysInMonth } from "../calendar/plain-date.ts";

/** Editable date segments. */
export type DateSegmentType = "year" | "month" | "day";

/** Editable time segments. `dayPeriod` is AM/PM and stores `0` or `1`. */
export type TimeSegmentType = "hour" | "minute" | "second" | "dayPeriod";

/** Every editable segment type. */
export type EditableSegmentType = DateSegmentType | TimeSegmentType;

/** Every rendered segment type; literals are separators such as `/` or `:`. */
export type FieldSegmentType = EditableSegmentType | "literal";

/** Current value per editable segment; `null` means empty. */
export type FieldSegmentValues = Readonly<Record<EditableSegmentType, number | null>>;

/** One entry of a locale segment layout. */
export interface FieldSegmentLayoutPart {
  /** Segment type. */
  readonly type: FieldSegmentType;

  /** Literal text for separators; empty for editable segments. */
  readonly literal: string;
}

/** Hour clock used by time segments; `h12` shows 1–12 plus a day period. */
export type SegmentHourCycle = "h12" | "h23";

/** Inclusive numeric bounds of one editable segment. */
export interface FieldSegmentBounds {
  readonly min: number;
  readonly max: number;
}

/** Result of typing one digit into a segment. */
export interface SegmentDigitResult {
  /** New segment value; `null` while the typed prefix is below the minimum (for example a lone `0` month). */
  readonly value: number | null;

  /** Digits typed so far for the segment. */
  readonly buffer: string;

  /** Whether no further digit can extend the value, so focus should advance. */
  readonly advance: boolean;
}

/** Empty values for every editable segment. */
export const emptySegmentValues: FieldSegmentValues = Object.freeze({
  year: null,
  month: null,
  day: null,
  hour: null,
  minute: null,
  second: null,
  dayPeriod: null,
});

const editableTypes = new Set<string>([
  "year",
  "month",
  "day",
  "hour",
  "minute",
  "second",
  "dayPeriod",
]);

/** Whether a string names an editable segment type. */
export function isEditableSegmentType(value: string): value is EditableSegmentType {
  return editableTypes.has(value);
}

function collectLayout(parts: readonly Intl.DateTimeFormatPart[]): FieldSegmentLayoutPart[] {
  const layout: FieldSegmentLayoutPart[] = [];
  for (const part of parts) {
    if (isEditableSegmentType(part.type)) {
      if (!layout.some((entry) => entry.type === part.type)) {
        layout.push({ type: part.type, literal: "" });
      }
    } else if (part.type === "literal") {
      const previous = layout.at(-1);
      if (previous?.type === "literal") {
        layout[layout.length - 1] = { type: "literal", literal: previous.literal + part.value };
      } else {
        layout.push({ type: "literal", literal: part.value });
      }
    }
  }
  while (layout[0]?.type === "literal") layout.shift();
  while (layout.at(-1)?.type === "literal") layout.pop();
  return layout;
}

function safeFormatter(locale: string, options: Intl.DateTimeFormatOptions): Intl.DateTimeFormat {
  try {
    return new Intl.DateTimeFormat(locale, { ...options, timeZone: "UTC" });
  } catch {
    return new Intl.DateTimeFormat("en-US", { ...options, timeZone: "UTC" });
  }
}

/** Locale order and separators of year, month, and day segments. */
export function resolveDateSegmentLayout(locale: string): readonly FieldSegmentLayoutPart[] {
  const formatter = safeFormatter(locale, {
    calendar: "gregory",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
  return collectLayout(formatter.formatToParts(Date.UTC(2001, 10, 22)));
}

/** Locale default hour cycle, normalized to `h12` or `h23`. */
export function resolveHourCycle(locale: string): SegmentHourCycle {
  const cycle = safeFormatter(locale, { hour: "numeric" }).resolvedOptions().hourCycle;
  return cycle === "h11" || cycle === "h12" ? "h12" : "h23";
}

/** Locale order and separators of hour, minute, second, and day-period segments. */
export function resolveTimeSegmentLayout(
  locale: string,
  hourCycle: SegmentHourCycle,
  granularity: "hour" | "minute" | "second",
): readonly FieldSegmentLayoutPart[] {
  const options: Intl.DateTimeFormatOptions = { hour: "2-digit", hourCycle };
  if (granularity !== "hour") options.minute = "2-digit";
  if (granularity === "second") options.second = "2-digit";
  return collectLayout(
    safeFormatter(locale, options).formatToParts(Date.UTC(2001, 0, 1, 21, 5, 9)),
  );
}

/** Localized AM and PM labels in that order. */
export function resolveDayPeriodLabels(locale: string): readonly [string, string] {
  const formatter = safeFormatter(locale, { hour: "numeric", hourCycle: "h12" });
  const read = (hour: number) =>
    formatter.formatToParts(Date.UTC(2001, 0, 1, hour)).find((part) => part.type === "dayPeriod")
      ?.value ?? (hour < 12 ? "AM" : "PM");
  return [read(9), read(21)];
}

/** Localized segment field names from `Intl.DisplayNames`, with English fallbacks. */
export function resolveSegmentNames(locale: string): Readonly<Record<EditableSegmentType, string>> {
  const fallback: Record<EditableSegmentType, string> = {
    year: "year",
    month: "month",
    day: "day",
    hour: "hour",
    minute: "minute",
    second: "second",
    dayPeriod: "AM/PM",
  };
  try {
    const names = new Intl.DisplayNames(locale, { type: "dateTimeField" });
    for (const type of Object.keys(fallback) as EditableSegmentType[]) {
      const name = names.of(type);
      if (name) fallback[type] = name;
    }
  } catch {
    // Engines without dateTimeField display names keep English labels.
  }
  return fallback;
}

/** Numeric bounds of a segment given the other segment values. */
export function segmentBounds(
  type: EditableSegmentType,
  values: FieldSegmentValues,
  hourCycle: SegmentHourCycle = "h23",
): FieldSegmentBounds {
  switch (type) {
    case "year":
      return { min: 1, max: 9_999 };
    case "month":
      return { min: 1, max: 12 };
    case "day":
      return {
        min: 1,
        max: values.month === null ? 31 : daysInMonth(values.year ?? 2000, values.month),
      };
    case "hour":
      return hourCycle === "h12" ? { min: 1, max: 12 } : { min: 0, max: 23 };
    case "dayPeriod":
      return { min: 0, max: 1 };
    default:
      return { min: 0, max: 59 };
  }
}

/** Maximum digits typed into a segment before focus advances. */
export function segmentMaxDigits(type: EditableSegmentType): number {
  if (type === "year") return 4;
  return type === "dayPeriod" ? 1 : 2;
}

/** Default PageUp/PageDown step per segment. */
export function segmentPageStep(type: EditableSegmentType): number {
  switch (type) {
    case "year":
      return 10;
    case "month":
      return 3;
    case "day":
      return 7;
    case "hour":
      return 2;
    case "dayPeriod":
      return 1;
    default:
      return 15;
  }
}

/** Step a segment by `delta`, wrapping within bounds; empty segments start at `start`. */
export function stepSegmentValue(
  value: number | null,
  delta: number,
  bounds: FieldSegmentBounds,
  start: number,
): number {
  if (value === null) return Math.min(bounds.max, Math.max(bounds.min, start));
  const span = bounds.max - bounds.min + 1;
  const offset = (((value - bounds.min + delta) % span) + span) % span;
  return bounds.min + offset;
}

/** Type one ASCII digit into a segment buffer. */
export function typeSegmentDigit(
  buffer: string,
  digit: string,
  bounds: FieldSegmentBounds,
  maxDigits: number,
): SegmentDigitResult {
  let nextBuffer = `${buffer}${digit}`.slice(-maxDigits);
  let next = Number(nextBuffer);
  if (next > bounds.max) {
    nextBuffer = digit;
    next = Number(digit);
  }
  const complete = nextBuffer.length >= maxDigits || next * 10 > bounds.max;
  const value = next >= bounds.min && next <= bounds.max ? next : null;
  return { value, buffer: nextBuffer, advance: complete && value !== null };
}

/** Remove the last digit of a segment value; single-digit values become empty. */
export function backspaceSegmentValue(
  value: number | null,
  bounds: FieldSegmentBounds,
): number | null {
  if (value === null) return null;
  const next = Math.trunc(value / 10);
  return next === 0 || next < bounds.min ? null : next;
}

/**
 * Pure, deterministic layout for scheduler views.
 *
 * Nothing here reads the clock, the DOM, or the host time zone: inputs are
 * plain dates and wall-clock date-times, so server and client compute
 * identical placements.
 */
import {
  addDays,
  compareDates,
  daysBetween,
  endOfMonth,
  endOfWeek,
  formatIsoDate,
  isSameMonth,
  startOfMonth,
  startOfWeek,
} from "../calendar/plain-date.ts";
import type { PlainDate, Weekday } from "../calendar/plain-date.ts";
import { compareDateTimes, minutesBetween } from "../datetime-field/plain-date-time.ts";
import type { PlainDateTime } from "../datetime-field/plain-date-time.ts";

/** Scheduler view granularity. */
export type SchedulerView = "day" | "week" | "month";

/** One consumer event. `Data` is the consumer's own payload type. */
export interface SchedulerEvent<Data = unknown> {
  /** Stable unique id. */
  readonly id: string;

  /** Inclusive start. */
  readonly start: PlainDateTime;

  /** Exclusive end; events with `end <= start` are treated as zero-length. */
  readonly end: PlainDateTime;

  /** Whether the event spans whole days and belongs in all-day lanes. */
  readonly allDay?: boolean;

  /** Accessible title. */
  readonly title: string;

  /** Consumer payload, returned verbatim in every emit and slot. */
  readonly data: Data;
}

/** Placement of a timed event inside one day column of a time grid. */
export interface TimeGridPlacement<Data = unknown> {
  readonly event: SchedulerEvent<Data>;
  /** ISO date of the day column. */
  readonly day: string;
  /** Visible start and end minute of day after clipping to the day window. */
  readonly startMinute: number;
  readonly endMinute: number;
  /** Zero-based column inside the overlap cluster. */
  readonly column: number;
  /** Columns in the event's overlap cluster. */
  readonly columns: number;
  /** Columns the event may widen into without overlapping. */
  readonly span: number;
  /** Offset and size as fractions of the day window (`0`–`1`). */
  readonly top: number;
  readonly height: number;
  /** Whether the event started before or continues after the visible window. */
  readonly continuesBefore: boolean;
  readonly continuesAfter: boolean;
}

/** Placement of an event across a row of consecutive days (all-day lanes, month weeks). */
export interface RowPlacement<Data = unknown> {
  readonly event: SchedulerEvent<Data>;
  /** First and last day index covered inside the row, inclusive. */
  readonly startIndex: number;
  readonly endIndex: number;
  /** Zero-based stacking lane. */
  readonly lane: number;
  /** Whether the event started before or continues after the row. */
  readonly continuesBefore: boolean;
  readonly continuesAfter: boolean;
}

/** Options for {@link layoutTimeGrid}. */
export interface TimeGridWindow {
  /** First visible minute of day. @default 0 */
  readonly startMinute?: number;
  /** Last visible minute of day, exclusive. @default 1440 */
  readonly endMinute?: number;
  /** Minimum rendered length in minutes so short events stay visible. @default 15 */
  readonly minimumMinutes?: number;
}

function dayStart(day: PlainDate): PlainDateTime {
  return Object.freeze({
    year: day.year,
    month: day.month,
    day: day.day,
    hour: 0,
    minute: 0,
    second: 0,
  });
}

function sortEvents<Data>(left: SchedulerEvent<Data>, right: SchedulerEvent<Data>): number {
  return (
    compareDateTimes(left.start, right.start) ||
    compareDateTimes(right.end, left.end) ||
    (left.id < right.id ? -1 : left.id > right.id ? 1 : 0)
  );
}

/** Whether an event covers any part of `day` (all-day events cover whole days). */
export function eventTouchesDay<Data>(event: SchedulerEvent<Data>, day: PlainDate): boolean {
  const start = dayStart(day);
  const startOffset = minutesBetween(start, event.start);
  const endOffset = minutesBetween(start, event.end);
  if (endOffset <= startOffset) return startOffset >= 0 && startOffset < 1_440;
  return startOffset < 1_440 && endOffset > 0;
}

/**
 * Lay out timed events for one day column.
 *
 * Events are sorted by start (longer first on ties), grouped into clusters of
 * transitively overlapping events, assigned the first free column, and then
 * widened to the right across columns that stay free for their whole span.
 */
export function layoutTimeGrid<Data>(
  events: readonly SchedulerEvent<Data>[],
  day: PlainDate,
  window: TimeGridWindow = {},
): readonly TimeGridPlacement<Data>[] {
  const windowStart = Math.max(0, Math.trunc(window.startMinute ?? 0));
  const windowEnd = Math.min(
    1_440,
    Math.max(windowStart + 1, Math.trunc(window.endMinute ?? 1_440)),
  );
  const minimum = Math.max(1, Math.trunc(window.minimumMinutes ?? 15));
  const base = dayStart(day);
  const items = events
    .filter((event) => event.allDay !== true && eventTouchesDay(event, day))
    .sort(sortEvents)
    .flatMap((event) => {
      const rawStart = minutesBetween(base, event.start);
      const rawEnd = Math.max(rawStart, minutesBetween(base, event.end));
      const startMinute = Math.max(windowStart, rawStart);
      const endMinute = Math.min(windowEnd, Math.max(rawEnd, rawStart + minimum));
      if (endMinute <= startMinute || startMinute >= windowEnd) return [];
      return [{ event, startMinute, endMinute, rawStart, rawEnd, column: 0 }];
    });

  const placements: TimeGridPlacement<Data>[] = [];
  let cluster: typeof items = [];
  let clusterEnd = -1;
  const flush = () => {
    const columnEnds: number[] = [];
    for (const item of cluster) {
      let column = columnEnds.findIndex((end) => end <= item.startMinute);
      if (column === -1) column = columnEnds.length;
      columnEnds[column] = item.endMinute;
      item.column = column;
    }
    const columns = Math.max(1, columnEnds.length);
    for (const item of cluster) {
      let span = 1;
      while (item.column + span < columns) {
        const next = item.column + span;
        const blocked = cluster.some(
          (other) =>
            other !== item &&
            other.column === next &&
            other.startMinute < item.endMinute &&
            other.endMinute > item.startMinute,
        );
        if (blocked) break;
        span += 1;
      }
      const size = windowEnd - windowStart;
      placements.push(
        Object.freeze({
          event: item.event,
          day: formatIsoDate(day),
          startMinute: item.startMinute,
          endMinute: item.endMinute,
          column: item.column,
          columns,
          span,
          top: (item.startMinute - windowStart) / size,
          height: (item.endMinute - item.startMinute) / size,
          continuesBefore: item.rawStart < item.startMinute,
          continuesAfter: item.rawEnd > item.endMinute,
        }),
      );
    }
    cluster = [];
    clusterEnd = -1;
  };
  for (const item of items) {
    if (cluster.length > 0 && item.startMinute >= clusterEnd) flush();
    cluster.push(item);
    clusterEnd = Math.max(clusterEnd, item.endMinute);
  }
  if (cluster.length > 0) flush();
  return placements;
}

/**
 * Stack events across a row of consecutive days into lanes.
 *
 * Used for all-day lanes above a time grid and for month-view week rows.
 * Longer events claim lower lanes first so bars stay aligned.
 */
export function layoutRow<Data>(
  events: readonly SchedulerEvent<Data>[],
  days: readonly PlainDate[],
  filter: (event: SchedulerEvent<Data>) => boolean = () => true,
): readonly RowPlacement<Data>[] {
  const first = days[0];
  const last = days.at(-1);
  if (!first || !last) return [];
  const spans = events
    .filter(filter)
    .flatMap((event) => {
      const indexes = days.flatMap((day, index) => (eventTouchesDay(event, day) ? [index] : []));
      const startIndex = indexes[0];
      const endIndex = indexes.at(-1);
      if (startIndex === undefined || endIndex === undefined) return [];
      return [{ event, startIndex, endIndex }];
    })
    .sort(
      (left, right) =>
        left.startIndex - right.startIndex ||
        right.endIndex - right.startIndex - (left.endIndex - left.startIndex) ||
        sortEvents(left.event, right.event),
    );
  const laneEnds: number[] = [];
  return spans.map(({ event, startIndex, endIndex }) => {
    let lane = laneEnds.findIndex((end) => end < startIndex);
    if (lane === -1) lane = laneEnds.length;
    laneEnds[lane] = endIndex;
    return Object.freeze({
      event,
      startIndex,
      endIndex,
      lane,
      continuesBefore: compareDates(event.start, first) < 0,
      continuesAfter: minutesBetween(dayStart(addDays(last, 1)), event.end) > 0,
    });
  });
}

/** Days rendered by a view anchored at `date`. */
export function visibleDays(
  view: SchedulerView,
  date: PlainDate,
  weekStartsOn: Weekday,
): readonly PlainDate[] {
  if (view === "day") return [date];
  if (view === "week") {
    const start = startOfWeek(date, weekStartsOn);
    return Array.from({ length: 7 }, (_, index) => addDays(start, index));
  }
  const start = startOfWeek(startOfMonth(date), weekStartsOn);
  const end = endOfWeek(endOfMonth(date), weekStartsOn);
  return Array.from({ length: daysBetween(start, end) + 1 }, (_, index) => addDays(start, index));
}

/** Split month-view days into week rows and flag days outside the anchor month. */
export function monthWeeks(
  days: readonly PlainDate[],
  anchor: PlainDate,
): readonly (readonly { readonly date: PlainDate; readonly outsideMonth: boolean }[])[] {
  const weeks: { readonly date: PlainDate; readonly outsideMonth: boolean }[][] = [];
  days.forEach((date, index) => {
    (weeks[Math.floor(index / 7)] ??= []).push({ date, outsideMonth: !isSameMonth(date, anchor) });
  });
  return weeks;
}

/** Move the view anchor by one period. */
export function shiftAnchor(view: SchedulerView, date: PlainDate, step: number): PlainDate {
  if (view === "day") return addDays(date, step);
  if (view === "week") return addDays(date, step * 7);
  const shifted = { year: date.year, month: date.month + step, day: 1 };
  const year = shifted.year + Math.floor((shifted.month - 1) / 12);
  const month = ((((shifted.month - 1) % 12) + 12) % 12) + 1;
  return Object.freeze({ year, month, day: Math.min(date.day, endOfMonth({ year, month }).day) });
}

/** Snap a minute of day to a multiple of `step` inside `[min, max]`. */
export function snapMinute(minute: number, step: number, min: number, max: number): number {
  const size = Math.max(1, Math.trunc(step));
  const snapped = Math.round(minute / size) * size;
  return Math.min(max, Math.max(min, snapped));
}

/**
 * Minute of day under a client Y coordinate inside a day column rectangle,
 * snapped down to `step` inside the visible window.
 */
export function minuteAtPoint(
  y: number,
  rect: { readonly top: number; readonly bottom: number },
  window: { readonly startMinute: number; readonly endMinute: number; readonly step: number },
): number {
  const height = rect.bottom - rect.top;
  const ratio = height > 0 ? (y - rect.top) / height : 0;
  const minute = window.startMinute + ratio * (window.endMinute - window.startMinute);
  const step = Math.max(1, Math.trunc(window.step));
  const floored = Math.floor(minute / step) * step;
  return Math.min(window.endMinute - step, Math.max(window.startMinute, floored));
}

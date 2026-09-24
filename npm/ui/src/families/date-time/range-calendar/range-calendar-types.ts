import type { CalendarRootExposeBase } from "../calendar/calendar-types.ts";
import type { DateRange, PlainDate } from "../calendar/plain-date.ts";

/** Public instance exposed by RangeCalendarRoot. */
export interface RangeCalendarRootExpose extends CalendarRootExposeBase {
  /** Current committed range. */
  readonly value: DateRange | null;

  /** First endpoint of a range that is still being picked, or `null`. */
  readonly anchor: PlainDate | null;

  /** Request a new range; returns whether it differs. */
  readonly setValue: (value: DateRange | null) => boolean;

  /** Cancel a pending anchor; returns whether one was pending. */
  readonly cancel: () => boolean;
}

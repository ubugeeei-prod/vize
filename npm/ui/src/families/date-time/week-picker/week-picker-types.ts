import type { CalendarRootExposeBase } from "../calendar/calendar-types.ts";
import type { DateRange, IsoWeek } from "../calendar/plain-date.ts";

/** Public instance exposed by WeekPickerRoot. */
export interface WeekPickerRootExpose extends CalendarRootExposeBase {
  /** Selected week as an inclusive start–end range. */
  readonly value: DateRange | null;

  /** ISO 8601 week of the selection, taken from its fourth day. */
  readonly isoWeek: IsoWeek | null;

  /** Request a week; returns whether it differs. */
  readonly setValue: (value: DateRange | null) => boolean;
}

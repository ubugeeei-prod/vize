import type { PlainDate } from "../calendar/plain-date.ts";
import type { PlainDateTime } from "../datetime-field/plain-date-time.ts";
import type {
  RowPlacement,
  SchedulerEvent,
  SchedulerView,
  TimeGridPlacement,
} from "./scheduler-layout.ts";

export type { RowPlacement, SchedulerEvent, SchedulerView, TimeGridPlacement };

/** Root state token mirrored to `data-state`. */
export type SchedulerState = "disabled" | "pending" | "readonly" | "ready";

/** Requested new timing for an event; the consumer applies it to `events`. */
export interface SchedulerEventChange<Data = unknown> {
  /** Event being moved or resized. */
  readonly event: SchedulerEvent<Data>;
  /** Requested start. */
  readonly start: PlainDateTime;
  /** Requested end. */
  readonly end: PlainDateTime;
}

/** Time range of an activated slot. */
export interface SchedulerSlotRange {
  readonly start: PlainDateTime;
  readonly end: PlainDateTime;
}

/** One rendered day column or month cell. */
export interface SchedulerDay {
  /** Calendar date. */
  readonly date: PlainDate;
  /** ISO date mirrored to `data-date`. */
  readonly iso: string;
  /** Localized column header, for example `Fri 25`. */
  readonly label: string;
  /** Localized short weekday, for example `Fri`. */
  readonly weekdayLabel: string;
  /** Localized full date used as the accessible name. */
  readonly fullLabel: string;
  /** Localized day of month. */
  readonly dayLabel: string;
  /** Whether the day equals the resolved current date. */
  readonly today: boolean;
  /** Whether the day belongs to an adjacent month in month view. */
  readonly outsideMonth: boolean;
}

/** One time-slot row of the time grid. */
export interface SchedulerSlotRow {
  /** Minute of day where the slot starts. */
  readonly minute: number;
  /** Localized slot start label. */
  readonly label: string;
}

/** Timed placements of one day column. */
export interface SchedulerDayColumnState<Data = unknown> {
  readonly day: SchedulerDay;
  readonly placements: readonly TimeGridPlacement<Data>[];
}

/** One month-view week row with its lane placements. */
export interface SchedulerWeekState<Data = unknown> {
  readonly days: readonly SchedulerDay[];
  readonly placements: readonly RowPlacement<Data>[];
  /** Events hidden beyond `maxLanes`, per day index. */
  readonly overflow: readonly number[];
}

/** State shared by every scheduler slot and exposed from the root. */
export interface SchedulerSlotState<Data = unknown> {
  readonly view: SchedulerView;
  /** View anchor, or `null` while pending. */
  readonly date: PlainDate | null;
  readonly today: PlainDate | null;
  readonly heading: string;
  readonly days: readonly SchedulerDay[];
  readonly slots: readonly SchedulerSlotRow[];
  readonly columns: readonly SchedulerDayColumnState<Data>[];
  /** All-day lane placements across the visible days (day and week views). */
  readonly allDay: readonly RowPlacement<Data>[];
  readonly weeks: readonly SchedulerWeekState<Data>[];
  readonly locale: string;
  readonly direction: "ltr" | "rtl";
  readonly disabled: boolean;
  readonly readOnly: boolean;
  readonly pending: boolean;
  readonly state: SchedulerState;
}

/** Public instance exposed by SchedulerRoot. */
export interface SchedulerRootExpose<Data = unknown> extends SchedulerSlotState<Data> {
  /** Rendered root element. */
  readonly root: HTMLDivElement | null;
  /** Move the anchor by one period; returns whether it moved. */
  readonly navigate: (step: -1 | 1) => boolean;
  /** Show the resolved current date; returns whether it moved. */
  readonly goToToday: () => boolean;
  /** Request a view. */
  readonly setView: (view: SchedulerView) => void;
  /** Request an anchor date. */
  readonly setDate: (date: PlainDate) => void;
  /** Focus the roving slot or day cell; returns whether focus moved. */
  readonly focus: (options?: FocusOptions) => boolean;
}

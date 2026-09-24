import type { CalendarWeekdayFormat, CalendarWeekdayLabel } from "./calendar-locale.ts";
import type { DateTimeNow } from "./calendar-today.ts";
import type { DateMatcher, PlainDate, PlainYearMonth, Weekday } from "./plain-date.ts";

export type { CalendarWeekdayFormat, CalendarWeekdayLabel, DateTimeNow };

/** Text direction used for horizontal arrow keys. */
export type CalendarDirection = "ltr" | "rtl";

/** Selection model owned by the calendar root. */
export type CalendarSelectionMode = "single" | "range";

/** Navigation unit accepted by previous/next controls. */
export type CalendarNavigationUnit = "month" | "year";

/** Root state token mirrored to `data-state`. */
export type CalendarState = "disabled" | "empty" | "pending" | "readonly" | "selected";

/** Day state token mirrored to each day's `data-state`. */
export type CalendarDayStateToken =
  | "disabled"
  | "idle"
  | "outside"
  | "range-middle"
  | "selected"
  | "unavailable";

/** State published for one rendered day. */
export interface CalendarDayState {
  /** Calendar date represented by the cell. */
  readonly date: PlainDate;

  /** ISO 8601 date, mirrored to `data-date`. */
  readonly iso: string;

  /** Localized day-of-month label. */
  readonly label: string;

  /** Localized full date used as the accessible name. */
  readonly fullLabel: string;

  /** Day of week, where `0` is Sunday. */
  readonly weekday: Weekday;

  /** Index of the month grid that renders this cell. */
  readonly monthIndex: number;

  /** Whether the date belongs to an adjacent month and is only shown for grid alignment. */
  readonly outsideMonth: boolean;

  /** Whether the date equals the resolved current date. */
  readonly today: boolean;

  /** Whether the date is the keyboard focus target (roving `tabindex=0`). */
  readonly focused: boolean;

  /** Whether the date is selected, including both ends of a range. */
  readonly selected: boolean;

  /** Whether the date is the first day of the selected or previewed range. */
  readonly rangeStart: boolean;

  /** Whether the date is the last day of the selected or previewed range. */
  readonly rangeEnd: boolean;

  /** Whether the date lies inside the selected or previewed range, ends included. */
  readonly inRange: boolean;

  /** Whether the range is a live preview while the second endpoint is chosen. */
  readonly preview: boolean;

  /** Whether the date is outside `min`/`max` or the calendar is disabled. */
  readonly disabled: boolean;

  /** Whether `isDateUnavailable` rejected the date; it stays focusable but not selectable. */
  readonly unavailable: boolean;

  /** Whether user selection is blocked while navigation still works. */
  readonly readOnly: boolean;

  /** Stable state token for styling and tests. */
  readonly state: CalendarDayStateToken;
}

/** One visible month grid. */
export interface CalendarMonthState {
  /** Zero-based position among the visible months. */
  readonly index: number;

  /** ISO year of the month. */
  readonly year: number;

  /** ISO month from `1` through `12`. */
  readonly month: number;

  /** Localized month and year label that names the grid. */
  readonly label: string;

  /** Rows of seven days in column order. */
  readonly weeks: readonly (readonly CalendarDayState[])[];
}

/** State shared by every calendar slot and exposed from the root. */
export interface CalendarSlotState {
  /** Selection model of the root. */
  readonly mode: CalendarSelectionMode;

  /** Visible month grids; empty while `pending`. */
  readonly months: readonly CalendarMonthState[];

  /** Localized weekday labels in column order. */
  readonly weekdays: readonly CalendarWeekdayLabel[];

  /** Localized heading for the visible months. */
  readonly heading: string;

  /** Keyboard focus target, or `null` while pending. */
  readonly focusedDate: PlainDate | null;

  /** Resolved current date, or `null` before a clock is available. */
  readonly today: PlainDate | null;

  /** Resolved BCP 47 locale. */
  readonly locale: string;

  /** Resolved text direction. */
  readonly direction: CalendarDirection;

  /** Resolved first day of week. */
  readonly weekStartsOn: Weekday;

  /** Whether every control is disabled. */
  readonly disabled: boolean;

  /** Whether selection is blocked while navigation works. */
  readonly readOnly: boolean;

  /**
   * Whether no date is known yet: no value, focused date, `today`, or `now`
   * was supplied, so the grid waits for the client clock after mount.
   */
  readonly pending: boolean;

  /** Stable root state token. */
  readonly state: CalendarState;
}

/** State exposed to CalendarHeading slots. */
export interface CalendarHeadingSlotState {
  /** Localized heading text. */
  readonly heading: string;

  /** Visible month grids. */
  readonly months: readonly CalendarMonthState[];
}

/** State exposed to CalendarPrev and CalendarNext slots. */
export interface CalendarNavigationSlotState {
  /** Navigation unit of the control. */
  readonly unit: CalendarNavigationUnit;

  /** Whether the control can move the view. */
  readonly disabled: boolean;
}

/** State exposed to CalendarGrid weekday slots. */
export interface CalendarWeekdaySlotState extends CalendarWeekdayLabel {
  /** Zero-based column index. */
  readonly column: number;
}

/** Common public instance surface of calendar roots. */
export interface CalendarRootExposeBase extends CalendarSlotState {
  /** Rendered root element. */
  readonly root: HTMLDivElement | null;

  /** Focus the roving day button; returns whether focus moved. */
  readonly focus: (options?: FocusOptions) => boolean;

  /** Move the keyboard focus target (and visible months) to a date, clamped to `min`/`max`. */
  readonly setFocusedDate: (date: PlainDate) => void;

  /** Show a month as the first visible month. */
  readonly setVisibleMonth: (month: PlainYearMonth) => void;

  /** Navigate by one page of months or years; returns whether the view moved. */
  readonly navigate: (unit: CalendarNavigationUnit, direction: -1 | 1) => boolean;
}

/** Public instance exposed by CalendarRoot. */
export interface CalendarRootExpose extends CalendarRootExposeBase {
  /** Current selected date. */
  readonly value: PlainDate | null;

  /** Request a new value; returns whether it differs. */
  readonly setValue: (value: PlainDate | null) => boolean;
}

/** Shared props of CalendarRoot and RangeCalendarRoot, documented for adapters. */
export interface CalendarSharedProps {
  /** Consumer-owned base id. `null` and `undefined` select a deterministic fallback. @default undefined */
  readonly id?: string | null;
  /** Controlled keyboard focus date; also controls which months are visible. @default undefined */
  readonly focusedDate?: PlainDate | null;
  /** Earliest selectable date, inclusive. @default undefined */
  readonly min?: PlainDate | null;
  /** Latest selectable date, inclusive. @default undefined */
  readonly max?: PlainDate | null;
  /** Predicate for dates that stay focusable but cannot be selected. @default undefined */
  readonly isDateUnavailable?: DateMatcher;
  /** BCP 47 locale; defaults to the nearest LocaleProvider. @default undefined */
  readonly locale?: string;
  /** Intl calendar for display labels only. @default undefined */
  readonly calendar?: string;
  /** Intl numbering system for labels. @default undefined */
  readonly numberingSystem?: string;
  /** Text direction; defaults to the nearest LocaleProvider. @default undefined */
  readonly dir?: CalendarDirection;
  /** First day of week (`0` = Sunday); defaults to the locale preference. @default undefined */
  readonly weekStartsOn?: Weekday;
  /** Weekday column label width. @default "short" */
  readonly weekdayFormat?: CalendarWeekdayFormat;
  /** Number of consecutive months rendered. @default 1 */
  readonly numberOfMonths?: number;
  /** Move by `numberOfMonths` instead of one month with previous/next month controls. @default false */
  readonly pagedNavigation?: boolean;
  /** Always render six week rows per month. @default false */
  readonly fixedWeeks?: boolean;
  /** Explicit current date; the SSR-safe way to highlight today. @default undefined */
  readonly today?: PlainDate | null;
  /** Injectable clock evaluated during setup on server and client. @default undefined */
  readonly now?: DateTimeNow;
  /** IANA time zone used with `now` and the post-mount host clock. @default undefined */
  readonly timeZone?: string;
  /** Disable navigation, focus, and selection. @default false */
  readonly disabled?: boolean;
  /** Allow navigation while blocking selection. @default false */
  readonly readOnly?: boolean;
  /** Accessible name for the calendar group. @default undefined */
  readonly ariaLabel?: string;
  /** Ids that label the calendar group. @default undefined */
  readonly ariaLabelledby?: string;
  /** Ids that describe the calendar group. @default undefined */
  readonly ariaDescribedby?: string;
}

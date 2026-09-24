/** Accessible, unstyled two-click date range calendar that reuses the Calendar grid parts. */
export { default as RangeCalendar, default as RangeCalendarRoot } from "./range-calendar-root.vue";
export { default as RangeCalendarGrid } from "../calendar/calendar-grid.vue";
export { default as RangeCalendarHeading } from "../calendar/calendar-heading.vue";
export { default as RangeCalendarMonthSelect } from "../calendar/calendar-month-select.vue";
export { default as RangeCalendarNext } from "../calendar/calendar-next.vue";
export { default as RangeCalendarPrev } from "../calendar/calendar-prev.vue";
export { default as RangeCalendarYearSelect } from "../calendar/calendar-year-select.vue";
export type { RangeCalendarRootExpose } from "./range-calendar-types.ts";
export type {
  CalendarDayState,
  CalendarMonthState,
  CalendarSlotState,
} from "../calendar/calendar-types.ts";
export type { DateRange, PlainDate } from "../calendar/plain-date.ts";

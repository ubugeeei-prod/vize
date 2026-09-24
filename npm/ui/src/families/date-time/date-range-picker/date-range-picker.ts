/** Accessible, unstyled date range picker: start/end DateFields plus a popover RangeCalendar. */
// Date-model helpers lead every date-time entry (and the root entry) so the shared
// model chunks load first in both root and subpath bundles.
export { formatIsoDate, parseIsoDate } from "../calendar/plain-date.ts";
export {
  default as DateRangePicker,
  default as DateRangePickerRoot,
} from "./date-range-picker-root.vue";
export { default as DateRangePickerCalendar } from "./date-range-picker-calendar.vue";
export { default as DateRangePickerContent } from "./date-range-picker-content.vue";
export { default as DateRangePickerField } from "./date-range-picker-field.vue";
export { default as DateRangePickerTrigger } from "../../overlays/popover/popover-trigger.vue";
export { default as DateRangePickerArrow } from "../../overlays/popover/popover-arrow.vue";
export { default as DateRangePickerGrid } from "../calendar/calendar-grid.vue";
export { default as DateRangePickerHeading } from "../calendar/calendar-heading.vue";
export { default as DateRangePickerMonthSelect } from "../calendar/calendar-month-select.vue";
export { default as DateRangePickerNext } from "../calendar/calendar-next.vue";
export { default as DateRangePickerPrev } from "../calendar/calendar-prev.vue";
export { default as DateRangePickerYearSelect } from "../calendar/calendar-year-select.vue";
export type {
  DateRangeBoundary,
  DateRangePickerRootExpose,
  DateRangePickerSlotState,
} from "./date-range-picker-types.ts";
export type { DateMatcher, DateRange, PlainDate } from "../calendar/plain-date.ts";

/** Accessible, unstyled calendar that selects whole weeks and shows ISO week numbers. */
export { default as WeekPicker, default as WeekPickerRoot } from "./week-picker-root.vue";
export { default as WeekPickerGrid } from "../calendar/calendar-grid.vue";
export { default as WeekPickerHeading } from "../calendar/calendar-heading.vue";
export { default as WeekPickerMonthSelect } from "../calendar/calendar-month-select.vue";
export { default as WeekPickerNext } from "../calendar/calendar-next.vue";
export { default as WeekPickerPrev } from "../calendar/calendar-prev.vue";
export { default as WeekPickerYearSelect } from "../calendar/calendar-year-select.vue";
export { weekOf, type WeekSelection } from "./week-picker-selection.ts";
export type { WeekPickerRootExpose } from "./week-picker-types.ts";
export type { DateRange, IsoWeek, PlainDate } from "../calendar/plain-date.ts";

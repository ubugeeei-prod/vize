/** Accessible, unstyled paged grid of years. */
// Date-model helpers lead every date-time entry (and the root entry) so the shared
// model chunks load first in both root and subpath bundles.
export { formatIsoDate, parseIsoDate } from "../calendar/plain-date.ts";
export { default as YearPicker } from "./year-picker.vue";
export type {
  YearPickerCellState,
  YearPickerExpose,
  YearPickerSlotState,
} from "./year-picker-types.ts";
export type { PeriodPickerState } from "../month-picker/month-picker-types.ts";

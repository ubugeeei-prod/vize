/** Accessible, unstyled segmented time input with 12/24-hour clocks and hour/minute/second granularity. */
// Date-model helpers lead every date-time entry (and the root entry) so the shared
// model chunks load first in both root and subpath bundles.
export { formatIsoDate, parseIsoDate } from "../calendar/plain-date.ts";
export {
  compareTimes,
  createPlainTime,
  formatIsoTime,
  isSameTime,
  isValidPlainTime,
  normalizePlainTime,
  parseIsoTime,
  toSecondOfDay,
  truncateTime,
  type HourCycle,
  type PlainTime,
  type TimeGranularity,
} from "./plain-time.ts";
export { default as TimeField } from "./time-field.vue";
export type { TimeFieldExpose, TimeFieldSlotState } from "./time-field-types.ts";
export type {
  FieldSegmentPlaceholders,
  FieldSegmentState,
  SegmentedFieldState,
  TimeSegmentType,
} from "../date-field/date-field-types.ts";

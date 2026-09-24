/** Accessible, unstyled segmented time input with 12/24-hour clocks and hour/minute/second granularity. */
export { default as TimeField } from "./time-field.vue";
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
export type { TimeFieldExpose, TimeFieldSlotState } from "./time-field-types.ts";
export type {
  FieldSegmentPlaceholders,
  FieldSegmentState,
  SegmentedFieldState,
  TimeSegmentType,
} from "../date-field/date-field-types.ts";

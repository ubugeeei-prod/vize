/** Accessible, unstyled segmented date-and-time input in locale order with 12/24-hour clocks. */
export { default as DateTimeField } from "./datetime-field.vue";
export {
  addMinutes,
  combineDateTime,
  compareDateTimes,
  createPlainDateTime,
  formatIsoDateTime,
  fromEpochMillisecondsDateTime,
  isSameDateTime,
  minutesBetween,
  normalizePlainDateTime,
  parseIsoDateTime,
  toPlainDate,
  toPlainTime,
  type PlainDateTime,
  type PlainDateTimeLike,
} from "./plain-date-time.ts";
export type { DateTimeFieldExpose, DateTimeFieldSlotState } from "./datetime-field-types.ts";
export type {
  FieldSegmentPlaceholders,
  FieldSegmentState,
  SegmentedFieldState,
} from "../date-field/date-field-types.ts";

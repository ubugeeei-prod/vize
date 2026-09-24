/** Accessible, unstyled segmented duration input that reads and writes ISO 8601 durations. */
export { default as DurationField } from "./duration-field.vue";
export {
  durationToSeconds,
  durationUnits,
  formatIsoDuration,
  isSameDuration,
  normalizeDuration,
  parseIsoDuration,
  type DurationUnit,
  type DurationValue,
  type IsoDuration,
} from "./duration.ts";
export { durationUnitBounds, normalizeDurationFields } from "./duration-field-runtime.ts";
export type {
  DurationFieldExpose,
  DurationFieldSlotState,
  DurationSegmentState,
  DurationUnitDisplay,
} from "./duration-field-types.ts";

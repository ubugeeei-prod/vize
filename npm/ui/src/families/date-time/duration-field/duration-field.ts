/** Accessible, unstyled segmented duration input that reads and writes ISO 8601 durations. */
// Date-model helpers lead every date-time entry (and the root entry) so the shared
// model chunks load first in both root and subpath bundles.
export { formatIsoDate, parseIsoDate } from "../calendar/plain-date.ts";
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

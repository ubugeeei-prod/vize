/** Accessible, unstyled segmented date input whose segment order follows the locale. */
// Date-model helpers lead every date-time entry (and the root entry) so the shared
// model chunks load first in both root and subpath bundles.
export { formatIsoDate, parseIsoDate } from "../calendar/plain-date.ts";
export { default as DateField } from "./date-field.vue";
export type {
  DateFieldExpose,
  DateFieldSlotState,
  DateSegmentType,
  EditableSegmentType,
  FieldSegmentPlaceholders,
  FieldSegmentState,
  FieldSegmentType,
  SegmentedFieldExpose,
  SegmentedFieldSlotState,
  SegmentedFieldState,
  TimeSegmentType,
} from "./date-field-types.ts";
export type { DateMatcher, PlainDate } from "../calendar/plain-date.ts";

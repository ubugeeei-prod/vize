/**
 * Pure segment helpers shared by the segmented date and time fields. They live
 * in `field-segment-runtime.ts` so every field bundle shares one chunk.
 */
export {
  emptySegmentValues,
  isEditableSegmentType,
  resolveDateSegmentLayout,
  resolveHourCycle,
  resolveTimeSegmentLayout,
  resolveDateTimeSegmentLayout,
  resolveDayPeriodLabels,
  resolveSegmentNames,
  segmentBounds,
  segmentMaxDigits,
  segmentPageStep,
  stepSegmentValue,
  typeSegmentDigit,
  backspaceSegmentValue,
  type DateSegmentType,
  type TimeSegmentType,
  type EditableSegmentType,
  type FieldSegmentType,
  type FieldSegmentValues,
  type FieldSegmentLayoutPart,
  type SegmentHourCycle,
  type FieldSegmentBounds,
  type SegmentDigitResult,
} from "./field-segment-runtime.ts";

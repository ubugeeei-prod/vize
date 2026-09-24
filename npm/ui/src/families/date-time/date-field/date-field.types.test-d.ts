/** Compile-only assertions for the public DateField contract. */

import {
  DateField,
  type DateFieldExpose,
  type DateFieldSlotState,
  type EditableSegmentType,
  type FieldSegmentState,
  type PlainDate,
  type SegmentedFieldState,
} from "./date-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const field: DateFieldExpose;
declare const slot: DateFieldSlotState;
declare const segment: FieldSegmentState;

type _ValueIsDate = Expect<Equal<typeof field.value, PlainDate | null>>;
type _StateIsClosed = Expect<
  Equal<SegmentedFieldState, "complete" | "empty" | "invalid" | "partial">
>;
type _SegmentsAreReadonly = Expect<Equal<typeof slot.segments, readonly FieldSegmentState[]>>;
type _SegmentTypes = Expect<
  Equal<EditableSegmentType, "year" | "month" | "day" | "hour" | "minute" | "second" | "dayPeriod">
>;
type _SegmentValue = Expect<Equal<typeof segment.value, number | null>>;

const props: InstanceType<typeof DateField>["$props"] = {
  modelValue: { year: 2026, month: 9, day: 25 },
  min: null,
  placeholders: { year: "jjjj" },
  locale: "de-DE",
  name: "birthday",
  "onUpdate:modelValue": (value: PlainDate | null) => value,
};

field.focus("month");
field.setValue(null);

// @ts-expect-error segments are a closed union.
field.focus("week");

// @ts-expect-error placeholders are keyed by segment type.
const badPlaceholders: InstanceType<typeof DateField>["$props"] = { placeholders: { era: "AD" } };

void props;
void badPlaceholders;

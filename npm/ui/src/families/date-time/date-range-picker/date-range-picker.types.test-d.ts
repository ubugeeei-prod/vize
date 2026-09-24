/** Compile-only assertions for the public DateRangePicker contract. */

import {
  DateRangePicker,
  DateRangePickerField,
  DateRangePickerRoot,
  type DateRange,
  type DateRangeBoundary,
  type DateRangePickerRootExpose,
} from "./date-range-picker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: DateRangePickerRootExpose;

type _Value = Expect<Equal<typeof root.value, DateRange | null>>;
type _Boundary = Expect<Equal<DateRangeBoundary, "start" | "end">>;

const rootProps: InstanceType<typeof DateRangePickerRoot>["$props"] = {
  startName: "checkin",
  endName: "checkout",
  allowNonContiguousRanges: true,
  "onUpdate:modelValue": (value: DateRange | null) => value,
};
const fieldProps: InstanceType<typeof DateRangePickerField>["$props"] = { boundary: "end" };

// @ts-expect-error boundaries are start or end.
const badBoundary: InstanceType<typeof DateRangePickerField>["$props"] = { boundary: "middle" };

void DateRangePicker;
void rootProps;
void fieldProps;
void badBoundary;

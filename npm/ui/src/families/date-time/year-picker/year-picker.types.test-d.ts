/** Compile-only assertions for the public YearPicker contract. */

import { YearPicker, type YearPickerCellState, type YearPickerExpose } from "./year-picker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const picker: YearPickerExpose;
declare const cell: YearPickerCellState;

type _Value = Expect<Equal<typeof picker.value, number | null>>;
type _Cell = Expect<Equal<typeof cell.year, number>>;

const props: InstanceType<typeof YearPicker>["$props"] = {
  modelValue: 2026,
  pageSize: 20,
  min: 1900,
  isYearUnavailable: (year) => year % 2 === 0,
  "onUpdate:modelValue": (value: number | null) => value,
};

// @ts-expect-error years are numbers.
const bad: InstanceType<typeof YearPicker>["$props"] = { modelValue: "2026" };

void props;
void bad;

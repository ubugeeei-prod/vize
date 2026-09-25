/** Compile-only assertions for the public WeekPicker contract. */

import {
  WeekPicker,
  weekOf,
  type DateRange,
  type IsoWeek,
  type WeekPickerRootExpose,
} from "./week-picker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const picker: WeekPickerRootExpose;

type _Value = Expect<Equal<typeof picker.value, DateRange | null>>;
type _IsoWeek = Expect<Equal<typeof picker.isoWeek, IsoWeek | null>>;
type _WeekOf = Expect<Equal<ReturnType<typeof weekOf>, DateRange>>;

const props: InstanceType<typeof WeekPicker>["$props"] = {
  modelValue: weekOf({ year: 2026, month: 9, day: 25 }, 1),
  hideWeekNumbers: true,
  name: "week",
  "onUpdate:modelValue": (value: DateRange | null) => value,
};

// @ts-expect-error weeks are ranges.
const bad: InstanceType<typeof WeekPicker>["$props"] = { modelValue: { year: 2026, week: 39 } };

void props;
void bad;

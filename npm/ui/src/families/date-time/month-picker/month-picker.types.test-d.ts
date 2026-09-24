/** Compile-only assertions for the public MonthPicker contract. */

import {
  MonthPicker,
  parseIsoYearMonth,
  type MonthPickerExpose,
  type MonthPickerFormat,
  type PeriodPickerState,
  type PlainYearMonth,
} from "./month-picker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const picker: MonthPickerExpose;

type _Value = Expect<Equal<typeof picker.value, PlainYearMonth | null>>;
type _Year = Expect<Equal<typeof picker.year, number | null>>;
type _Format = Expect<
  Equal<MonthPickerFormat, "long" | "short" | "narrow" | "numeric" | "2-digit">
>;
type _State = Expect<
  Equal<PeriodPickerState, "disabled" | "empty" | "pending" | "readonly" | "selected">
>;
type _Parse = Expect<Equal<ReturnType<typeof parseIsoYearMonth>, PlainYearMonth | null>>;

const props: InstanceType<typeof MonthPicker>["$props"] = {
  modelValue: { year: 2026, month: 9 },
  min: { year: 2020, month: 1 },
  columns: 4,
  monthFormat: "long",
  isMonthUnavailable: (value) => value.month === 2,
  "onUpdate:modelValue": (value: PlainYearMonth | null) => value,
};

picker.navigate(1);

// @ts-expect-error month formats are Intl month widths.
const bad: InstanceType<typeof MonthPicker>["$props"] = { monthFormat: "tiny" };

void props;
void bad;

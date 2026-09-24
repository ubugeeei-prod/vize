/** Compile-only assertions for the public DateTimeField contract. */

import {
  DateTimeField,
  createPlainDateTime,
  parseIsoDateTime,
  type DateTimeFieldExpose,
  type PlainDateTime,
} from "./datetime-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const field: DateTimeFieldExpose;

type _Value = Expect<Equal<typeof field.value, PlainDateTime | null>>;
type _Parse = Expect<Equal<ReturnType<typeof parseIsoDateTime>, PlainDateTime | null>>;
type _Extends = Expect<
  Equal<PlainDateTime extends { readonly year: number; readonly hour: number } ? true : false, true>
>;

const props: InstanceType<typeof DateTimeField>["$props"] = {
  modelValue: createPlainDateTime(2026, 9, 25, 9, 30),
  hourCycle: 24,
  granularity: "second",
  isDateUnavailable: (date) => date.day === 1,
  "onUpdate:modelValue": (value: PlainDateTime | null) => value,
};

const bad: InstanceType<typeof DateTimeField>["$props"] = {
  // @ts-expect-error date-time values need hours and minutes.
  modelValue: { year: 2026, month: 1, day: 1 },
};

void props;
void bad;

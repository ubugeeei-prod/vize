/** Compile-only assertions for the public TimeField contract. */

import {
  TimeField,
  createPlainTime,
  type HourCycle,
  type PlainTime,
  type TimeFieldExpose,
  type TimeGranularity,
} from "./time-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const field: TimeFieldExpose;

type _ValueIsTime = Expect<Equal<typeof field.value, PlainTime | null>>;
type _HourCycle = Expect<Equal<HourCycle, 12 | 24>>;
type _Granularity = Expect<Equal<TimeGranularity, "hour" | "minute" | "second">>;
type _ExposeCycle = Expect<Equal<typeof field.hourCycle, HourCycle>>;

const props: InstanceType<typeof TimeField>["$props"] = {
  modelValue: createPlainTime(9, 30),
  hourCycle: 12,
  granularity: "second",
  min: { hour: 9, minute: 0, second: 0 },
  "onUpdate:modelValue": (value: PlainTime | null) => value,
};

// @ts-expect-error hour cycles are 12 or 24.
const badCycle: InstanceType<typeof TimeField>["$props"] = { hourCycle: 11 };

// @ts-expect-error granularity is hour, minute, or second.
const badGranularity: InstanceType<typeof TimeField>["$props"] = { granularity: "millisecond" };

void props;
void badCycle;
void badGranularity;

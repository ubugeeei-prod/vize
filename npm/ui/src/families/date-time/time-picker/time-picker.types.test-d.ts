/** Compile-only assertions for the public TimePicker contract. */

import {
  TimePicker,
  createTimeSlots,
  type PlainTime,
  type TimePickerExpose,
  type TimePickerSlot,
} from "./time-picker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const picker: TimePickerExpose;

type _Value = Expect<Equal<typeof picker.value, PlainTime | null>>;
type _Slots = Expect<Equal<typeof picker.slots, readonly TimePickerSlot[]>>;
type _Generated = Expect<Equal<ReturnType<typeof createTimeSlots>, readonly PlainTime[]>>;

const props: InstanceType<typeof TimePicker>["$props"] = {
  step: 15,
  min: { hour: 9, minute: 0, second: 0 },
  hourCycle: 12,
  isTimeUnavailable: (time) => time.hour === 12,
  "onUpdate:modelValue": (value: PlainTime | null) => value,
};

// @ts-expect-error hour cycles are 12 or 24.
const bad: InstanceType<typeof TimePicker>["$props"] = { hourCycle: 23 };

void props;
void bad;

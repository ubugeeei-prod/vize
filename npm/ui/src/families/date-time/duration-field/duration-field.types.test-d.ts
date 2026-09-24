/** Compile-only assertions for the public DurationField contract. */

import {
  DurationField,
  formatIsoDuration,
  type DurationFieldExpose,
  type DurationUnit,
  type DurationValue,
  type IsoDuration,
} from "./duration-field.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const field: DurationFieldExpose;

type _Value = Expect<Equal<typeof field.value, DurationValue | null>>;
type _Units = Expect<
  Equal<DurationUnit, "years" | "months" | "weeks" | "days" | "hours" | "minutes" | "seconds">
>;
type _IsoTyped = Expect<Equal<ReturnType<typeof formatIsoDuration>, IsoDuration>>;
type _IsoPrefix = Expect<Equal<IsoDuration, `P${string}`>>;

const iso: IsoDuration = "PT1H";
const props: InstanceType<typeof DurationField>["$props"] = {
  modelValue: { hours: 1, minutes: 30 },
  fields: ["hours", "minutes"],
  unitDisplay: "long",
  "onUpdate:modelValue": (value: DurationValue | null) => value,
};

// @ts-expect-error ISO durations start with P.
const badIso: IsoDuration = "1H";

// @ts-expect-error fields are duration units.
const badFields: InstanceType<typeof DurationField>["$props"] = { fields: ["fortnights"] };

void iso;
void props;
void badIso;
void badFields;

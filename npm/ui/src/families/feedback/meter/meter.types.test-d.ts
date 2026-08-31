/** Compile-only assertions for the public Meter contract. */

import type { MeterExpose, MeterRange, MeterSlotState, MeterState } from "./meter.ts";
import { Meter, getMeterState } from "./meter.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: MeterExpose;

type _RangeIsLiteral = Expect<Equal<MeterRange, "high" | "low" | "medium">>;
type _StateIsLiteral = Expect<
  Equal<MeterState, "empty" | "full" | "high" | "low" | "medium" | "optimum">
>;
type _ValueIsNumber = Expect<Equal<typeof exposed.value, number>>;
type _LowIsNullable = Expect<Equal<typeof exposed.low, number | null>>;
type _ElementIsNative = Expect<Equal<typeof exposed.element, HTMLMeterElement | null>>;
type _SlotStateIsExact = Expect<
  Equal<
    MeterSlotState,
    {
      readonly value: number;
      readonly min: number;
      readonly max: number;
      readonly low: number | null;
      readonly high: number | null;
      readonly optimum: number | null;
      readonly percent: number;
      readonly range: MeterRange;
      readonly optimal: boolean;
      readonly invalid: boolean;
      readonly state: MeterState;
    }
  >
>;

const props: InstanceType<typeof Meter>["$props"] = {
  ariaDescribedby: "usage-help",
  ariaLabel: "Storage usage",
  high: 90,
  low: 30,
  max: 100,
  min: 0,
  optimum: 50,
  value: 64,
};
const state = getMeterState(props);

// @ts-expect-error Meter state has a fixed token contract.
const invalidState: MeterState = "warning";

// @ts-expect-error Meter range has a fixed token contract.
const invalidRange: MeterRange = "optimum";

// @ts-expect-error value is numeric.
const badProps: InstanceType<typeof Meter>["$props"] = { value: "64" };

void badProps;
void invalidRange;
void invalidState;
void props;
void state;

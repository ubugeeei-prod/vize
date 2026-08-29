/** Compile-only assertions for the public Progress contract. */

import type { ProgressExpose, ProgressSlotState, ProgressState } from "./progress.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const progress: ProgressExpose;

type _ValueIsNullableNumber = Expect<Equal<typeof progress.value, number | null>>;
type _MaxIsNumber = Expect<Equal<typeof progress.max, number>>;
type _PercentIsNullableNumber = Expect<Equal<typeof progress.percent, number | null>>;
type _ElementIsNativeProgress = Expect<Equal<typeof progress.element, HTMLProgressElement | null>>;
type _StateIsLiteral = Expect<Equal<ProgressState, "complete" | "indeterminate" | "loading">>;
type _SlotStateIsLiteral = Expect<
  Equal<
    ProgressSlotState,
    {
      readonly value: number | null;
      readonly max: number;
      readonly percent: number | null;
      readonly indeterminate: boolean;
      readonly complete: boolean;
      readonly state: ProgressState;
    }
  >
>;

export const states: readonly ProgressState[] = ["loading", "complete", "indeterminate"];

// @ts-expect-error Progress has a closed state token contract.
export const invalidState: ProgressState = "paused";

const missingValue: ProgressSlotState = {
  // @ts-expect-error indeterminate progress represents missing value with null, not undefined.
  value: undefined,
  max: 100,
  percent: null,
  indeterminate: true,
  complete: false,
  state: "indeterminate",
};

void missingValue;

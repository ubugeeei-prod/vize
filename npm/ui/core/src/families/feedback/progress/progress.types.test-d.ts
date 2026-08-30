/** Compile-only assertions for the public Progress contract. */

import type { ComponentPublicInstance } from "vue";

import { Progress } from "./progress.ts";
import type {
  ProgressExpose,
  ProgressProps,
  ProgressSlots,
  ProgressSlotState,
  ProgressState,
} from "./progress.ts";

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
type _PropsFeedComponentProps = Expect<
  ProgressProps extends InstanceType<typeof Progress>["$props"] ? true : false
>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof ProgressProps,
    "ariaDescribedby" | "ariaLabel" | "ariaLabelledby" | "ariaValueText" | "id" | "max" | "value"
  >
>;
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
type _SlotsAreClosed = Expect<
  Equal<
    ProgressSlots,
    {
      readonly default?: (props: ProgressSlotState) => unknown;
    }
  >
>;
type _ExposeHasNativeElement = Expect<
  ProgressExpose extends { readonly element: HTMLProgressElement | null } ? true : false
>;
type _ExposeDoesNotUseProgressBarParts = Expect<
  Equal<Extract<keyof ProgressExpose, "indicator" | "root" | "track">, never>
>;
type _ProgressRootIsStillNative = Expect<
  Equal<ProgressExpose["element"], HTMLProgressElement | null>
>;
type _ProgressInstanceStillHasExpose = Expect<
  ComponentPublicInstance extends ProgressExpose ? false : true
>;

export const states: readonly ProgressState[] = ["loading", "complete", "indeterminate"];

const publicProps: ProgressProps = {
  ariaLabel: "Upload progress",
  id: "upload-progress",
  max: 100,
  value: 40,
};
const slotState: ProgressSlotState = {
  complete: false,
  indeterminate: false,
  max: 100,
  percent: 40,
  state: "loading",
  value: 40,
};

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

// @ts-expect-error Progress props reject unknown tone presets.
const invalidProps: ProgressProps = { tone: "success" };

void Progress;
void invalidProps;
void missingValue;
void publicProps;
void slotState;

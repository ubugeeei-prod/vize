/** Compile-only assertions for the public ProgressBar contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { ProgressBar } from "./progress-bar.ts";
import type {
  ProgressBarDirection,
  ProgressBarExpose,
  ProgressBarProps,
  ProgressBarSlots,
  ProgressBarSlotState,
  ProgressBarState,
  ProgressBarStyle,
} from "./progress-bar.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const progressBar: ProgressBarExpose;

type _ValueIsNullableNumber = Expect<Equal<typeof progressBar.value, number | null>>;
type _MinIsNumber = Expect<Equal<typeof progressBar.min, number>>;
type _MaxIsNumber = Expect<Equal<typeof progressBar.max, number>>;
type _PercentIsNullableNumber = Expect<Equal<typeof progressBar.percent, number | null>>;
type _RatioIsNullableNumber = Expect<Equal<typeof progressBar.ratio, number | null>>;
type _RootIsRenderable = Expect<
  Equal<typeof progressBar.root, Element | ComponentPublicInstance | null>
>;
type _TrackIsSpan = Expect<Equal<typeof progressBar.track, HTMLSpanElement | null>>;
type _IndicatorIsSpan = Expect<Equal<typeof progressBar.indicator, HTMLSpanElement | null>>;
type _StateIsLiteral = Expect<Equal<ProgressBarState, "complete" | "indeterminate" | "loading">>;
type _DirectionIsLiteral = Expect<Equal<ProgressBarDirection, "ltr" | "rtl">>;
type _PropsFeedComponentProps = Expect<
  ProgressBarProps extends InstanceType<typeof ProgressBar>["$props"] ? true : false
>;
type _PropsKeysAreClosed = Expect<
  Equal<
    keyof ProgressBarProps,
    | "ariaDescribedby"
    | "ariaLabel"
    | "ariaLabelledby"
    | "ariaValueText"
    | "as"
    | "dir"
    | "id"
    | "label"
    | "max"
    | "min"
    | "value"
    | "valueLabel"
  >
>;
type _StyleIsClosed = Expect<
  Equal<
    keyof ProgressBarStyle,
    | "--vize-ui-progress-bar-max"
    | "--vize-ui-progress-bar-min"
    | "--vize-ui-progress-bar-percent"
    | "--vize-ui-progress-bar-ratio"
    | "--vize-ui-progress-bar-value"
  >
>;
type _SlotStateIsLiteral = Expect<
  Equal<
    ProgressBarSlotState,
    {
      readonly value: number | null;
      readonly min: number;
      readonly max: number;
      readonly percent: number | null;
      readonly ratio: number | null;
      readonly dir: ProgressBarDirection;
      readonly indeterminate: boolean;
      readonly complete: boolean;
      readonly invalid: boolean;
      readonly state: ProgressBarState;
      readonly id: string;
      readonly labelId: string;
      readonly valueId: string;
      readonly style: ProgressBarStyle;
    }
  >
>;
type _SlotsAreClosed = Expect<
  Equal<
    ProgressBarSlots,
    {
      readonly default?: (props: ProgressBarSlotState) => unknown;
      readonly label?: (props: ProgressBarSlotState) => unknown;
      readonly value?: (props: ProgressBarSlotState) => unknown;
      readonly indicator?: (props: ProgressBarSlotState) => unknown;
    }
  >
>;

export const states: readonly ProgressBarState[] = ["loading", "complete", "indeterminate"];

const publicProps: ProgressBarProps = {
  ariaLabelledby: "upload-label",
  as: componentTarget,
  dir: "rtl",
  max: 100,
  min: 10,
  value: 40,
};
const slotState: ProgressBarSlotState = {
  complete: false,
  dir: "ltr",
  id: "upload-progress",
  indeterminate: false,
  invalid: false,
  labelId: "upload-progress-label",
  max: 100,
  min: 0,
  percent: 40,
  ratio: 0.4,
  state: "loading",
  style: {
    "--vize-ui-progress-bar-max": "100",
    "--vize-ui-progress-bar-min": "0",
    "--vize-ui-progress-bar-percent": "40%",
    "--vize-ui-progress-bar-ratio": "0.4",
    "--vize-ui-progress-bar-value": "40",
  },
  value: 40,
  valueId: "upload-progress-value",
};

// @ts-expect-error ProgressBar has a closed state token contract.
export const invalidState: ProgressBarState = "paused";

const missingValue: ProgressBarSlotState = {
  // @ts-expect-error indeterminate progress represents missing value with null, not undefined.
  value: undefined,
  dir: "ltr",
  id: "upload-progress",
  labelId: "upload-progress-label",
  max: 100,
  min: 0,
  percent: null,
  ratio: null,
  indeterminate: true,
  complete: false,
  invalid: false,
  state: "indeterminate",
  style: {
    "--vize-ui-progress-bar-max": "100",
    "--vize-ui-progress-bar-min": "0",
    "--vize-ui-progress-bar-percent": "0%",
    "--vize-ui-progress-bar-ratio": "0",
    "--vize-ui-progress-bar-value": "0",
  },
  valueId: "upload-progress-value",
};

// @ts-expect-error ProgressBar direction is intentionally narrow.
const invalidDirection: ProgressBarDirection = "auto";

// @ts-expect-error ProgressBar props reject unknown tone presets.
const invalidProps: ProgressBarProps = { tone: "success" };

void ProgressBar;
void invalidDirection;
void invalidProps;
void missingValue;
void publicProps;
void slotState;

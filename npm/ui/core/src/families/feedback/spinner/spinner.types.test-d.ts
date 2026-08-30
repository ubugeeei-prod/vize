/** Compile-only assertions for the public Spinner contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Spinner } from "./spinner.ts";
import type {
  SpinnerAriaState,
  SpinnerElement,
  SpinnerExpose,
  SpinnerProgressState,
  SpinnerRole,
  SpinnerSlotState,
  SpinnerState,
} from "./spinner.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: SpinnerExpose;
declare const componentTarget: Component;

type _RoleIsLiteral = Expect<Equal<SpinnerRole, "progressbar" | "status">>;
type _AriaStateIsLiteral = Expect<Equal<SpinnerAriaState, "decorative" | SpinnerRole>>;
type _ProgressStateIsLiteral = Expect<
  Equal<SpinnerProgressState, "determinate" | "indeterminate" | "none">
>;
type _StateIsLiteral = Expect<Equal<SpinnerState, "complete" | "hidden" | "idle" | "loading">>;
type _ElementIsRenderable = Expect<Equal<SpinnerElement, Element | ComponentPublicInstance>>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, SpinnerState>>;
type _ExposeValueIsNullableNumber = Expect<Equal<typeof exposed.value, number | null>>;
type _ExposePercentIsNullableNumber = Expect<Equal<typeof exposed.percent, number | null>>;
type _ExposeElementIsRenderable = Expect<Equal<typeof exposed.element, SpinnerElement | null>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    SpinnerSlotState,
    {
      readonly loading: boolean;
      readonly visible: boolean;
      readonly state: SpinnerState;
      readonly ariaState: SpinnerAriaState;
      readonly progressState: SpinnerProgressState;
      readonly value: number | null;
      readonly min: number;
      readonly max: number;
      readonly percent: number | null;
      readonly complete: boolean;
    }
  >
>;

const customHost: InstanceType<typeof Spinner>["$props"] = {
  ariaHidden: false,
  ariaLabel: "Uploading",
  ariaValueText: "1 of 2 chunks",
  as: componentTarget,
  atomic: false,
  id: "upload-spinner",
  loading: true,
  max: 2,
  min: 0,
  role: "progressbar",
  value: 1,
  visible: true,
};
const slotState: SpinnerSlotState = {
  ariaState: "progressbar",
  complete: false,
  loading: true,
  max: 100,
  min: 0,
  percent: null,
  progressState: "indeterminate",
  state: "loading",
  value: null,
  visible: true,
};

// @ts-expect-error Spinner role is intentionally status/progressbar only.
const invalidRole: SpinnerRole = "alert";

// @ts-expect-error Spinner has a closed state token contract.
const invalidState: SpinnerState = "spinning";

// @ts-expect-error component props require boolean loading.
const badLoading: InstanceType<typeof Spinner>["$props"] = { loading: "true" };

// @ts-expect-error progress value must be numeric or null.
const badValue: InstanceType<typeof Spinner>["$props"] = { value: "50" };

// @ts-expect-error slot state uses null for indeterminate value.
const badSlotState: SpinnerSlotState = { loading: true, value: undefined };

void Spinner;
void badLoading;
void badSlotState;
void badValue;
void customHost;
void invalidRole;
void invalidState;
void slotState;

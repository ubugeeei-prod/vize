/** Compile-only assertions for the public slider contract. */

import type {
  SliderAriaInvalid,
  SliderDirection,
  SliderEmits,
  SliderExpose,
  SliderOrientation,
  SliderProps,
  SliderSlotState,
  SliderSlots,
  SliderState,
  SliderStep,
} from "./slider.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const control: SliderExpose;
declare const slot: SliderSlotState;

type _ValueIsNumber = Expect<Equal<typeof control.value, number>>;
type _StepIsLiteral = Expect<Equal<SliderStep, number | "any">>;
type _OrientationIsClosed = Expect<Equal<SliderOrientation, "horizontal" | "vertical">>;
type _DirectionIsClosed = Expect<Equal<SliderDirection, "ltr" | "rtl">>;
type _StateIsClosed = Expect<
  Equal<SliderState, "disabled" | "in-range" | "invalid" | "max" | "min" | "readonly">
>;
type _InvalidStateIsNative = Expect<Equal<SliderAriaInvalid, boolean | "grammar" | "spelling">>;
type _SlotValueIsNumber = Expect<Equal<typeof slot.value, number>>;
type _SlotOrientationIsLiteral = Expect<Equal<typeof slot.orientation, SliderOrientation>>;
type _PropsModelValueIsNumber = Expect<Equal<NonNullable<SliderProps["modelValue"]>, number>>;
type _PropsStepAcceptsNativeAny = Expect<Equal<NonNullable<SliderProps["step"]>, SliderStep>>;
type _PropsInvalidIsAriaInvalid = Expect<
  Equal<NonNullable<SliderProps["ariaInvalid"]>, SliderAriaInvalid>
>;
type _UpdatePayloadIsNumber = Expect<Equal<SliderEmits["update:modelValue"], [value: number]>>;
type _InputPayloadIncludesNativeEvent = Expect<
  Equal<SliderEmits["input"], [value: number, nativeEvent: Event]>
>;
type _SlotContractUsesState = Expect<Equal<Parameters<SliderSlots["default"]>[0], SliderSlotState>>;

control.focus();
control.setValue(50);
control.stepUp();
control.stepDown(2);
control.reset();

const props = {
  ariaInvalid: "grammar",
  defaultValue: 25,
  dir: "rtl",
  max: 100,
  min: 0,
  step: "any",
} satisfies SliderProps;

// @ts-expect-error slider values are numbers.
control.setValue("50");

// @ts-expect-error slider direction is closed to native LTR and RTL.
const direction: SliderDirection = "auto";

// @ts-expect-error slider props reject non-numeric controlled values.
const invalidProps = { modelValue: "50" } satisfies SliderProps;

// @ts-expect-error slider state is closed to the public data contract.
const state: SliderState = "loading";

void direction;
void invalidProps;
void props;
void state;

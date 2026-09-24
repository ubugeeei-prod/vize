/** Compile-only assertions for the public RangeSlider contract. */

import {
  RangeSlider,
  RangeSliderRange,
  RangeSliderThumb,
  RangeSliderTrack,
  setRangeSliderThumb,
  type RangeSliderBounds,
  type RangeSliderChangeSource,
  type RangeSliderEmits,
  type RangeSliderExpose,
  type RangeSliderProps,
  type RangeSliderSlotState,
  type RangeSliderSlots,
  type RangeSliderState,
  type RangeSliderThumbSlotState,
  type RangeSliderValue,
} from "./range-slider.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const control: RangeSliderExpose;
declare const thumb: RangeSliderThumbSlotState;
declare const bounds: RangeSliderBounds;

type _ValueIsReadonlyNumbers = Expect<Equal<RangeSliderValue, readonly number[]>>;
type _StateIsClosed = Expect<Equal<RangeSliderState, "disabled" | "dragging" | "idle" | "invalid">>;
type _SourceIsClosed = Expect<Equal<RangeSliderChangeSource, "api" | "keyboard" | "pointer">>;
type _UpdatePayload = Expect<
  Equal<RangeSliderEmits["update:modelValue"], [value: RangeSliderValue]>
>;
type _ChangePayload = Expect<
  Equal<
    RangeSliderEmits["change"],
    [value: RangeSliderValue, thumbIndex: number, source: RangeSliderChangeSource]
  >
>;
type _SlotUsesState = Expect<
  Equal<Parameters<RangeSliderSlots["default"]>[0], RangeSliderSlotState>
>;
type _ExposeValues = Expect<Equal<typeof control.values, RangeSliderValue>>;
type _ActiveThumb = Expect<Equal<typeof control.activeThumb, number | null>>;
type _ThumbValue = Expect<Equal<typeof thumb.value, number>>;

const props = {
  defaultValue: [10, 90],
  dir: "rtl",
  getValueText: (value: number, index: number) => `${index}:${value}`,
  largeStep: 20,
  minStepsBetweenThumbs: 2,
  orientation: "vertical",
  step: 5,
} satisfies RangeSliderProps;
const rootProps: InstanceType<typeof RangeSlider>["$props"] = {
  ariaLabel: "Price",
  modelValue: [1, 2, 3],
  onChange: (value: RangeSliderValue, index: number, source: RangeSliderChangeSource) => {
    void value;
    void index;
    void source;
  },
};
const thumbProps: InstanceType<typeof RangeSliderThumb>["$props"] = { index: 0, ariaLabel: "Min" };
const trackProps: InstanceType<typeof RangeSliderTrack>["$props"] = {};
const rangeProps: InstanceType<typeof RangeSliderRange>["$props"] = {};

control.setThumbValue(1, 40);
control.setValue([5, 6]);
control.focusThumb(0);
setRangeSliderThumb([1, 2], 0, 1, bounds);

// @ts-expect-error thumb values are numbers.
control.setValue(["5"]);

// @ts-expect-error a thumb needs an index.
const missingIndex: InstanceType<typeof RangeSliderThumb>["$props"] = { ariaLabel: "Min" };

// @ts-expect-error orientation is closed.
const invalidOrientation = { orientation: "diagonal" } satisfies RangeSliderProps;

void invalidOrientation;
void missingIndex;
void props;
void rangeProps;
void rootProps;
void thumbProps;
void trackProps;

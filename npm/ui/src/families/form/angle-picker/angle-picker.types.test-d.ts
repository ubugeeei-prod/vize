/** Compile-only assertions for the public AnglePicker contract. */

import {
  AnglePicker,
  type AnglePickerChangeSource,
  type AnglePickerExpose,
  type AnglePickerSlotState,
} from "./angle-picker.ts";
import type { KnobSlotState } from "../knob/knob.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const picker: AnglePickerExpose;

type _SlotSharesKnobShape = Expect<Equal<AnglePickerSlotState, KnobSlotState>>;
type _AngleIsNumber = Expect<Equal<typeof picker.angle, number>>;

const props: InstanceType<typeof AnglePicker>["$props"] = {
  step: 15,
  largeStep: 45,
  getValueText: (angle: number) => `${angle}°`,
  onChange: (angle: number, source: AnglePickerChangeSource) => {
    void angle;
    void source;
  },
};

// @ts-expect-error angles are numeric.
const wrongModel: InstanceType<typeof AnglePicker>["$props"] = { modelValue: "90" };

void props;
void wrongModel;

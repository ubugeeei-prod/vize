/** Compile-only assertions for the public Knob contract. */

import { angleToValue, normalizeRotaryBounds, normalizeRotarySweep } from "./knob-geometry.ts";
import {
  Knob,
  type KnobChangeSource,
  type KnobExpose,
  type KnobState,
  type RotarySweep,
} from "./knob.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const knob: KnobExpose;

type _SourceIsClosed = Expect<Equal<KnobChangeSource, "api" | "keyboard" | "pointer" | "wheel">>;
type _StateIsClosed = Expect<Equal<KnobState, "disabled" | "dragging" | "idle" | "readonly">>;
type _AngleIsNumber = Expect<Equal<typeof knob.angle, number>>;
type _SweepShape = Expect<Equal<RotarySweep, { readonly start: number; readonly end: number }>>;

const props: InstanceType<typeof Knob>["$props"] = {
  min: -60,
  max: 12,
  step: 0.5,
  startAngle: -150,
  endAngle: 150,
  allowWheel: true,
  getValueText: (value: number) => `${value} dB`,
  onChange: (value: number, source: KnobChangeSource) => {
    void value;
    void source;
  },
};
const value: number = angleToValue(0, normalizeRotaryBounds({}), normalizeRotarySweep({}));

// @ts-expect-error values are numeric.
knob.setValue("5");

// @ts-expect-error value text formatters receive numbers.
const wrongText: InstanceType<typeof Knob>["$props"] = { getValueText: (value: string) => value };

void props;
void value;
void wrongText;

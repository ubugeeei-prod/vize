import { ColorWheel } from "../index.ts";
import type { ColorWheelProps } from "../index.ts";

type Expect<Condition extends true> = Condition;

type _PublicPropsFeedTheSfc = Expect<
  ColorWheelProps extends InstanceType<typeof ColorWheel>["$props"] ? true : false
>;

export const props = { modelValue: 180, label: "Hue" } satisfies ColorWheelProps;

// @ts-expect-error hue is a number of degrees.
export const badHue: ColorWheelProps = { modelValue: "red" };

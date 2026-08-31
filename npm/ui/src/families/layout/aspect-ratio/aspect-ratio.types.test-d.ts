/** Compile-only assertions for the public aspect ratio contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type {
  AspectRatioElement,
  AspectRatioExpose,
  AspectRatioSlotState,
  AspectRatioStyle,
} from "./aspect-ratio.ts";
import { AspectRatio } from "./aspect-ratio.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: AspectRatioExpose;
declare const componentTarget: Component;

type _ElementIsRenderable = Expect<Equal<AspectRatioElement, Element | ComponentPublicInstance>>;
type _RatioIsNumber = Expect<Equal<typeof exposed.ratio, number>>;
type _InvalidIsBoolean = Expect<Equal<typeof exposed.invalid, boolean>>;
type _SlotStateIsLiteral = Expect<
  Equal<AspectRatioSlotState, { readonly ratio: number; readonly invalid: boolean }>
>;
type _StyleIsIntrinsic = Expect<
  Equal<
    AspectRatioStyle,
    { readonly "--vize-ui-aspect-ratio": string; readonly aspectRatio: string }
  >
>;

const exposedElement: AspectRatioElement | null = exposed.element;
void AspectRatio;

const slotState: AspectRatioSlotState = { ratio: 16 / 9, invalid: false };
const style: AspectRatioStyle = {
  "--vize-ui-aspect-ratio": "1.7777777777777777",
  aspectRatio: "var(--vize-ui-aspect-ratio)",
};

// @ts-expect-error ratio state is always numeric.
const badSlotState: AspectRatioSlotState = { ratio: "16 / 9", invalid: false };

// @ts-expect-error invalid state is always boolean.
const badInvalid: AspectRatioSlotState = { ratio: 1, invalid: "false" };

// @ts-expect-error component props require a numeric ratio.
const badProps: InstanceType<typeof AspectRatio>["$props"] = { ratio: "16 / 9" };

const customHost: InstanceType<typeof AspectRatio>["$props"] = {
  as: componentTarget,
  ratio: 1,
};

void badSlotState;
void badInvalid;
void badProps;
void customHost;
void exposedElement;
void slotState;
void style;

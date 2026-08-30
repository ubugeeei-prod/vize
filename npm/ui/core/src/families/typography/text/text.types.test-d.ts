/** Compile-only assertions for the public Text contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Text } from "./text.ts";
import type {
  TextElement,
  TextExpose,
  TextSize,
  TextSlotState,
  TextTone,
  TextWeight,
} from "./text.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: TextExpose;

type _SizeIsLiteral = Expect<Equal<TextSize, "xs" | "sm" | "md" | "lg" | "xl">>;
type _WeightIsLiteral = Expect<Equal<TextWeight, "regular" | "medium" | "semibold" | "bold">>;
type _ToneIsLiteral = Expect<
  Equal<TextTone, "accent" | "danger" | "muted" | "neutral" | "success" | "warning">
>;
type _ElementIsRenderable = Expect<Equal<TextElement, Element | ComponentPublicInstance>>;
type _ExposeSizeIsLiteral = Expect<Equal<typeof exposed.size, TextSize>>;
type _ExposeWeightIsLiteral = Expect<Equal<typeof exposed.weight, TextWeight>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, TextTone>>;
type _ExposeTruncateIsBoolean = Expect<Equal<typeof exposed.truncate, boolean>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    TextSlotState,
    {
      readonly size: TextSize;
      readonly weight: TextWeight;
      readonly tone: TextTone;
      readonly truncate: boolean;
    }
  >
>;

const exposedElement: TextElement | null = exposed.element;
const customHost: InstanceType<typeof Text>["$props"] = {
  as: componentTarget,
  size: "sm",
  tone: "muted",
  truncate: true,
  weight: "medium",
};
const slotState: TextSlotState = {
  size: "xl",
  tone: "accent",
  truncate: false,
  weight: "bold",
};

// @ts-expect-error Text sizes are strict typography tokens.
const invalidSize: TextSize = "2xl";

// @ts-expect-error Text weights are strict typography tokens.
const invalidWeight: TextWeight = "black";

// @ts-expect-error Text tones are strict consumer styling tokens.
const invalidTone: TextTone = "info";

// @ts-expect-error truncate must stay boolean so data hooks are deterministic.
const badTruncateProp: InstanceType<typeof Text>["$props"] = { truncate: "true" };

void Text;
void badTruncateProp;
void customHost;
void exposedElement;
void invalidSize;
void invalidTone;
void invalidWeight;
void slotState;

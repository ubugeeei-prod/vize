/** Compile-only assertions for the public Heading contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Heading } from "./heading.ts";
import type {
  HeadingElement,
  HeadingExpose,
  HeadingLevel,
  HeadingSize,
  HeadingSlotState,
  HeadingTone,
  HeadingWeight,
} from "./heading.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: HeadingExpose;

type _LevelIsLiteral = Expect<Equal<HeadingLevel, 1 | 2 | 3 | 4 | 5 | 6>>;
type _SizeIsLiteral = Expect<Equal<HeadingSize, "xs" | "sm" | "md" | "lg" | "xl" | "2xl">>;
type _WeightIsLiteral = Expect<Equal<HeadingWeight, "regular" | "medium" | "semibold" | "bold">>;
type _ToneIsLiteral = Expect<
  Equal<HeadingTone, "accent" | "danger" | "muted" | "neutral" | "success" | "warning">
>;
type _ElementIsRenderable = Expect<Equal<HeadingElement, Element | ComponentPublicInstance>>;
type _ExposeLevelIsLiteral = Expect<Equal<typeof exposed.level, HeadingLevel>>;
type _ExposeSizeIsLiteral = Expect<Equal<typeof exposed.size, HeadingSize>>;
type _ExposeWeightIsLiteral = Expect<Equal<typeof exposed.weight, HeadingWeight>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, HeadingTone>>;
type _ExposeTruncateIsBoolean = Expect<Equal<typeof exposed.truncate, boolean>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    HeadingSlotState,
    {
      readonly level: HeadingLevel;
      readonly size: HeadingSize;
      readonly weight: HeadingWeight;
      readonly tone: HeadingTone;
      readonly truncate: boolean;
    }
  >
>;

const exposedElement: HeadingElement | null = exposed.element;
const customHost: InstanceType<typeof Heading>["$props"] = {
  as: componentTarget,
  level: 3,
  size: "lg",
  tone: "muted",
  truncate: true,
  weight: "medium",
};
const slotState: HeadingSlotState = {
  level: 1,
  size: "2xl",
  tone: "accent",
  truncate: false,
  weight: "bold",
};

// @ts-expect-error Heading levels are strict native heading levels.
const invalidLevel: HeadingLevel = 7;

// @ts-expect-error Heading sizes are strict visual tokens.
const invalidSize: HeadingSize = "3xl";

// @ts-expect-error Heading tones are strict consumer styling tokens.
const invalidTone: HeadingTone = "info";

// @ts-expect-error truncate must stay boolean so data hooks are deterministic.
const badTruncateProp: InstanceType<typeof Heading>["$props"] = { truncate: "true" };

void Heading;
void badTruncateProp;
void customHost;
void exposedElement;
void invalidLevel;
void invalidSize;
void invalidTone;
void slotState;

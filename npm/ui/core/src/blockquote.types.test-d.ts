/** Compile-only assertions for the public Blockquote contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Blockquote } from "./blockquote.ts";
import type {
  BlockquoteElement,
  BlockquoteExpose,
  BlockquoteSize,
  BlockquoteSlotState,
  BlockquoteTone,
} from "./blockquote.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: BlockquoteExpose;

type _SizeIsLiteral = Expect<Equal<BlockquoteSize, "sm" | "md" | "lg">>;
type _ToneIsLiteral = Expect<
  Equal<BlockquoteTone, "accent" | "danger" | "muted" | "neutral" | "success" | "warning">
>;
type _ElementIsRenderable = Expect<Equal<BlockquoteElement, Element | ComponentPublicInstance>>;
type _ExposeSizeIsLiteral = Expect<Equal<typeof exposed.size, BlockquoteSize>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, BlockquoteTone>>;
type _ExposeCiteIsNativeString = Expect<Equal<typeof exposed.cite, string | undefined>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    BlockquoteSlotState,
    {
      readonly size: BlockquoteSize;
      readonly tone: BlockquoteTone;
      readonly cite: string | undefined;
    }
  >
>;

const exposedElement: BlockquoteElement | null = exposed.element;
const customHost: InstanceType<typeof Blockquote>["$props"] = {
  as: componentTarget,
  cite: "https://example.com/source",
  size: "lg",
  tone: "muted",
};
const slotState: BlockquoteSlotState = {
  cite: undefined,
  size: "sm",
  tone: "accent",
};

// @ts-expect-error Blockquote sizes are strict typography tokens.
const invalidSize: BlockquoteSize = "xl";

// @ts-expect-error Blockquote tones are strict consumer styling tokens.
const invalidTone: BlockquoteTone = "info";

// @ts-expect-error cite must stay a native string URL attribute when provided.
const badCiteProp: InstanceType<typeof Blockquote>["$props"] = { cite: 42 };

void Blockquote;
void badCiteProp;
void customHost;
void exposedElement;
void invalidSize;
void invalidTone;
void slotState;

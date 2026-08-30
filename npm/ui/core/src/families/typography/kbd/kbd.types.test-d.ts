/** Compile-only assertions for the public Kbd contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Kbd } from "./kbd.ts";
import type { KbdElement, KbdExpose, KbdSize, KbdSlotState, KbdTone, KbdVariant } from "./kbd.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: KbdExpose;

type _SizeIsLiteral = Expect<Equal<KbdSize, "sm" | "md" | "lg">>;
type _VariantIsLiteral = Expect<Equal<KbdVariant, "key" | "shortcut" | "sequence">>;
type _ToneIsLiteral = Expect<
  Equal<KbdTone, "accent" | "danger" | "muted" | "neutral" | "success" | "warning">
>;
type _ElementIsRenderable = Expect<Equal<KbdElement, Element | ComponentPublicInstance>>;
type _ExposeSizeIsLiteral = Expect<Equal<typeof exposed.size, KbdSize>>;
type _ExposeVariantIsLiteral = Expect<Equal<typeof exposed.variant, KbdVariant>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, KbdTone>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    KbdSlotState,
    {
      readonly size: KbdSize;
      readonly variant: KbdVariant;
      readonly tone: KbdTone;
    }
  >
>;

const exposedElement: KbdElement | null = exposed.element;
const customHost: InstanceType<typeof Kbd>["$props"] = {
  as: componentTarget,
  size: "lg",
  tone: "muted",
  variant: "shortcut",
};
const slotState: KbdSlotState = {
  size: "sm",
  tone: "accent",
  variant: "sequence",
};

// @ts-expect-error Kbd sizes are strict visual tokens.
const invalidSize: KbdSize = "xl";

// @ts-expect-error Kbd variants are strict keyboard presentation tokens.
const invalidVariant: KbdVariant = "chord";

// @ts-expect-error Kbd tones are strict consumer styling tokens.
const invalidTone: KbdTone = "info";

// @ts-expect-error variant must use the literal KbdVariant contract.
const badVariantProp: InstanceType<typeof Kbd>["$props"] = { variant: "chord" };

void Kbd;
void badVariantProp;
void customHost;
void exposedElement;
void invalidSize;
void invalidTone;
void invalidVariant;
void slotState;

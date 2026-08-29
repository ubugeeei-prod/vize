/** Compile-only assertions for the public Badge contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Badge } from "./badge.ts";
import type {
  BadgeElement,
  BadgeExpose,
  BadgeSlotState,
  BadgeTone,
  BadgeVariant,
} from "./badge.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: BadgeExpose;

type _VariantIsLiteral = Expect<Equal<BadgeVariant, "count" | "label" | "status">>;
type _ToneIsLiteral = Expect<
  Equal<BadgeTone, "accent" | "danger" | "info" | "neutral" | "success" | "warning">
>;
type _ElementIsRenderable = Expect<Equal<BadgeElement, Element | ComponentPublicInstance>>;
type _ExposeVariantIsLiteral = Expect<Equal<typeof exposed.variant, BadgeVariant>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, BadgeTone>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    BadgeSlotState,
    {
      readonly variant: BadgeVariant;
      readonly tone: BadgeTone;
    }
  >
>;

const exposedElement: BadgeElement | null = exposed.element;
const customHost: InstanceType<typeof Badge>["$props"] = {
  as: componentTarget,
  tone: "success",
  variant: "status",
};
const slotState: BadgeSlotState = {
  tone: "danger",
  variant: "count",
};

// @ts-expect-error Badge variants are strict usage tokens.
const invalidVariant: BadgeVariant = "pill";

// @ts-expect-error Badge tones are strict consumer styling tokens.
const invalidTone: BadgeTone = "brand";

// @ts-expect-error variant must use the literal BadgeVariant contract.
const badVariantProp: InstanceType<typeof Badge>["$props"] = { variant: "chip" };

// @ts-expect-error tone must use the literal BadgeTone contract.
const badToneProp: InstanceType<typeof Badge>["$props"] = { tone: "notice" };

void Badge;
void badToneProp;
void badVariantProp;
void customHost;
void exposedElement;
void invalidTone;
void invalidVariant;
void slotState;

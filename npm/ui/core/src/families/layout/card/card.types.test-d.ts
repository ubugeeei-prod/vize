/** Compile-only assertions for the public Card contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Card } from "./card.ts";
import type {
  CardDensity,
  CardElement,
  CardExpose,
  CardSlotState,
  CardTone,
  CardVariant,
} from "./card.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: CardExpose;

type _VariantIsLiteral = Expect<Equal<CardVariant, "card" | "panel" | "surface">>;
type _DensityIsLiteral = Expect<Equal<CardDensity, "compact" | "comfortable" | "spacious">>;
type _ToneIsLiteral = Expect<
  Equal<CardTone, "neutral" | "accent" | "info" | "success" | "warning" | "danger">
>;
type _ElementIsRenderable = Expect<Equal<CardElement, Element | ComponentPublicInstance>>;
type _ExposeVariantIsLiteral = Expect<Equal<typeof exposed.variant, CardVariant>>;
type _ExposeDensityIsLiteral = Expect<Equal<typeof exposed.density, CardDensity>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, CardTone>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    CardSlotState,
    {
      readonly variant: CardVariant;
      readonly density: CardDensity;
      readonly tone: CardTone;
    }
  >
>;

const exposedElement: CardElement | null = exposed.element;
const customHost: InstanceType<typeof Card>["$props"] = {
  as: componentTarget,
  density: "compact",
  tone: "success",
  variant: "surface",
};
const slotState: CardSlotState = {
  density: "spacious",
  tone: "danger",
  variant: "panel",
};

// @ts-expect-error Card variants are strict usage tokens.
const invalidVariant: CardVariant = "tile";

// @ts-expect-error Card densities are strict spacing tokens.
const invalidDensity: CardDensity = "dense";

// @ts-expect-error Card tones are strict consumer styling tokens.
const invalidTone: CardTone = "brand";

// @ts-expect-error variant must use the literal CardVariant contract.
const badVariantProp: InstanceType<typeof Card>["$props"] = { variant: "callout" };

// @ts-expect-error density must use the literal CardDensity contract.
const badDensityProp: InstanceType<typeof Card>["$props"] = { density: "loose" };

// @ts-expect-error tone must use the literal CardTone contract.
const badToneProp: InstanceType<typeof Card>["$props"] = { tone: "notice" };

void Card;
void badDensityProp;
void badToneProp;
void badVariantProp;
void customHost;
void exposedElement;
void invalidDensity;
void invalidTone;
void invalidVariant;
void slotState;

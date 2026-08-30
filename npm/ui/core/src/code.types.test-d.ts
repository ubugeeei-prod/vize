/** Compile-only assertions for the public Code contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { Code } from "./code.ts";
import type {
  CodeElement,
  CodeExpose,
  CodeSize,
  CodeSlotState,
  CodeTone,
  CodeVariant,
} from "./code.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: CodeExpose;

type _SizeIsLiteral = Expect<Equal<CodeSize, "sm" | "md" | "lg">>;
type _VariantIsLiteral = Expect<Equal<CodeVariant, "inline" | "block" | "snippet">>;
type _ToneIsLiteral = Expect<
  Equal<CodeTone, "accent" | "danger" | "muted" | "neutral" | "success" | "warning">
>;
type _ElementIsRenderable = Expect<Equal<CodeElement, Element | ComponentPublicInstance>>;
type _ExposeSizeIsLiteral = Expect<Equal<typeof exposed.size, CodeSize>>;
type _ExposeVariantIsLiteral = Expect<Equal<typeof exposed.variant, CodeVariant>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, CodeTone>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    CodeSlotState,
    {
      readonly size: CodeSize;
      readonly variant: CodeVariant;
      readonly tone: CodeTone;
    }
  >
>;

const exposedElement: CodeElement | null = exposed.element;
const customHost: InstanceType<typeof Code>["$props"] = {
  as: componentTarget,
  size: "lg",
  tone: "muted",
  variant: "block",
};
const slotState: CodeSlotState = {
  size: "sm",
  tone: "accent",
  variant: "snippet",
};

// @ts-expect-error Code sizes are strict typography tokens.
const invalidSize: CodeSize = "xl";

// @ts-expect-error Code variants are strict presentation tokens.
const invalidVariant: CodeVariant = "terminal";

// @ts-expect-error Code tones are strict consumer styling tokens.
const invalidTone: CodeTone = "info";

// @ts-expect-error variant must use the literal CodeVariant contract.
const badVariantProp: InstanceType<typeof Code>["$props"] = { variant: "terminal" };

void Code;
void badVariantProp;
void customHost;
void exposedElement;
void invalidSize;
void invalidTone;
void invalidVariant;
void slotState;

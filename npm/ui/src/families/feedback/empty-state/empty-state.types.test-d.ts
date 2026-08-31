/** Compile-only assertions for the public EmptyState contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type {
  EmptyStateDensity,
  EmptyStateElement,
  EmptyStateExpose,
  EmptyStateOrientation,
  EmptyStateSlotState,
  EmptyStateState,
  EmptyStateTone,
} from "./empty-state.ts";
import { EmptyState } from "./empty-state.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: EmptyStateExpose;
declare const componentTarget: Component;

type _ElementIsRenderable = Expect<Equal<EmptyStateElement, Element | ComponentPublicInstance>>;
type _ToneIsLiteral = Expect<
  Equal<EmptyStateTone, "danger" | "info" | "neutral" | "success" | "warning">
>;
type _DensityIsLiteral = Expect<Equal<EmptyStateDensity, "compact" | "comfortable">>;
type _OrientationIsLiteral = Expect<Equal<EmptyStateOrientation, "block" | "inline">>;
type _StateIsLiteral = Expect<Equal<EmptyStateState, "empty">>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, EmptyStateTone>>;
type _ExposeDensityIsLiteral = Expect<Equal<typeof exposed.density, EmptyStateDensity>>;
type _ExposeOrientationIsLiteral = Expect<Equal<typeof exposed.orientation, EmptyStateOrientation>>;
type _ExposeStateIsLiteral = Expect<Equal<typeof exposed.state, "empty">>;
type _SlotStateIsLiteral = Expect<
  Equal<
    EmptyStateSlotState,
    {
      readonly tone: EmptyStateTone;
      readonly density: EmptyStateDensity;
      readonly orientation: EmptyStateOrientation;
      readonly state: EmptyStateState;
    }
  >
>;

const exposedElement: EmptyStateElement | null = exposed.element;
const customHost: InstanceType<typeof EmptyState>["$props"] = {
  as: componentTarget,
  density: "compact",
  orientation: "inline",
  tone: "danger",
};
const nativeHost: InstanceType<typeof EmptyState>["$props"] = {
  as: "article",
  density: "comfortable",
  orientation: "block",
  tone: "neutral",
};
const slotState: EmptyStateSlotState = {
  density: "comfortable",
  orientation: "block",
  state: "empty",
  tone: "neutral",
};

// @ts-expect-error EmptyState tone has a closed token contract.
const invalidTone: EmptyStateTone = "accent";

// @ts-expect-error EmptyState density has a closed token contract.
const invalidDensity: EmptyStateDensity = "dense";

// @ts-expect-error EmptyState orientation has a closed token contract.
const invalidOrientation: EmptyStateOrientation = "horizontal";

// @ts-expect-error EmptyState state is always empty.
const invalidState: EmptyStateState = "ready";

// @ts-expect-error component props require a known density token.
const badDensity: InstanceType<typeof EmptyState>["$props"] = { density: "dense" };

void EmptyState;
void badDensity;
void customHost;
void exposedElement;
void invalidDensity;
void invalidOrientation;
void invalidState;
void invalidTone;
void nativeHost;
void slotState;

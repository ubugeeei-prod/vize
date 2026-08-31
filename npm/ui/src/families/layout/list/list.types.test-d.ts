/** Compile-only assertions for the public List contract. */

import type { Component, ComponentPublicInstance } from "vue";

import { List } from "./list.ts";
import type {
  ListElement,
  ListExpose,
  ListMarker,
  ListSlotState,
  ListSpacing,
  ListTone,
} from "./list.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: ListExpose;

type _MarkerIsLiteral = Expect<Equal<ListMarker, "disc" | "decimal" | "none">>;
type _SpacingIsLiteral = Expect<Equal<ListSpacing, "compact" | "normal" | "loose">>;
type _ToneIsLiteral = Expect<
  Equal<ListTone, "accent" | "danger" | "muted" | "neutral" | "success" | "warning">
>;
type _ElementIsRenderable = Expect<Equal<ListElement, Element | ComponentPublicInstance>>;
type _ExposeMarkerIsLiteral = Expect<Equal<typeof exposed.marker, ListMarker>>;
type _ExposeSpacingIsLiteral = Expect<Equal<typeof exposed.spacing, ListSpacing>>;
type _ExposeToneIsLiteral = Expect<Equal<typeof exposed.tone, ListTone>>;
type _SlotStateIsLiteral = Expect<
  Equal<
    ListSlotState,
    {
      readonly marker: ListMarker;
      readonly spacing: ListSpacing;
      readonly tone: ListTone;
    }
  >
>;

const exposedElement: ListElement | null = exposed.element;
const customHost: InstanceType<typeof List>["$props"] = {
  as: componentTarget,
  marker: "decimal",
  spacing: "loose",
  tone: "muted",
};
const slotState: ListSlotState = {
  marker: "disc",
  spacing: "normal",
  tone: "accent",
};

// @ts-expect-error List markers are strict consumer styling tokens.
const invalidMarker: ListMarker = "circle";

// @ts-expect-error List spacing values are strict consumer layout tokens.
const invalidSpacing: ListSpacing = "relaxed";

// @ts-expect-error List tones are strict consumer styling tokens.
const invalidTone: ListTone = "info";

// @ts-expect-error marker must stay a strict token when provided as a prop.
const badMarkerProp: InstanceType<typeof List>["$props"] = { marker: "circle" };

void List;
void badMarkerProp;
void customHost;
void exposedElement;
void invalidMarker;
void invalidSpacing;
void invalidTone;
void slotState;

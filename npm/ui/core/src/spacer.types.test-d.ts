/** Compile-only assertions for the public spacer contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type {
  SpacerAxis,
  SpacerDisplay,
  SpacerElement,
  SpacerExpose,
  SpacerResolvedLayout,
  SpacerSize,
  SpacerState,
  SpacerStyle,
} from "./spacer.ts";
import { Spacer } from "./spacer.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const componentTarget: Component;
declare const exposed: SpacerExpose;

type _AxisIsLiteral = Expect<Equal<SpacerAxis, "block" | "inline" | "both">>;
type _DisplayIsLiteral = Expect<
  Equal<SpacerDisplay, "block" | "inline-block" | "flex" | "inline-flex" | "grid" | "inline-grid">
>;
type _ElementIsRenderable = Expect<Equal<SpacerElement, Element | ComponentPublicInstance>>;
type _SizeIsCssString = Expect<Equal<SpacerSize, string>>;
type _StateIsSized = Expect<Equal<SpacerState, "sized">>;
type _ExposeAxisIsLiteral = Expect<Equal<typeof exposed.axis, SpacerAxis>>;
type _ExposeInlineSizeIsString = Expect<Equal<typeof exposed.inlineSize, SpacerSize>>;
type _ExposeBlockSizeIsString = Expect<Equal<typeof exposed.blockSize, SpacerSize>>;
type _ExposeDisplayIsLiteral = Expect<Equal<typeof exposed.display, SpacerDisplay>>;
type _StyleIsLogical = Expect<
  Equal<
    SpacerStyle,
    {
      readonly "--vize-ui-spacer-inline-size": SpacerSize;
      readonly "--vize-ui-spacer-block-size": SpacerSize;
      readonly display: SpacerDisplay;
      readonly inlineSize: string;
      readonly blockSize: string;
    }
  >
>;

const exposedElement: SpacerElement | null = exposed.element;
const resolved: SpacerResolvedLayout = {
  axis: "both",
  blockSize: "1lh",
  display: "inline-block",
  inlineSize: "1lh",
  state: "sized",
  style: {
    "--vize-ui-spacer-block-size": "1lh",
    "--vize-ui-spacer-inline-size": "1lh",
    blockSize: "var(--vize-ui-spacer-block-size)",
    display: "inline-block",
    inlineSize: "var(--vize-ui-spacer-inline-size)",
  },
};
const customHost: InstanceType<typeof Spacer>["$props"] = {
  as: componentTarget,
  axis: "inline",
  blockSize: "1lh",
  display: "inline-grid",
  inlineSize: "var(--space-4)",
  size: "2rem",
};

// @ts-expect-error axis is intentionally limited to logical spacer directions.
const badAxis: SpacerAxis = "horizontal";

// @ts-expect-error size values must be native CSS strings with explicit units or keywords.
const badSize: InstanceType<typeof Spacer>["$props"] = { size: 2 };

// @ts-expect-error display is limited to modes that can retain logical size.
const badDisplay: InstanceType<typeof Spacer>["$props"] = { display: "contents" };

void Spacer;
void badAxis;
void badDisplay;
void badSize;
void customHost;
void exposedElement;
void resolved;

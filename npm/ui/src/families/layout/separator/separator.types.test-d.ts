/** Compile-only assertions for the public separator contract. */

import type { Component, ComponentPublicInstance } from "vue";

import type { SeparatorElement, SeparatorExpose, SeparatorOrientation } from "./separator.ts";
import { Separator } from "./separator.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const exposed: SeparatorExpose;
declare const componentTarget: Component;

type _OrientationIsLiteral = Expect<Equal<SeparatorOrientation, "horizontal" | "vertical">>;
type _ElementIsRenderable = Expect<Equal<SeparatorElement, Element | ComponentPublicInstance>>;
type _ExposeOrientationIsLiteral = Expect<Equal<typeof exposed.orientation, SeparatorOrientation>>;
type _ExposeDecorativeIsBoolean = Expect<Equal<typeof exposed.decorative, boolean>>;

const exposedElement: SeparatorElement | null = exposed.element;
const customHost: InstanceType<typeof Separator>["$props"] = {
  ariaLabel: "Pane boundary",
  ariaLabelledby: "pane-label",
  as: componentTarget,
  decorative: true,
  orientation: "vertical",
};

// @ts-expect-error orientation is intentionally limited to supported ARIA values.
const badOrientation: SeparatorOrientation = "block";

// @ts-expect-error component props reject unsupported orientation values.
const badProps: InstanceType<typeof Separator>["$props"] = { orientation: "block" };

// @ts-expect-error decorative is a boolean opt-out.
const badDecorative: InstanceType<typeof Separator>["$props"] = { decorative: "true" };

void Separator;
void badDecorative;
void badOrientation;
void badProps;
void customHost;
void exposedElement;

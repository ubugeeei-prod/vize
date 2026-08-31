/** Compile-only assertions for the public toggle contract. */

import type { Component, ComputedRef } from "vue";

import { Toggle } from "./toggle.ts";
import type { ToggleExpose, ToggleSlotState } from "./toggle.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const toggle: ToggleExpose;
declare const componentTarget: Component;

type ToggleProps = InstanceType<typeof Toggle>["$props"];

type _PressedIsComputedBoolean = Expect<Equal<typeof toggle.pressed, ComputedRef<boolean>>>;
type _TypePropIsLiteral = Expect<
  Equal<NonNullable<ToggleProps["type"]>, "button" | "reset" | "submit">
>;
type _ModelValueIsBoolean = Expect<Equal<Exclude<ToggleProps["modelValue"], undefined>, boolean>>;
type _DefaultPressedIsBoolean = Expect<
  Equal<Exclude<ToggleProps["defaultPressed"], undefined>, boolean>
>;
type _SlotStateIsLiteral = Expect<
  Equal<ToggleSlotState, { readonly disabled: boolean; readonly pressed: boolean }>
>;

const nativeHost: ToggleProps = {
  ariaLabel: "Bold",
  defaultPressed: true,
  type: "submit",
};
const customHost: ToggleProps = {
  as: componentTarget,
  disabled: false,
  modelValue: true,
  native: false,
};

toggle.setPressed(true);
toggle.reset();
toggle.focus();

// @ts-expect-error native button type is limited to platform button submit modes.
const badButtonType: ToggleProps = { type: "menu" };

// @ts-expect-error controlled pressed state is boolean.
const badModelValue: ToggleProps = { modelValue: "true" };

// @ts-expect-error disabled state is boolean.
const badDisabled: ToggleProps = { disabled: "true" };

// @ts-expect-error pressed values are booleans.
toggle.setPressed("true");

// @ts-expect-error slot state always includes the pressed boolean.
const missingPressed: ToggleSlotState = { disabled: false };

void Toggle;
void badButtonType;
void badDisabled;
void badModelValue;
void customHost;
void missingPressed;
void nativeHost;

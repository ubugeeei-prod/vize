/** Compile-only assertions for the public toggle contract. */

import type { ComputedRef } from "vue";

import type { ToggleExpose, ToggleSlotState } from "./toggle.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const toggle: ToggleExpose;

type _PressedIsComputedBoolean = Expect<Equal<typeof toggle.pressed, ComputedRef<boolean>>>;
type _SlotStateIsLiteral = Expect<
  Equal<ToggleSlotState, { readonly disabled: boolean; readonly pressed: boolean }>
>;

toggle.setPressed(true);
toggle.reset();
toggle.focus();

// @ts-expect-error pressed values are booleans.
toggle.setPressed("true");

// @ts-expect-error slot state always includes the pressed boolean.
const missingPressed: ToggleSlotState = { disabled: false };

void missingPressed;

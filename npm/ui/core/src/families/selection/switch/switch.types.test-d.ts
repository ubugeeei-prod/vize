/** Compile-only assertions for the public switch contract. */

import type { SwitchAriaInvalid, SwitchExpose, SwitchSlotState, SwitchState } from "./switch.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const control: SwitchExpose;
declare const slot: SwitchSlotState;

type _CheckedIsBoolean = Expect<Equal<typeof control.checked, boolean>>;
type _StateIsClosed = Expect<Equal<SwitchState, "checked" | "disabled" | "readonly" | "unchecked">>;
type _InvalidStateIsNative = Expect<Equal<SwitchAriaInvalid, boolean | "grammar" | "spelling">>;
type _SlotCheckedIsBoolean = Expect<Equal<typeof slot.checked, boolean>>;
type _SlotReadOnlyIsBoolean = Expect<Equal<typeof slot.readOnly, boolean>>;

control.focus();
control.toggle();
control.setChecked(true);
control.reset();

// @ts-expect-error switch checked state is boolean.
control.setChecked("true");

// @ts-expect-error switch states are closed to the public data contract.
const state: SwitchState = "mixed";

void state;

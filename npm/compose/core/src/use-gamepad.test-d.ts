/** Compile-only assertions for the `use-gamepad` type contracts. */

import { mapGamepad, standardGamepadAxes, useGamepad } from "./use-gamepad.ts";
import type { GamepadButtonState, GamepadHost, GamepadSnapshot } from "./use-gamepad.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const { gamepads } = useGamepad({
  mapping: { jump: { button: 0 }, moveX: { axis: standardGamepadAxes.leftStickX, deadzone: 0.1 } },
});
const mapped = gamepads.value[0]!.mapped;
type _Jump = Expect<Equal<typeof mapped.jump, GamepadButtonState>>;
type _MoveX = Expect<Equal<typeof mapped.moveX, number>>;
type _Keys = Expect<Equal<keyof typeof mapped, "jump" | "moveX">>;
type _AxisIndex = Expect<Equal<typeof standardGamepadAxes.leftStickX, 0>>;

declare const snapshot: GamepadSnapshot;
const direct = mapGamepad(snapshot, { fire: { button: 7 } });
type _Direct = Expect<Equal<typeof direct.fire, GamepadButtonState>>;

// @ts-expect-error unknown mapping keys are not present.
void mapped.crouch;

// @ts-expect-error the pad list is read-only.
gamepads.value = [];

// Real browser windows satisfy the host contract.
window satisfies GamepadHost;

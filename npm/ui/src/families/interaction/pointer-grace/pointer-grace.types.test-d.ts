/** Compile-only assertions for the public pointer-grace contract. */

import type { ShallowRef } from "vue";

import { createPointerGrace, type Point } from "./pointer-grace.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const origin: Point = { x: 0, y: 0 };
export const controller = createPointerGrace({ delay: 300 });

type _PendingIsReadonly = Expect<Equal<typeof controller.isPending, Readonly<ShallowRef<boolean>>>>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.isPending.value = true;
// @ts-expect-error delay must resolve to a number.
createPointerGrace({ delay: "soon" });

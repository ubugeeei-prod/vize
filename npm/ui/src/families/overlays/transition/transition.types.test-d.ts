/** Compile-only assertions for the public transition contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import { createTransition, type TransitionOptions } from "./transition.ts";
import type { PresenceStatus } from "../presence/presence-types.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const statuses: readonly PresenceStatus[] = ["entering", "exiting", "present", "unmounted"];

const present = ref(false);
export const controller = createTransition({
  present,
  timeoutPadding: () => 0,
});

type _StatusIsReadonly = Expect<
  Equal<typeof controller.status, Readonly<ShallowRef<PresenceStatus>>>
>;
type _OptionsAcceptPadding = Expect<
  Equal<NonNullable<TransitionOptions["timeoutPadding"]> extends never ? false : true, true>
>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.status.value = "present";
// @ts-expect-error timeoutPadding must resolve to a number.
createTransition({ timeoutPadding: "slow" });

/** Compile-only assertions for the public presence contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import { createPresence, type PresenceStatus, type PresenceProps } from "./presence.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const statuses: readonly PresenceStatus[] = ["entering", "exiting", "present", "unmounted"];
// @ts-expect-error hidden is not a presence phase.
export const invalidStatus: PresenceStatus = "hidden";

const present = ref(false);
export const controller = createPresence({
  present,
  respectReducedMotion: () => true,
});

type _PresentIsReadonly = Expect<Equal<typeof controller.isPresent, Readonly<ShallowRef<boolean>>>>;
type _StatusIsReadonly = Expect<
  Equal<typeof controller.status, Readonly<ShallowRef<PresenceStatus>>>
>;
type _PropsAreExact = Expect<Equal<typeof controller.presenceProps, Readonly<PresenceProps>>>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.isPresent.value = true;
// @ts-expect-error present must resolve to boolean.
createPresence({ present: "true" });

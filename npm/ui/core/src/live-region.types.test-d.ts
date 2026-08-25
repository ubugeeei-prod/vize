/** Compile-only assertions for the public live-region contract. */

import type { ShallowRef } from "vue";

import { createLiveRegion, type LiveRegionPoliteness } from "./live-region.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const politeness: readonly LiveRegionPoliteness[] = ["assertive", "polite"];
// @ts-expect-error off is not a live-region politeness.
export const invalidPoliteness: LiveRegionPoliteness = "off";

export const controller = createLiveRegion({ politeness: "polite" });

type _MessageIsReadonly = Expect<Equal<typeof controller.message, Readonly<ShallowRef<string>>>>;
type _PolitenessIsReadonly = Expect<
  Equal<typeof controller.politeness, Readonly<ShallowRef<LiveRegionPoliteness>>>
>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.message.value = "nope";
// @ts-expect-error politeness must resolve to the closed union.
createLiveRegion({ politeness: "off" });

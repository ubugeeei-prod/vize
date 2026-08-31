/** Compile-only assertions for the public announcer contract. */

import type { ShallowRef } from "vue";

import { createAnnouncer, createBusyAnnouncement, type AnnouncerPoliteness } from "./announcer.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const politeness: readonly AnnouncerPoliteness[] = ["assertive", "polite"];
// @ts-expect-error off is not an announcer politeness.
export const invalidPoliteness: AnnouncerPoliteness = "off";

export const announcer = createAnnouncer({ politeness: "polite" });

type _PoliteMessageIsReadonly = Expect<
  Equal<typeof announcer.politeMessage, Readonly<ShallowRef<string>>>
>;
type _PendingCountIsReadonly = Expect<
  Equal<typeof announcer.pendingCount, Readonly<ShallowRef<number>>>
>;
type _AnnounceReportsDeduplication = Expect<Equal<ReturnType<typeof announcer.announce>, boolean>>;

// @ts-expect-error consumers cannot mutate readonly reactive state.
announcer.politeMessage.value = "nope";
// @ts-expect-error politeness must resolve to the closed union.
announcer.announce("Saved", { politeness: "off" });
// @ts-expect-error politeness must resolve to the closed union.
createAnnouncer({ politeness: "off" });

export const busy = createBusyAnnouncement(announcer, { label: "Loading" });

type _BusyIsReadonly = Expect<Equal<typeof busy.isBusy, Readonly<ShallowRef<boolean>>>>;

// @ts-expect-error a busy announcement requires a label.
createBusyAnnouncement(announcer, {});
// @ts-expect-error consumers cannot mutate the busy flag.
busy.isBusy.value = false;

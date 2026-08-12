/** Compile-only assertions for the public document scroll-lock contract. */

import { ref } from "vue";
import type { ShallowRef } from "vue";

import {
  createScrollLock,
  type ScrollLockController,
  type ScrollLockStrategy,
} from "./scroll-lock.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const ownerDocument = ref<Document | null>(null);
const strategy = ref<ScrollLockStrategy>("auto");
export const controller: ScrollLockController = createScrollLock({
  document: ownerDocument,
  enabled: ref(true),
  preserveScrollbarGap: () => true,
  restoreScroll: false,
  strategy,
});

type _ActiveIsReadonly = Expect<Equal<typeof controller.isActive, Readonly<ShallowRef<boolean>>>>;
type _LockedIsReadonly = Expect<Equal<typeof controller.isLocked, Readonly<ShallowRef<boolean>>>>;
type _GapIsReadonly = Expect<Equal<typeof controller.scrollbarGap, Readonly<ShallowRef<number>>>>;

// @ts-expect-error strategy is a closed union.
createScrollLock({ document: ownerDocument, strategy: "position" });
// @ts-expect-error document retains DOM type safety.
createScrollLock({ document: window, strategy });
// @ts-expect-error enablement must resolve to boolean.
createScrollLock({ document: ownerDocument, enabled: ref("yes") });

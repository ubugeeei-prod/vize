/** Compile-only assertions for the public focus contracts. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import {
  createFocus,
  createFocusRing,
  createFocusWithin,
  type FocusEvent,
  type FocusMode,
  type FocusProps,
} from "./focus.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const modes: readonly FocusMode[] = ["target", "within"];
// @ts-expect-error focus ownership has no sibling-only mode.
export const invalidMode: FocusMode = "siblings";

const disabled = ref(false);
export const controller = createFocus({
  isDisabled: disabled,
  onFocus(event: FocusEvent) {
    const target: Element = event.target;
    const original: globalThis.FocusEvent | null = event.originalEvent;
    void [target, original];
  },
});
export const within = createFocusWithin({ isDisabled: () => false });
export const ring = createFocusRing({ autoFocus: true });

type _FocusedIsReadonly = Expect<Equal<typeof controller.isFocused, Readonly<ShallowRef<boolean>>>>;
type _VisibleIsReadonly = Expect<
  Equal<typeof controller.isFocusVisible, Readonly<ShallowRef<boolean>>>
>;
type _PropsAreExact = Expect<Equal<typeof controller.focusProps, Readonly<FocusProps>>>;

export const vueAttributes: HTMLAttributes = controller.focusProps;
// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.isFocused.value = true;
// @ts-expect-error disabled must resolve to boolean.
createFocus({ isDisabled: "false" });
// @ts-expect-error direct callbacks receive immutable library snapshots.
createFocus({ onFocus: (event: globalThis.FocusEvent) => event.preventDefault() });

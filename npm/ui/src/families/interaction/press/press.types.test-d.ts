/** Compile-only assertions for the public press contract. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import {
  createPress,
  type PressEvent,
  type PressKeyboardBehavior,
  type PressPointerType,
  type PressProps,
} from "./press.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const pointerTypes: readonly PressPointerType[] = [
  "keyboard",
  "mouse",
  "pen",
  "pointer",
  "touch",
  "virtual",
];
export const behavior: PressKeyboardBehavior = "button";

// @ts-expect-error pointerType is a closed, device-independent union.
export const invalidPointerType: PressPointerType = "trackpad";
// @ts-expect-error keyboard behavior does not accept arbitrary ARIA roles.
export const invalidBehavior: PressKeyboardBehavior = "menuitem";

const disabled = ref(false);
export const controller = createPress({
  isDisabled: disabled,
  keyboardBehavior: () => "link",
  onPress(event: PressEvent) {
    const target: Element = event.target;
    const native: Event | null = event.originalEvent;
    void target;
    void native;
  },
});

type _PressedIsReadonly = Expect<Equal<typeof controller.isPressed, Readonly<ShallowRef<boolean>>>>;
type _PropsAreExact = Expect<Equal<typeof controller.pressProps, Readonly<PressProps>>>;

export const vueAttributes: HTMLAttributes = controller.pressProps;
controller.pressProps.onClick(new MouseEvent("click"));
// @ts-expect-error consumers cannot mutate readonly reactive state directly.
controller.isPressed.value = true;
// @ts-expect-error isDisabled must resolve to boolean or undefined.
createPress({ isDisabled: "false" });
// @ts-expect-error callbacks receive the immutable PressEvent contract.
createPress({ onPress: (value: boolean) => value });

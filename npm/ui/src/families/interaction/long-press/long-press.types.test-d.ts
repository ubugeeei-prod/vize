/** Compile-only assertions for the public long-press contract. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import {
  createLongPress,
  type LongPressEvent,
  type LongPressPointerType,
  type LongPressProps,
} from "./long-press.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const pointerTypes: readonly LongPressPointerType[] = ["mouse", "pen", "pointer", "touch"];
// @ts-expect-error keyboard is an explicit alternative, not a held pointer family.
export const invalidPointerType: LongPressPointerType = "keyboard";

const threshold = ref(500);
export const controller = createLongPress({
  accessibilityDescription: () => "Hold for actions",
  pointerType: () => "touch",
  threshold,
  onLongPress(event: LongPressEvent) {
    const target: Element = event.target;
    void target;
  },
  onPress(event) {
    const keyboardAlternative: boolean = event.pointerType === "keyboard";
    void keyboardAlternative;
  },
});

type _PressedIsReadonly = Expect<Equal<typeof controller.isPressed, Readonly<ShallowRef<boolean>>>>;
type _LongIsReadonly = Expect<
  Equal<typeof controller.isLongPressed, Readonly<ShallowRef<boolean>>>
>;
type _PropsAreExact = Expect<Equal<typeof controller.longPressProps, Readonly<LongPressProps>>>;

export const vueAttributes: HTMLAttributes = controller.longPressProps;
// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.isLongPressed.value = true;
// @ts-expect-error threshold is finite milliseconds, never a string.
createLongPress({ threshold: "500" });
// @ts-expect-error descriptions must resolve to text.
createLongPress({ accessibilityDescription: 42 });

/** Compile-only assertions for the public move contract. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import { createMove, type MoveEvent, type MovePointerType, type MoveProps } from "./move.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const pointerTypes: readonly MovePointerType[] = [
  "keyboard",
  "mouse",
  "pen",
  "pointer",
  "touch",
];
// @ts-expect-error virtual activation has no continuous movement coordinate.
export const invalidPointerType: MovePointerType = "virtual";

const disabled = ref(false);
export const controller = createMove({
  isDisabled: disabled,
  keyboardStep: () => 10,
  onMove(event: MoveEvent) {
    if (event.type === "move") {
      const delta: number = event.deltaX;
      void delta;
    }
  },
});

type _MovingIsReadonly = Expect<Equal<typeof controller.isMoving, Readonly<ShallowRef<boolean>>>>;
type _PropsAreExact = Expect<Equal<typeof controller.moveProps, Readonly<MoveProps>>>;

export const vueAttributes: HTMLAttributes = controller.moveProps;
// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.isMoving.value = true;
// @ts-expect-error keyboard step must resolve to a number.
createMove({ keyboardStep: "10" });

/** Compile-only assertions for the public hover contract. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import { createHover, type HoverEvent, type HoverPointerType, type HoverProps } from "./hover.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const pointerTypes: readonly HoverPointerType[] = ["mouse", "pen"];
// @ts-expect-error touch does not have a persistent hover state.
export const invalidPointerType: HoverPointerType = "touch";

const disabled = ref(false);
export const controller = createHover({
  isDisabled: disabled,
  pointerType: () => "pen",
  onHoverStart(event: HoverEvent) {
    const target: Element = event.target;
    void target;
  },
});

type _HoveredIsReadonly = Expect<Equal<typeof controller.isHovered, Readonly<ShallowRef<boolean>>>>;
type _PropsAreExact = Expect<Equal<typeof controller.hoverProps, Readonly<HoverProps>>>;

export const vueAttributes: HTMLAttributes = controller.hoverProps;
// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.isHovered.value = true;
// @ts-expect-error disabled must resolve to boolean.
createHover({ isDisabled: "false" });

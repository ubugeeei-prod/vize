/** Compile-only assertions for the public interaction hook bundle. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import {
  createInteractionHooks,
  type InteractionHooksController,
  type InteractionHooksProps,
} from "./interaction-hooks.ts";
import type { FocusProps } from "../../accessibility/focus/focus.ts";
import type { HoverController, HoverProps } from "../hover/hover.ts";
import type { PressController, PressProps } from "../press/press.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const disabled = ref(false);
export const controller = createInteractionHooks({
  isDisabled: disabled,
  focusWithin: false,
  hover: { pointerType: () => "pen" as const },
  press: { keyboardBehavior: () => "link" as const },
});

type _ControllerShape = Expect<
  Equal<
    typeof controller,
    InteractionHooksController<
      typeof controller extends InteractionHooksController<infer Options> ? Options : never
    >
  >
>;
type _PressController = Expect<Equal<typeof controller.press, PressController>>;
type _HoverController = Expect<Equal<typeof controller.hover, HoverController>>;
type _FocusWithinDisabled = Expect<Equal<typeof controller.focusWithin, null>>;
type _PressedIsReadonly = Expect<Equal<typeof controller.isPressed, Readonly<ShallowRef<boolean>>>>;
type _PropsIncludeEnabledFamilies = Expect<
  Equal<typeof controller.interactionProps, Readonly<PressProps & HoverProps & FocusProps>>
>;

export const vueAttributes: HTMLAttributes = controller.interactionProps;
controller.interactionProps.onClick(new MouseEvent("click"));

export const withinOnly = createInteractionHooks({
  focusRing: false,
  focusWithin: {},
  hover: false,
  press: false,
});
type _WithinProps = Expect<Equal<typeof withinOnly.interactionProps, Readonly<FocusProps>>>;

// @ts-expect-error disabled press controllers are statically null.
withinOnly.press.cancel();
// @ts-expect-error touch does not have a persistent hover state.
createInteractionHooks({ hover: { pointerType: "touch" } });
// @ts-expect-error keyboard behavior is a closed activation contract.
createInteractionHooks({ press: { keyboardBehavior: "menuitem" } });
// @ts-expect-error consumers cannot mutate readonly reactive state directly.
controller.isPressed.value = true;
// @ts-expect-error combined props keep the exact enabled-family surface.
const missingClick: Pick<InteractionHooksProps<typeof controller>, "onClick"> = {};
void missingClick;

/** Compile-only assertions for the public interaction hook bundle. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import {
  createInteractionHooks,
  type InteractionHooksController,
  type InteractionHooksProps,
  type InteractionShortcutController,
  type InteractionShortcutProps,
} from "./interaction-hooks.ts";
import type { FocusProps } from "../../accessibility/focus/focus.ts";
import type {
  InteractionModality,
  InteractionModalityTracker,
} from "../../accessibility/interaction-modality/interaction-modality.ts";
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
  modality: { document: null, initialModality: "keyboard" },
  press: { keyboardBehavior: () => "link" as const },
  shortcuts: { platform: "standard" },
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
type _ModalityController = Expect<
  Equal<typeof controller.modalityTracker, InteractionModalityTracker>
>;
type _ShortcutController = Expect<
  Equal<typeof controller.shortcutController, InteractionShortcutController>
>;
type _FocusWithinDisabled = Expect<Equal<typeof controller.focusWithin, null>>;
type _PressedIsReadonly = Expect<Equal<typeof controller.isPressed, Readonly<ShallowRef<boolean>>>>;
type _ModalityIsReadonly = Expect<
  Equal<typeof controller.currentModality, Readonly<ShallowRef<InteractionModality | null>>>
>;
type _PropsIncludeEnabledFamilies = Expect<
  Equal<
    typeof controller.interactionProps,
    Readonly<PressProps & HoverProps & FocusProps & InteractionShortcutProps>
  >
>;

export const vueAttributes: HTMLAttributes = controller.interactionProps;
controller.interactionProps.onClick(new MouseEvent("click"));

export const withinOnly = createInteractionHooks({
  focusRing: false,
  focusWithin: {},
  hover: false,
  modality: false,
  press: false,
  shortcuts: false,
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

import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { createHover } from "../hover/hover.ts";
import { createPress } from "../press/press.ts";
import { createFocusRing, createFocusWithin } from "../../accessibility/focus/focus.ts";
import type { HoverOptions } from "../hover/hover.ts";
import type { PressOptions } from "../press/press.ts";
import type { FocusRingOptions, FocusWithinOptions } from "../../accessibility/focus/focus.ts";
import type {
  InteractionFeatureOptions,
  InteractionHooksCancelResult,
  InteractionHooksController,
  InteractionHooksOptions,
  InteractionHooksProps,
} from "./interaction-hooks-types.ts";

const setupDiagnostic = "VIZE_UI_INTERACTION_HOOKS_SETUP";

const falseState = shallowReadonly(shallowRef(false));

function withDisabled<
  Options extends { readonly isDisabled?: MaybeRefOrGetter<boolean | undefined> },
>(
  feature: Options | undefined,
  isDisabled: MaybeRefOrGetter<boolean | undefined> | undefined,
): Options {
  if (isDisabled === undefined) return (feature ?? {}) as Options;
  return { isDisabled, ...feature } as Options;
}

function enabled<Options>(
  feature: InteractionFeatureOptions<Options>,
): feature is Options | undefined {
  return feature !== false;
}

function readonlyState(
  controller: { readonly value: boolean } | Readonly<ShallowRef<boolean>> | null,
) {
  return controller ?? falseState;
}

function collectErrors(action: () => void, errors: unknown[]): void {
  try {
    action();
  } catch (error) {
    errors.push(error);
  }
}

function surfaceErrors(errors: readonly unknown[]): void {
  if (errors.length === 1) throw errors[0];
  if (errors.length > 1) throw new AggregateError(errors, "Interaction hook cleanup failed");
}

export function createInteractionHooks<const Options extends InteractionHooksOptions = {}>(
  options: Options = {} as Options,
): InteractionHooksController<Options> {
  const press = enabled(options.press)
    ? createPress(withDisabled(options.press, options.isDisabled))
    : null;
  const hover = enabled(options.hover)
    ? createHover(withDisabled(options.hover, options.isDisabled))
    : null;
  const focusRing = enabled(options.focusRing)
    ? createFocusRing(withDisabled(options.focusRing, options.isDisabled))
    : null;
  const focusWithin = enabled(options.focusWithin)
    ? createFocusWithin(withDisabled(options.focusWithin, options.isDisabled))
    : null;

  const interactionProps = Object.freeze({
    ...press?.pressProps,
    ...hover?.hoverProps,
    ...focusRing?.focusProps,
    ...focusWithin?.focusProps,
  }) as Readonly<InteractionHooksProps<Options>>;

  return Object.freeze({
    focusRing,
    focusWithin,
    hover,
    press,
    interactionProps,
    isFocused: readonlyState(focusRing?.isFocused ?? null),
    isFocusVisible: readonlyState(focusRing?.isFocusVisible ?? null),
    isFocusWithin: readonlyState(focusWithin?.isFocused ?? null),
    isHovered: readonlyState(hover?.isHovered ?? null),
    isPressed: readonlyState(press?.isPressed ?? null),
    cancel(): InteractionHooksCancelResult {
      return Object.freeze({
        focusRing: focusRing?.cancel() ?? false,
        focusWithin: focusWithin?.cancel() ?? false,
        hover: hover?.cancel() ?? false,
        press: press?.cancel() ?? false,
      });
    },
    dispose(): void {
      const errors: unknown[] = [];
      collectErrors(() => press?.dispose(), errors);
      collectErrors(() => hover?.dispose(), errors);
      collectErrors(() => focusRing?.dispose(), errors);
      collectErrors(() => focusWithin?.dispose(), errors);
      surfaceErrors(errors);
    },
  }) as InteractionHooksController<Options>;
}

export function useInteractionHooks<const Options extends InteractionHooksOptions = {}>(
  options: Options = {} as Options,
): InteractionHooksController<Options> {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createInteractionHooks(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  InteractionFeatureOptions,
  InteractionHooksCancelResult,
  InteractionHooksController,
  InteractionHooksOptions,
  InteractionHooksProps,
} from "./interaction-hooks-types.ts";
export type { FocusRingOptions, FocusWithinOptions, HoverOptions, PressOptions };

import { computed, ref, watch } from "vue";
import type { WritableComputedRef } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Options for {@link useScrollLock}. */
export interface UseScrollLockOptions {
  /**
   * Lock as soon as the element resolves.
   *
   * @default false
   */
  readonly initialValue?: boolean;

  /**
   * Also cancel `touchmove` on the element while locked (iOS Safari ignores
   * `overflow: hidden` for touch scrolling).
   *
   * @default true
   */
  readonly preventTouchMove?: boolean;
}

interface StyledElement extends Element {
  readonly style: Pick<CSSStyleDeclaration, "overflow" | "setProperty" | "getPropertyValue">;
}

/**
 * Lock scrolling of a single element by toggling `overflow: hidden`.
 *
 * The element's previous inline `overflow` is restored on unlock, target
 * change, and scope disposal. This is intentionally element-scoped: for the
 * document viewport (nested lock stacking, scrollbar-gap compensation,
 * scroll restoration) use `createScrollLock` from `@vizejs/ui/scroll-lock`.
 * Server renders never touch styles and report the requested initial value.
 *
 * @param target Reactive element target.
 * @param options Initial state and touch handling.
 * @default options {}
 * @returns Writable locked flag.
 */
export function useScrollLock(
  target: MaybeElementTarget,
  options: UseScrollLockOptions = {},
): WritableComputedRef<boolean> {
  const locked = ref(options.initialValue ?? false);
  const preventTouchMove = options.preventTouchMove ?? true;
  const onTouchMove = (event: Event): void => {
    if (event.cancelable) event.preventDefault();
  };

  const stop = watch(
    [() => resolveElement(target), locked],
    ([element, isLocked], _previous, onCleanup) => {
      if (!element || !isLocked || !isStyled(element)) return;
      const previous = element.style.overflow;
      element.style.setProperty("overflow", "hidden");
      if (preventTouchMove) element.addEventListener("touchmove", onTouchMove, { passive: false });
      onCleanup(() => {
        element.style.setProperty("overflow", previous);
        element.removeEventListener("touchmove", onTouchMove);
      });
    },
    { immediate: true, flush: "post" },
  );
  tryOnScopeDispose(() => stop.stop());

  return computed({
    get: () => locked.value,
    set: (next: boolean) => {
      locked.value = next;
    },
  });
}

function isStyled(element: Element): element is StyledElement {
  return "style" in element && typeof element.style === "object" && element.style !== null;
}

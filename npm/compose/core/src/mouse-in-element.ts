import { computed, readonly, ref, watch } from "vue";
import type { Ref } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { useMouse } from "./mouse.ts";
import type { MouseControls, UseMouseOptions } from "./mouse.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Options for {@link useMouseInElement}. */
export interface UseMouseInElementOptions extends Omit<UseMouseOptions, "type"> {
  /**
   * Keep updating element-relative coordinates while the pointer is outside
   * the element. When `false`, they freeze at the last inside position.
   *
   * @default true
   */
  readonly handleOutside?: boolean;
}

/** Reactive state returned by {@link useMouseInElement}. */
export interface MouseInElementControls extends MouseControls {
  /** Pointer position relative to the element's left edge. */
  readonly elementX: Readonly<Ref<number>>;
  /** Pointer position relative to the element's top edge. */
  readonly elementY: Readonly<Ref<number>>;
  /** Element left edge in viewport coordinates at the latest update. */
  readonly elementPositionX: Readonly<Ref<number>>;
  /** Element top edge in viewport coordinates at the latest update. */
  readonly elementPositionY: Readonly<Ref<number>>;
  /** Element width at the latest update. */
  readonly elementWidth: Readonly<Ref<number>>;
  /** Element height at the latest update. */
  readonly elementHeight: Readonly<Ref<number>>;
  /** Whether the pointer is outside the element. `true` during server rendering. */
  readonly isOutside: Readonly<Ref<boolean>>;
  /** Stop updating element-relative state. Idempotent. */
  readonly stop: () => void;
}

/**
 * Track the pointer position relative to an element.
 *
 * Built on {@link useMouse} in client coordinates: every move re-reads the
 * element rectangle, so transforms and scrolling are reflected. Server
 * renders report zero geometry and `isOutside: true`.
 *
 * @param target Reactive element target.
 * @param options Outside handling plus {@link useMouse} options.
 * @default options {}
 * @returns Pointer, element-relative, and geometry refs.
 */
export function useMouseInElement(
  target: MaybeElementTarget,
  options: UseMouseInElementOptions = {},
): MouseInElementControls {
  const handleOutside = options.handleOutside ?? true;
  const mouse = useMouse({ ...options, type: "client" });
  const elementX = ref(0);
  const elementY = ref(0);
  const elementPositionX = ref(0);
  const elementPositionY = ref(0);
  const elementWidth = ref(0);
  const elementHeight = ref(0);
  const isOutside = ref(true);
  const element = computed(() => resolveElement(target));

  const stopWatch = watch(
    [element, mouse.x, mouse.y, mouse.sourceType],
    ([currentElement, x, y, source]) => {
      if (!currentElement || source === null) return;
      const rect = currentElement.getBoundingClientRect();
      elementPositionX.value = rect.left;
      elementPositionY.value = rect.top;
      elementWidth.value = rect.width;
      elementHeight.value = rect.height;
      const relativeX = x - rect.left;
      const relativeY = y - rect.top;
      isOutside.value =
        rect.width === 0 ||
        rect.height === 0 ||
        relativeX < 0 ||
        relativeY < 0 ||
        relativeX > rect.width ||
        relativeY > rect.height;
      if (handleOutside || !isOutside.value) {
        elementX.value = relativeX;
        elementY.value = relativeY;
      }
    },
    { flush: "sync" },
  );
  const stop = (): void => stopWatch.stop();
  tryOnScopeDispose(stop);

  return {
    ...mouse,
    elementX: readonly(elementX),
    elementY: readonly(elementY),
    elementPositionX: readonly(elementPositionX),
    elementPositionY: readonly(elementPositionY),
    elementWidth: readonly(elementWidth),
    elementHeight: readonly(elementHeight),
    isOutside: readonly(isOutside),
    stop,
  };
}

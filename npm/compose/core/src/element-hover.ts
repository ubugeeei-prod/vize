import { readonly, ref, watch } from "vue";
import type { Ref } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Options for {@link useElementHover}. */
export interface UseElementHoverOptions {
  /**
   * Milliseconds to wait before reporting hover.
   *
   * @default 0
   */
  readonly delayEnter?: number;

  /**
   * Milliseconds to wait before reporting the end of hover.
   *
   * @default 0
   */
  readonly delayLeave?: number;

  /**
   * Owns the delay timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

/**
 * Track whether the pointer hovers an element.
 *
 * Uses `pointerenter`/`pointerleave` (which ignore descendants) with optional
 * enter/leave delays; a pending delay is cancelled by the opposite event.
 * Server renders report `false`; listeners and timers are released with the
 * owning reactive scope.
 *
 * @param target Reactive element target.
 * @param options Enter/leave delays and timer scheduler.
 * @default options {}
 * @returns Readonly hover flag.
 */
export function useElementHover(
  target: MaybeElementTarget,
  options: UseElementHoverOptions = {},
): Readonly<Ref<boolean>> {
  const { delayEnter = 0, delayLeave = 0, scheduler = defaultScheduler } = options;
  const hovered = ref(false);
  let handle: unknown;
  let pending = false;

  const cancel = (): void => {
    if (!pending) return;
    scheduler.clearTimeout(handle);
    pending = false;
  };
  const set = (next: boolean): void => {
    cancel();
    const delay = next ? delayEnter : delayLeave;
    if (delay <= 0) {
      hovered.value = next;
      return;
    }
    pending = true;
    handle = scheduler.setTimeout(() => {
      pending = false;
      hovered.value = next;
    }, delay);
  };
  const onEnter = (): void => set(true);
  const onLeave = (): void => set(false);

  const stop = watch(
    () => resolveElement(target),
    (element, _previous, onCleanup) => {
      if (!element) return;
      element.addEventListener("pointerenter", onEnter, { passive: true });
      element.addEventListener("pointerleave", onLeave, { passive: true });
      onCleanup(() => {
        element.removeEventListener("pointerenter", onEnter);
        element.removeEventListener("pointerleave", onLeave);
        cancel();
        hovered.value = false;
      });
    },
    { immediate: true, flush: "post" },
  );
  tryOnScopeDispose(() => {
    stop.stop();
    cancel();
  });

  return readonly(hovered);
}

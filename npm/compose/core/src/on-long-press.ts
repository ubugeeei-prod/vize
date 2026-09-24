import { watch } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Event modifiers applied to the pointer events handled by {@link onLongPress}. */
export interface LongPressModifiers {
  /** Call `stopPropagation()`. */
  readonly stop?: boolean;
  /** Fire at most once. */
  readonly once?: boolean;
  /** Call `preventDefault()`. */
  readonly prevent?: boolean;
  /** Listen in the capture phase. */
  readonly capture?: boolean;
  /** Only start when the element itself (not a descendant) is pressed. */
  readonly self?: boolean;
}

/** Summary passed to {@link OnLongPressOptions.onRelease}. */
export interface LongPressRelease {
  /** Milliseconds between press and release. */
  readonly duration: number;
  /** Pointer travel in CSS pixels. */
  readonly distance: number;
  /** Whether the long-press handler fired. */
  readonly isLongPress: boolean;
}

/** Options for {@link onLongPress}. */
export interface OnLongPressOptions {
  /**
   * Hold time in milliseconds.
   *
   * @default 500
   */
  readonly delay?: number;

  /**
   * Maximum pointer travel before the press is cancelled.
   *
   * @default 10
   */
  readonly distanceThreshold?: number;

  /**
   * Event modifiers.
   *
   * @default {}
   */
  readonly modifiers?: LongPressModifiers;

  /**
   * Called on every release with timing details.
   *
   * @default undefined
   */
  readonly onRelease?: (release: LongPressRelease, event: PointerEvent) => void;

  /**
   * Clock for durations.
   *
   * @default Date.now
   */
  readonly now?: () => number;

  /**
   * Owns the hold timer.
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
 * Call a handler when an element is pressed and held.
 *
 * Uses Pointer Events, so mouse, touch, and pen behave alike. Moving more than
 * `distanceThreshold` or releasing early cancels the press. Nothing is
 * registered during server rendering; listeners and the timer are released
 * with the owning scope or the returned stop function.
 *
 * @param target Reactive element target.
 * @param handler Called with the initiating `pointerdown` event.
 * @param options Delay, threshold, modifiers, and callbacks.
 * @default options {}
 * @returns Stop function.
 */
export function onLongPress(
  target: MaybeElementTarget,
  handler: (event: PointerEvent) => void,
  options: OnLongPressOptions = {},
): () => void {
  const {
    delay = 500,
    distanceThreshold = 10,
    modifiers = {},
    now = Date.now,
    scheduler = defaultScheduler,
  } = options;
  let handle: unknown;
  let pending = false;
  let fired = false;
  let used = false;
  let start: { readonly x: number; readonly y: number; readonly time: number } | null = null;

  const clear = (): void => {
    if (pending) scheduler.clearTimeout(handle);
    pending = false;
  };
  const apply = (event: PointerEvent): void => {
    if (modifiers.prevent && event.cancelable) event.preventDefault();
    if (modifiers.stop) event.stopPropagation();
  };

  const stopWatch = watch(
    () => resolveElement(target),
    (element, _previous, onCleanup) => {
      if (!element) return;
      const onDown = (event: Event): void => {
        if (!isPointerEvent(event) || (modifiers.once && used)) return;
        if (modifiers.self && event.target !== element) return;
        apply(event);
        clear();
        fired = false;
        start = { x: event.clientX, y: event.clientY, time: now() };
        pending = true;
        handle = scheduler.setTimeout(() => {
          pending = false;
          fired = true;
          used = true;
          handler(event);
        }, delay);
      };
      const onMove = (event: Event): void => {
        if (!isPointerEvent(event) || !start || !pending) return;
        if (Math.hypot(event.clientX - start.x, event.clientY - start.y) > distanceThreshold) {
          clear();
        }
      };
      const onUp = (event: Event): void => {
        if (!isPointerEvent(event) || !start) return;
        apply(event);
        const release: LongPressRelease = {
          duration: now() - start.time,
          distance: Math.hypot(event.clientX - start.x, event.clientY - start.y),
          isLongPress: fired,
        };
        clear();
        start = null;
        options.onRelease?.(release, event);
      };
      const listenerOptions: AddEventListenerOptions = { capture: modifiers.capture ?? false };
      element.addEventListener("pointerdown", onDown, listenerOptions);
      element.addEventListener("pointermove", onMove, listenerOptions);
      element.addEventListener("pointerup", onUp, listenerOptions);
      element.addEventListener("pointerleave", onUp, listenerOptions);
      element.addEventListener("pointercancel", onUp, listenerOptions);
      onCleanup(() => {
        const removeOptions: EventListenerOptions = { capture: modifiers.capture ?? false };
        element.removeEventListener("pointerdown", onDown, removeOptions);
        element.removeEventListener("pointermove", onMove, removeOptions);
        element.removeEventListener("pointerup", onUp, removeOptions);
        element.removeEventListener("pointerleave", onUp, removeOptions);
        element.removeEventListener("pointercancel", onUp, removeOptions);
        clear();
        start = null;
      });
    },
    { immediate: true, flush: "post" },
  );
  const stop = (): void => stopWatch.stop();
  tryOnScopeDispose(stop);
  return stop;
}

function isPointerEvent(event: Event): event is PointerEvent {
  return "pointerId" in event && "clientX" in event;
}

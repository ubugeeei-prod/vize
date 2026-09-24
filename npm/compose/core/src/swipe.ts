import { computed, readonly, ref, shallowReactive, shallowReadonly, watch } from "vue";
import type { ComputedRef, Ref } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import type { Point } from "./mouse.ts";
import type { PointerKind } from "./pointer.ts";
import { toPointerKind } from "./pointer.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Dominant direction of a swipe gesture. `"none"` below the threshold. */
export type SwipeDirection = "up" | "down" | "left" | "right" | "none";

/** Callbacks shared by {@link useSwipe} and {@link usePointerSwipe}. */
export interface SwipeCallbacks<GestureEvent extends Event> {
  /**
   * Called when a gesture starts.
   *
   * @default undefined
   */
  readonly onSwipeStart?: (event: GestureEvent) => void;

  /**
   * Called on every move after the threshold has been exceeded.
   *
   * @default undefined
   */
  readonly onSwipe?: (event: GestureEvent) => void;

  /**
   * Called when a gesture that exceeded the threshold ends.
   *
   * @default undefined
   */
  readonly onSwipeEnd?: (event: GestureEvent, direction: SwipeDirection) => void;
}

/** Options for {@link useSwipe}. */
export interface UseSwipeOptions extends SwipeCallbacks<TouchEvent> {
  /**
   * Minimum travel in CSS pixels before a gesture counts as a swipe.
   *
   * @default 50
   */
  readonly threshold?: number;

  /**
   * Register passive touch listeners. Set to `false` to call
   * `preventDefault()` on moves while swiping (for example to block
   * pull-to-refresh).
   *
   * @default true
   */
  readonly passive?: boolean;
}

/** Reactive gesture state shared by the swipe composables. */
export interface SwipeControls {
  /** Whether a gesture past the threshold is in progress. */
  readonly isSwiping: Readonly<Ref<boolean>>;
  /** Dominant direction of the current (or last) gesture. */
  readonly direction: ComputedRef<SwipeDirection>;
  /** Gesture start coordinates. */
  readonly coordsStart: Readonly<Point>;
  /** Latest gesture coordinates. */
  readonly coordsEnd: Readonly<Point>;
  /** `start.x - end.x`: positive when moving left. */
  readonly lengthX: ComputedRef<number>;
  /** `start.y - end.y`: positive when moving up. */
  readonly lengthY: ComputedRef<number>;
  /** Remove listeners. Idempotent. */
  readonly stop: () => void;
}

/**
 * Classify a displacement into a {@link SwipeDirection}.
 *
 * Pure helper shared by the swipe composables and usable for custom gesture
 * recognizers.
 *
 * @param lengthX `start.x - end.x`.
 * @param lengthY `start.y - end.y`.
 * @param threshold Minimum travel before a direction is reported.
 * @returns The dominant axis direction, or `"none"`.
 */
export function swipeDirection(
  lengthX: number,
  lengthY: number,
  threshold: number,
): SwipeDirection {
  if (Math.max(Math.abs(lengthX), Math.abs(lengthY)) < threshold) return "none";
  if (Math.abs(lengthX) > Math.abs(lengthY)) return lengthX > 0 ? "left" : "right";
  return lengthY > 0 ? "up" : "down";
}

interface GestureCore {
  readonly controls: Omit<SwipeControls, "stop">;
  readonly begin: (point: Point) => void;
  readonly move: (point: Point) => boolean;
  readonly reset: () => void;
  readonly swiping: Ref<boolean>;
}

function createGesture(threshold: number): GestureCore {
  const coordsStart = shallowReactive({ x: 0, y: 0 });
  const coordsEnd = shallowReactive({ x: 0, y: 0 });
  const swiping = ref(false);
  const lengthX = computed(() => coordsStart.x - coordsEnd.x);
  const lengthY = computed(() => coordsStart.y - coordsEnd.y);
  const direction = computed(() => swipeDirection(lengthX.value, lengthY.value, threshold));
  return {
    controls: {
      isSwiping: readonly(swiping),
      direction,
      coordsStart: shallowReadonly(coordsStart),
      coordsEnd: shallowReadonly(coordsEnd),
      lengthX,
      lengthY,
    },
    begin: (point) => {
      coordsStart.x = point.x;
      coordsStart.y = point.y;
      coordsEnd.x = point.x;
      coordsEnd.y = point.y;
    },
    move: (point) => {
      coordsEnd.x = point.x;
      coordsEnd.y = point.y;
      if (!swiping.value && direction.value !== "none") swiping.value = true;
      return swiping.value;
    },
    reset: () => {
      swiping.value = false;
    },
    swiping,
  };
}

/**
 * Detect touch swipes on an element.
 *
 * Tracks the first touch point from `touchstart` through `touchend`/
 * `touchcancel`. Direction is derived from the dominant axis once the travel
 * exceeds `threshold`. Server renders expose zeroed coordinates and
 * `direction: "none"`; listeners are removed with the owning scope.
 *
 * @param target Reactive element target.
 * @param options Threshold, passivity, and callbacks.
 * @default options {}
 * @returns Reactive gesture state and stop control.
 */
export function useSwipe(target: MaybeElementTarget, options: UseSwipeOptions = {}): SwipeControls {
  const gesture = createGesture(options.threshold ?? 50);
  const passive = options.passive ?? true;

  const firstTouch = (event: TouchEvent): Point | null => {
    const touch = event.touches[0] ?? event.changedTouches[0];
    return touch ? { x: touch.clientX, y: touch.clientY } : null;
  };
  const onStart = (event: Event): void => {
    if (!isTouchEvent(event)) return;
    const point = firstTouch(event);
    if (!point || event.touches.length > 1) return;
    gesture.begin(point);
    options.onSwipeStart?.(event);
  };
  const onMove = (event: Event): void => {
    if (!isTouchEvent(event)) return;
    const point = firstTouch(event);
    if (!point) return;
    if (!gesture.move(point)) return;
    if (!passive && event.cancelable) event.preventDefault();
    options.onSwipe?.(event);
  };
  const onEnd = (event: Event): void => {
    if (!isTouchEvent(event)) return;
    if (gesture.swiping.value) options.onSwipeEnd?.(event, gesture.controls.direction.value);
    gesture.reset();
  };

  const stopWatch = watch(
    () => resolveElement(target),
    (element, _previous, onCleanup) => {
      if (!element) return;
      const listenerOptions: AddEventListenerOptions = { passive };
      element.addEventListener("touchstart", onStart, { passive: true });
      element.addEventListener("touchmove", onMove, listenerOptions);
      element.addEventListener("touchend", onEnd, { passive: true });
      element.addEventListener("touchcancel", onEnd, { passive: true });
      onCleanup(() => {
        element.removeEventListener("touchstart", onStart);
        element.removeEventListener("touchmove", onMove);
        element.removeEventListener("touchend", onEnd);
        element.removeEventListener("touchcancel", onEnd);
      });
    },
    { immediate: true, flush: "post" },
  );
  const stop = (): void => stopWatch.stop();
  tryOnScopeDispose(stop);
  return { ...gesture.controls, stop };
}

/** Options for {@link usePointerSwipe}. */
export interface UsePointerSwipeOptions extends SwipeCallbacks<PointerEvent> {
  /**
   * Minimum travel in CSS pixels before a gesture counts as a swipe.
   *
   * @default 50
   */
  readonly threshold?: number;

  /**
   * Device kinds that may start a gesture.
   *
   * @default ["mouse", "touch", "pen"]
   */
  readonly pointerTypes?: readonly PointerKind[];

  /**
   * Suppress text selection on the element while a gesture is active.
   *
   * @default false
   */
  readonly disableTextSelect?: boolean;
}

/**
 * Detect swipes from any pointer device (mouse, touch, pen).
 *
 * Captures the pointer on `pointerdown` so moves outside the element still
 * count, and ignores secondary pointers during a gesture. Shares the state
 * shape and SSR defaults of {@link useSwipe}.
 *
 * @param target Reactive element target.
 * @param options Threshold, device filter, selection handling, and callbacks.
 * @default options {}
 * @returns Reactive gesture state and stop control.
 */
export function usePointerSwipe(
  target: MaybeElementTarget,
  options: UsePointerSwipeOptions = {},
): SwipeControls {
  const gesture = createGesture(options.threshold ?? 50);
  const pointerTypes = options.pointerTypes ?? ["mouse", "touch", "pen"];
  let activePointer: number | null = null;

  const accepts = (event: Event): event is PointerEvent => {
    if (!isPointerEvent(event)) return false;
    const kind = toPointerKind(event.pointerType);
    return kind !== null && pointerTypes.includes(kind);
  };
  const setSelection = (element: Element, disabled: boolean): void => {
    if (!options.disableTextSelect || !hasStyle(element)) return;
    element.style.setProperty("user-select", disabled ? "none" : "");
    element.style.setProperty("-webkit-user-select", disabled ? "none" : "");
  };

  const stopWatch = watch(
    () => resolveElement(target),
    (element, _previous, onCleanup) => {
      if (!element) return;
      const onDown = (event: Event): void => {
        if (!accepts(event) || activePointer !== null) return;
        if (event.pointerType === "mouse" && event.button !== 0) return;
        activePointer = event.pointerId;
        if (hasPointerCapture(element)) element.setPointerCapture(event.pointerId);
        gesture.begin({ x: event.clientX, y: event.clientY });
        setSelection(element, true);
        options.onSwipeStart?.(event);
      };
      const onMove = (event: Event): void => {
        if (!accepts(event) || event.pointerId !== activePointer) return;
        if (gesture.move({ x: event.clientX, y: event.clientY })) options.onSwipe?.(event);
      };
      const onUp = (event: Event): void => {
        if (!accepts(event) || event.pointerId !== activePointer) return;
        activePointer = null;
        if (hasPointerCapture(element)) element.releasePointerCapture(event.pointerId);
        if (gesture.swiping.value) options.onSwipeEnd?.(event, gesture.controls.direction.value);
        gesture.reset();
        setSelection(element, false);
      };
      element.addEventListener("pointerdown", onDown, { passive: true });
      element.addEventListener("pointermove", onMove, { passive: true });
      element.addEventListener("pointerup", onUp, { passive: true });
      element.addEventListener("pointercancel", onUp, { passive: true });
      onCleanup(() => {
        element.removeEventListener("pointerdown", onDown);
        element.removeEventListener("pointermove", onMove);
        element.removeEventListener("pointerup", onUp);
        element.removeEventListener("pointercancel", onUp);
        activePointer = null;
        gesture.reset();
        setSelection(element, false);
      });
    },
    { immediate: true, flush: "post" },
  );
  const stop = (): void => stopWatch.stop();
  tryOnScopeDispose(stop);
  return { ...gesture.controls, stop };
}

function isTouchEvent(event: Event): event is TouchEvent {
  return "touches" in event && "changedTouches" in event;
}

function isPointerEvent(event: Event): event is PointerEvent {
  return "pointerId" in event && "pointerType" in event;
}

function hasPointerCapture(
  element: Element,
): element is Element & Pick<HTMLElement, "setPointerCapture" | "releasePointerCapture"> {
  return typeof element.setPointerCapture === "function";
}

function hasStyle(element: Element): element is Element & ElementCSSInlineStyle {
  return "style" in element && typeof element.style === "object" && element.style !== null;
}

import { readonly, ref, shallowReactive, shallowReadonly, watch } from "vue";
import type { Ref } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import type { Point } from "./mouse.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Snapshot passed to pinch callbacks. */
export interface PinchSnapshot {
  /** Scale relative to the start of the gesture (`1` = unchanged). */
  readonly scale: number;
  /** Rotation relative to the start of the gesture, in degrees. */
  readonly rotation: number;
  /** Midpoint between the two pointers in client coordinates. */
  readonly origin: Point;
  /** Current distance between the two pointers. */
  readonly distance: number;
}

/** Options for {@link usePinch}. */
export interface UsePinchOptions {
  /**
   * Called when a second pointer starts a pinch.
   *
   * @default undefined
   */
  readonly onPinchStart?: (snapshot: PinchSnapshot) => void;

  /**
   * Called on every move while pinching.
   *
   * @default undefined
   */
  readonly onPinch?: (snapshot: PinchSnapshot) => void;

  /**
   * Called when one of the two pointers lifts.
   *
   * @default undefined
   */
  readonly onPinchEnd?: (snapshot: PinchSnapshot) => void;
}

/** Reactive gesture state returned by {@link usePinch}. */
export interface PinchControls {
  /** Whether two pointers are currently pinching. */
  readonly isPinching: Readonly<Ref<boolean>>;
  /** Scale relative to the gesture start; stays at the final value after the gesture. */
  readonly scale: Readonly<Ref<number>>;
  /** Rotation relative to the gesture start in degrees. */
  readonly rotation: Readonly<Ref<number>>;
  /** Distance between the pointers when the gesture started. */
  readonly initialDistance: Readonly<Ref<number>>;
  /** Current distance between the pointers. */
  readonly distance: Readonly<Ref<number>>;
  /** Midpoint between the pointers. */
  readonly origin: Readonly<Point>;
  /** Restore scale `1` and rotation `0`. */
  readonly reset: () => void;
  /** Remove listeners. Idempotent. */
  readonly stop: () => void;
}

/**
 * Recognize two-pointer pinch/rotate gestures on an element.
 *
 * Tracks active pointers with Pointer Events (touch, pen, or mouse chords).
 * The first two pointers define the gesture; additional pointers are
 * ignored. Pair it with `touch-action: none` on the element so the browser
 * does not claim the gesture for zooming. Server renders expose scale `1`,
 * rotation `0`, and `isPinching: false`.
 *
 * @param target Reactive element target.
 * @param options Gesture lifecycle callbacks.
 * @default options {}
 * @returns Reactive gesture state plus reset/stop controls.
 */
export function usePinch(target: MaybeElementTarget, options: UsePinchOptions = {}): PinchControls {
  const isPinching = ref(false);
  const scale = ref(1);
  const rotation = ref(0);
  const initialDistance = ref(0);
  const distance = ref(0);
  const origin = shallowReactive({ x: 0, y: 0 });
  const pointers = new Map<number, Point>();
  let initialAngle = 0;

  const pair = (): readonly [Point, Point] | null => {
    const [first, second] = pointers.values();
    return first && second ? [first, second] : null;
  };
  const snapshot = (): PinchSnapshot => ({
    scale: scale.value,
    rotation: rotation.value,
    origin: { x: origin.x, y: origin.y },
    distance: distance.value,
  });
  const measure = (first: Point, second: Point): { distance: number; angle: number } => {
    origin.x = (first.x + second.x) / 2;
    origin.y = (first.y + second.y) / 2;
    return {
      distance: Math.hypot(second.x - first.x, second.y - first.y),
      angle: (Math.atan2(second.y - first.y, second.x - first.x) * 180) / Math.PI,
    };
  };

  const onDown = (event: Event): void => {
    if (!isPointerEvent(event) || pointers.size >= 2) return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    const points = pair();
    if (!points) return;
    const measured = measure(...points);
    initialDistance.value = measured.distance;
    distance.value = measured.distance;
    initialAngle = measured.angle;
    scale.value = 1;
    rotation.value = 0;
    isPinching.value = true;
    options.onPinchStart?.(snapshot());
  };
  const onMove = (event: Event): void => {
    if (!isPointerEvent(event) || !pointers.has(event.pointerId)) return;
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
    const points = pair();
    if (!points || !isPinching.value) return;
    const measured = measure(...points);
    distance.value = measured.distance;
    scale.value = initialDistance.value === 0 ? 1 : measured.distance / initialDistance.value;
    rotation.value = normalizeDegrees(measured.angle - initialAngle);
    options.onPinch?.(snapshot());
  };
  const onUp = (event: Event): void => {
    if (!isPointerEvent(event) || !pointers.delete(event.pointerId)) return;
    if (!isPinching.value) return;
    isPinching.value = false;
    options.onPinchEnd?.(snapshot());
  };

  const stopWatch = watch(
    () => resolveElement(target),
    (element, _previous, onCleanup) => {
      if (!element) return;
      const listenerOptions: AddEventListenerOptions = { passive: true };
      element.addEventListener("pointerdown", onDown, listenerOptions);
      element.addEventListener("pointermove", onMove, listenerOptions);
      for (const type of ["pointerup", "pointercancel", "pointerleave"] as const) {
        element.addEventListener(type, onUp, listenerOptions);
      }
      onCleanup(() => {
        element.removeEventListener("pointerdown", onDown);
        element.removeEventListener("pointermove", onMove);
        for (const type of ["pointerup", "pointercancel", "pointerleave"] as const) {
          element.removeEventListener(type, onUp);
        }
        pointers.clear();
        isPinching.value = false;
      });
    },
    { immediate: true, flush: "post" },
  );
  const stop = (): void => stopWatch.stop();
  tryOnScopeDispose(stop);

  return {
    isPinching: readonly(isPinching),
    scale: readonly(scale),
    rotation: readonly(rotation),
    initialDistance: readonly(initialDistance),
    distance: readonly(distance),
    origin: shallowReadonly(origin),
    reset: () => {
      scale.value = 1;
      rotation.value = 0;
    },
    stop,
  };
}

function normalizeDegrees(value: number): number {
  const wrapped = ((((value + 180) % 360) + 360) % 360) - 180;
  return wrapped === -180 ? 180 : wrapped;
}

function isPointerEvent(event: Event): event is PointerEvent {
  return "pointerId" in event && "clientX" in event;
}

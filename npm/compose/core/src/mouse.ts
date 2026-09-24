import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Coordinate space reported by {@link useMouse}. */
export type MouseCoordinateType = "page" | "client" | "screen" | "movement";

/** Input device that produced the latest coordinates. */
export type MouseSourceType = "mouse" | "touch";

/** Two-dimensional coordinate in CSS pixels. */
export interface Point {
  /** Horizontal coordinate. */
  readonly x: number;
  /** Vertical coordinate. */
  readonly y: number;
}

/**
 * Custom coordinate extractor. Return `null` to ignore an event.
 *
 * Receives the mouse event, or the first touch point for touch input.
 */
export type MouseCoordinateExtractor = (input: MouseEvent | Touch) => Point | null;

/** Options for {@link useMouse}. */
export interface UseMouseOptions {
  /**
   * Coordinate space, or a custom extractor.
   *
   * @default "page"
   */
  readonly type?: MouseCoordinateType | MouseCoordinateExtractor;

  /**
   * Event target to listen on.
   *
   * @default globalThis.window when available
   */
  readonly target?: MaybeRefOrGetter<EventTarget | null | undefined>;

  /**
   * Track touch input in addition to mouse input.
   *
   * @default true
   */
  readonly touch?: boolean;

  /**
   * Restore `initialValue` when a touch ends.
   *
   * @default false
   */
  readonly resetOnTouchEnds?: boolean;

  /**
   * Coordinates exposed before any input and during server rendering.
   *
   * @default { x: 0, y: 0 }
   */
  readonly initialValue?: Point;
}

/** Reactive pointer position returned by {@link useMouse}. */
export interface MouseControls {
  /** Latest horizontal coordinate. */
  readonly x: Readonly<Ref<number>>;
  /** Latest vertical coordinate. */
  readonly y: Readonly<Ref<number>>;
  /** Device that produced the latest coordinates; `null` before any input. */
  readonly sourceType: Readonly<Ref<MouseSourceType | null>>;
}

/**
 * Track the mouse (and optionally touch) position.
 *
 * Listens passively for `mousemove`, `dragover`, and touch events on the
 * target; listeners follow the reactive target and are removed with the
 * owning reactive scope. Server renders expose `initialValue` with a `null`
 * source type.
 *
 * @param options Coordinate space, target, touch handling, and initial value.
 * @default options {}
 * @returns Reactive coordinates and input source.
 */
export function useMouse(options: UseMouseOptions = {}): MouseControls {
  const { touch = true, resetOnTouchEnds = false } = options;
  const initial = options.initialValue ?? { x: 0, y: 0 };
  const extract = extractorFor(options.type ?? "page");
  const x = ref(initial.x);
  const y = ref(initial.y);
  const sourceType = ref<MouseSourceType | null>(null);

  const apply = (point: Point | null, source: MouseSourceType): void => {
    if (!point) return;
    x.value = point.x;
    y.value = point.y;
    sourceType.value = source;
  };
  const onMouse = (event: Event): void => {
    if (isMouseEvent(event)) apply(extract(event), "mouse");
  };
  const onTouch = (event: Event): void => {
    if (!isTouchEvent(event)) return;
    const first = event.touches[0];
    if (first) apply(extract(first), "touch");
  };
  const onTouchEnd = (): void => {
    x.value = initial.x;
    y.value = initial.y;
  };

  const stop = watch(
    () => (options.target === undefined ? browserWindow() : toValue(options.target)),
    (target, _previous, onCleanup) => {
      if (!target) return;
      const listenerOptions: AddEventListenerOptions = { passive: true };
      target.addEventListener("mousemove", onMouse, listenerOptions);
      target.addEventListener("dragover", onMouse, listenerOptions);
      if (touch) {
        target.addEventListener("touchstart", onTouch, listenerOptions);
        target.addEventListener("touchmove", onTouch, listenerOptions);
        if (resetOnTouchEnds) target.addEventListener("touchend", onTouchEnd, listenerOptions);
      }
      onCleanup(() => {
        target.removeEventListener("mousemove", onMouse);
        target.removeEventListener("dragover", onMouse);
        target.removeEventListener("touchstart", onTouch);
        target.removeEventListener("touchmove", onTouch);
        target.removeEventListener("touchend", onTouchEnd);
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return { x: readonly(x), y: readonly(y), sourceType: readonly(sourceType) };
}

function extractorFor(
  type: MouseCoordinateType | MouseCoordinateExtractor,
): MouseCoordinateExtractor {
  if (typeof type === "function") return type;
  switch (type) {
    case "page":
      return (input) => ({ x: input.pageX, y: input.pageY });
    case "client":
      return (input) => ({ x: input.clientX, y: input.clientY });
    case "screen":
      return (input) => ({ x: input.screenX, y: input.screenY });
    case "movement":
      return (input) => ("movementX" in input ? { x: input.movementX, y: input.movementY } : null);
  }
}

function isMouseEvent(event: Event): event is MouseEvent {
  return "clientX" in event && "button" in event;
}

function isTouchEvent(event: Event): event is TouchEvent {
  return "touches" in event;
}

function browserWindow(): EventTarget | undefined {
  return typeof window !== "undefined" ? window : undefined;
}

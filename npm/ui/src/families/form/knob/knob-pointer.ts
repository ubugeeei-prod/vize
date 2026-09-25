import { onScopeDispose } from "vue";

import { pointerAngle } from "./knob-geometry.ts";

/** Handlers produced by {@link useRotaryDrag}. */
export interface RotaryDragController {
  /** Start a drag from a primary-button pointer press on the rotary element. */
  readonly onPointerdown: (event: PointerEvent) => void;
}

/** Options for {@link useRotaryDrag}. */
export interface RotaryDragOptions {
  /** Whether dragging is currently allowed. */
  readonly enabled: () => boolean;

  /** Called with each pointer angle (degrees clockwise from 12 o'clock). */
  readonly onAngle: (angle: number) => void;

  /** Called when the drag ends. */
  readonly onEnd: () => void;
}

/**
 * Track a rotary drag around the pressed element's center. Listeners live on
 * the element with pointer capture and are removed on release or scope disposal.
 */
export function useRotaryDrag(options: RotaryDragOptions): RotaryDragController {
  let stop: (() => void) | undefined;

  function onPointerdown(event: PointerEvent): void {
    if (event.button !== 0 || !options.enabled()) return;
    if (!(event.currentTarget instanceof HTMLElement)) return;
    const element = event.currentTarget;
    event.preventDefault();
    element.focus({ preventScroll: true });
    stop?.();
    const rect = element.getBoundingClientRect();
    const center = { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    const pointerId = event.pointerId;
    const report = (move: PointerEvent) =>
      options.onAngle(pointerAngle(center, { x: move.clientX, y: move.clientY }));
    try {
      element.setPointerCapture(pointerId);
    } catch {
      // Synthetic pointers cannot be captured; element listeners still work.
    }
    const onMove = (move: PointerEvent) => {
      if (move.pointerId === pointerId) report(move);
    };
    const onUp = (up: PointerEvent) => {
      if (up.pointerId !== pointerId) return;
      stop?.();
      options.onEnd();
    };
    element.addEventListener("pointermove", onMove);
    element.addEventListener("pointerup", onUp);
    element.addEventListener("pointercancel", onUp);
    stop = () => {
      element.removeEventListener("pointermove", onMove);
      element.removeEventListener("pointerup", onUp);
      element.removeEventListener("pointercancel", onUp);
      stop = undefined;
    };
    report(event);
  }

  onScopeDispose(() => stop?.());
  return { onPointerdown };
}

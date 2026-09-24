/** Callbacks for one captured pointer drag. */
export interface PointerDragCallbacks {
  /** Called with the total pointer movement since pointerdown, in CSS pixels. */
  readonly onMove: (dx: number, dy: number) => void;

  /** Called once when the pointer is released or cancelled. */
  readonly onEnd: (cancelled: boolean) => void;
}

/**
 * Track one pointer from `pointerdown` until release. Movement is reported as a
 * total delta from the start position so repeated moves never accumulate
 * rounding error. Pointer capture keeps events flowing when the pointer leaves
 * the element. Returns a function that stops tracking early.
 */
export function trackPointerDrag(
  element: Element,
  start: PointerEvent,
  callbacks: PointerDragCallbacks,
): () => void {
  const pointerId = start.pointerId;
  const originX = start.clientX;
  const originY = start.clientY;
  try {
    element.setPointerCapture(pointerId);
  } catch {
    // Synthetic or already-released pointers cannot be captured; tracking continues.
  }

  const onMove = (event: Event): void => {
    if (!(event instanceof PointerEvent) || event.pointerId !== pointerId) return;
    event.preventDefault();
    callbacks.onMove(event.clientX - originX, event.clientY - originY);
  };
  const finish =
    (cancelled: boolean) =>
    (event: Event): void => {
      if (!(event instanceof PointerEvent) || event.pointerId !== pointerId) return;
      stop();
      callbacks.onEnd(cancelled);
    };
  const onUp = finish(false);
  const onCancel = finish(true);

  function stop(): void {
    element.removeEventListener("pointermove", onMove);
    element.removeEventListener("pointerup", onUp);
    element.removeEventListener("pointercancel", onCancel);
  }

  element.addEventListener("pointermove", onMove);
  element.addEventListener("pointerup", onUp);
  element.addEventListener("pointercancel", onCancel);
  return stop;
}

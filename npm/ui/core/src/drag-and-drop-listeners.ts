import { capture, surfaceErrors } from "./families/interaction/move/move-internal.ts";

import type { Point } from "./drag-and-drop-internal.ts";

/** Pointer families that install session-scoped document listeners. */
export type DragListenerSource = "mouse" | "pointer" | "touch";

export interface DragListenerContext {
  /** Contact identifier owned by the session, or `null` for mouse sessions. */
  readonly getContactId: () => number | null;

  /** Deliver one owned movement sample in client coordinates. */
  readonly onPoint: (event: Event, point: Point) => void;

  /** Settle the session; `canceled` reports abnormal teardown. */
  readonly onFinish: (event: Event | null, canceled: boolean) => void;
}

function clientPoint(event: Event, contactId: number | null): Point | null {
  if ("clientX" in event && "clientY" in event) {
    const x = Number((event as MouseEvent).clientX);
    const y = Number((event as MouseEvent).clientY);
    if (Number.isFinite(x) && Number.isFinite(y)) return { x, y };
  }
  if ("changedTouches" in event) {
    const touch = Array.from((event as TouchEvent).changedTouches).find(
      ({ identifier }) => contactId === null || identifier === contactId,
    );
    if (touch) return { x: touch.clientX, y: touch.clientY };
  }
  return null;
}

/** Install and exhaustively clean up the document listeners for one session. */
export function installDragListeners(
  document: Document,
  source: DragListenerSource,
  context: DragListenerContext,
): () => void {
  const removals: Array<() => void> = [];
  const listen = (
    owner: Document | Window,
    type: string,
    callback: EventListener,
    capturePhase = true,
  ) => {
    owner.addEventListener(type, callback, capturePhase);
    removals.push(() => owner.removeEventListener(type, callback, capturePhase));
  };
  const move = (event: Event) => {
    const point = clientPoint(event, context.getContactId());
    if (point) context.onPoint(event, point);
  };
  const end = (event: Event, canceled: boolean) => {
    if (source === "pointer" && (event as PointerEvent).pointerId !== context.getContactId()) {
      return;
    }
    if (source === "mouse" && (event as MouseEvent).button !== 0) return;
    if (source === "touch" && !clientPoint(event, context.getContactId())) return;
    context.onFinish(event, canceled);
  };
  const matchesPointer = (event: Event) =>
    source !== "pointer" || (event as PointerEvent).pointerId === context.getContactId();
  try {
    if (source === "pointer") {
      listen(document, "pointermove", (event) => {
        if (matchesPointer(event)) move(event);
      });
      listen(document, "pointerup", (event) => end(event, false));
      listen(document, "pointercancel", (event) => end(event, true));
    } else if (source === "mouse") {
      listen(document, "mousemove", move);
      listen(document, "mouseup", (event) => end(event, false));
    } else {
      listen(document, "touchmove", move);
      listen(document, "touchend", (event) => end(event, false));
      listen(document, "touchcancel", (event) => end(event, true));
    }
    listen(document, "keydown", (event) => {
      if ((event as KeyboardEvent).key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        context.onFinish(event, true);
      }
    });
    listen(document, "dragstart", (event) => context.onFinish(event, true));
    listen(document, "visibilitychange", () => {
      if (document.visibilityState === "hidden") context.onFinish(null, true);
    });
    if (document.defaultView) {
      listen(document.defaultView, "blur", (event) => context.onFinish(event, true), false);
    }
  } catch (error) {
    const errors: unknown[] = [error];
    for (const remove of removals.reverse()) capture(errors, remove);
    surfaceErrors(errors, "Drag listener setup failed");
    throw error;
  }
  let released = false;
  return () => {
    if (released) return;
    released = true;
    const errors: unknown[] = [];
    for (const remove of removals) capture(errors, remove);
    surfaceErrors(errors, "Drag listener cleanup failed");
  };
}

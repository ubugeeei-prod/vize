import {
  classifyKeyboardEvent,
  classifyPointerEvent,
  classifyVirtualClick,
} from "./interaction-modality-events.ts";
import type {
  InteractionModality,
  InteractionModalityChangeReason,
} from "./interaction-modality-types.ts";

export interface DocumentModalityUpdate {
  readonly modality: InteractionModality | null;
  readonly reason: InteractionModalityChangeReason;
  readonly originalEvent: Event | null;
}

interface DocumentModalityHub {
  readonly subscribers: Set<(update: DocumentModalityUpdate) => void>;
  publish: (update: DocumentModalityUpdate) => boolean;
  modality: InteractionModality | null;
  disposeListeners: () => void;
}

const hubs = new WeakMap<Document, DocumentModalityHub>();

/** Install one capture listener set for all consumers in a document. */
function createHub(
  document: Document,
  initialModality: InteractionModality | null,
): DocumentModalityHub {
  const subscribers = new Set<(update: DocumentModalityUpdate) => void>();
  const pendingUpdates: DocumentModalityUpdate[] = [];
  let publishing = false;
  const hub: DocumentModalityHub = {
    subscribers,
    modality: initialModality,
    disposeListeners: () => undefined,
    publish: () => false,
  };

  hub.publish = (update: DocumentModalityUpdate) => {
    if (hub.modality === update.modality && pendingUpdates.length === 0) return false;
    pendingUpdates.push(update);
    if (publishing) return true;

    publishing = true;
    const errors: unknown[] = [];
    try {
      while (pendingUpdates.length > 0) {
        const nextUpdate = pendingUpdates.shift()!;
        if (hub.modality === nextUpdate.modality) continue;
        hub.modality = nextUpdate.modality;
        for (const subscriber of Array.from(subscribers)) {
          if (!subscribers.has(subscriber)) continue;
          try {
            subscriber(nextUpdate);
          } catch (error) {
            errors.push(error);
          }
        }
      }
    } finally {
      publishing = false;
    }

    if (errors.length === 1) throw errors[0];
    if (errors.length > 1) {
      throw new AggregateError(errors, "Interaction modality subscribers failed");
    }
    return true;
  };
  let lastTouchTime = Number.NEGATIVE_INFINITY;
  const onKeyDown = (event: Event) => {
    const update = classifyKeyboardEvent(event as KeyboardEvent);
    if (update) hub.publish(update);
  };
  const onPointerDown = (event: Event) => hub.publish(classifyPointerEvent(event as PointerEvent));
  const onMouseDown = (event: Event) => {
    const elapsedSinceTouch = event.timeStamp - lastTouchTime;
    if (elapsedSinceTouch >= 0 && elapsedSinceTouch < 800) return;
    hub.publish({ modality: "pointer", reason: "pointer", originalEvent: event });
  };
  const onTouchStart = (event: Event) => {
    lastTouchTime = event.timeStamp;
    hub.publish({ modality: "touch", reason: "touch", originalEvent: event });
  };
  const onClick = (event: Event) => {
    const update = classifyVirtualClick(event as MouseEvent, hub.modality);
    if (update) hub.publish(update);
  };

  document.addEventListener("keydown", onKeyDown, true);
  document.addEventListener("click", onClick, true);
  const supportsPointerEvents = Boolean(document.defaultView?.PointerEvent);
  if (supportsPointerEvents) {
    document.addEventListener("pointerdown", onPointerDown, true);
  } else {
    document.addEventListener("mousedown", onMouseDown, true);
    document.addEventListener("touchstart", onTouchStart, true);
  }

  hub.disposeListeners = () => {
    document.removeEventListener("keydown", onKeyDown, true);
    document.removeEventListener("click", onClick, true);
    if (supportsPointerEvents) {
      document.removeEventListener("pointerdown", onPointerDown, true);
    } else {
      document.removeEventListener("mousedown", onMouseDown, true);
      document.removeEventListener("touchstart", onTouchStart, true);
    }
  };
  return hub;
}

/** Subscribe to shared document state and return an idempotent release callback. */
export function subscribeToDocumentModality(
  document: Document,
  initialModality: InteractionModality | null,
  subscriber: (update: DocumentModalityUpdate) => void,
): { readonly current: InteractionModality | null; readonly release: () => void } {
  let hub = hubs.get(document);
  if (!hub) {
    hub = createHub(document, initialModality);
    hubs.set(document, hub);
  }
  hub.subscribers.add(subscriber);
  let released = false;

  return {
    current: hub.modality,
    release: () => {
      if (released) return;
      released = true;
      hub?.subscribers.delete(subscriber);
      if (hub?.subscribers.size === 0) {
        hub.disposeListeners();
        hubs.delete(document);
      }
    },
  };
}

/** Synchronize an explicit consumer update with every peer in a document. */
export function publishDocumentModality(
  document: Document,
  update: DocumentModalityUpdate,
): boolean {
  const hub = hubs.get(document);
  return hub?.publish(update) ?? false;
}

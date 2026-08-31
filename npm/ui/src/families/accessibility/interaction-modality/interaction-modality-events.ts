import type {
  InteractionModality,
  InteractionModalityChangeReason,
} from "./interaction-modality-types.ts";

/** Internal event classification delivered by a document hub. */
export interface InteractionModalityEvent {
  readonly modality: InteractionModality;
  readonly reason: InteractionModalityChangeReason;
  readonly originalEvent: Event;
}

const modifierKeys = new Set(["Alt", "AltGraph", "Control", "Meta"]);

/** Classify keyboard intent while excluding composition and modified shortcuts. */
export function classifyKeyboardEvent(event: KeyboardEvent): InteractionModalityEvent | null {
  if (
    event.isComposing ||
    event.altKey ||
    event.ctrlKey ||
    event.metaKey ||
    modifierKeys.has(event.key)
  ) {
    return null;
  }

  return { modality: "keyboard", reason: "keyboard", originalEvent: event };
}

/** Map pointer hardware into the stable public modality vocabulary. */
export function classifyPointerEvent(event: PointerEvent): InteractionModalityEvent {
  if (event.pointerId === -1 && event.pointerType === "") {
    return { modality: "virtual", reason: "virtual", originalEvent: event };
  }

  const modality = event.pointerType === "touch" ? "touch" : "pointer";
  return { modality, reason: modality, originalEvent: event };
}

/** Classify coordinate-free clicks used by keyboards and assistive technology. */
export function classifyVirtualClick(
  event: MouseEvent,
  current: InteractionModality | null,
): InteractionModalityEvent | null {
  if (event.detail !== 0 || current === "keyboard") return null;
  return { modality: "virtual", reason: "virtual", originalEvent: event };
}

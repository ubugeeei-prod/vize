import type { PrimitiveElement } from "./primitive.ts";

/** Consumer marker tokens mirrored by {@link List} through `data-marker`. */
export type ListMarker = "disc" | "decimal" | "none";

/** Consumer spacing tokens mirrored by {@link List} through `data-spacing`. */
export type ListSpacing = "compact" | "normal" | "loose";

/** Consumer tone tokens mirrored by {@link List} through `data-tone`. */
export type ListTone = "accent" | "danger" | "muted" | "neutral" | "success" | "warning";

/** Rendered value exposed by {@link List}. */
export type ListElement = PrimitiveElement;

/** State exposed to the default List slot. */
export interface ListSlotState {
  /** Consumer marker token mirrored to `data-marker`. */
  readonly marker: ListMarker;

  /** Consumer spacing token mirrored to `data-spacing`. */
  readonly spacing: ListSpacing;

  /** Consumer tone token mirrored to `data-tone`. */
  readonly tone: ListTone;
}

/** Public component instance state exposed by the List primitive. */
export interface ListExpose extends ListSlotState {
  /** Rendered host element or component instance. */
  readonly element: ListElement | null;
}

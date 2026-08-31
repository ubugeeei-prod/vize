import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Consumer sizing tokens mirrored by {@link Text} through `data-size`. */
export type TextSize = "xs" | "sm" | "md" | "lg" | "xl";

/** Consumer font-weight tokens mirrored by {@link Text} through `data-weight`. */
export type TextWeight = "regular" | "medium" | "semibold" | "bold";

/** Consumer tone tokens mirrored by {@link Text} through `data-tone`. */
export type TextTone = "accent" | "danger" | "muted" | "neutral" | "success" | "warning";

/** Rendered value exposed by {@link Text}. */
export type TextElement = PrimitiveElement;

/** State exposed to the default Text slot. */
export interface TextSlotState {
  /** Consumer size token mirrored to `data-size`. */
  readonly size: TextSize;

  /** Consumer font-weight token mirrored to `data-weight`. */
  readonly weight: TextWeight;

  /** Consumer tone token mirrored to `data-tone`. */
  readonly tone: TextTone;

  /** Whether the consumer requested a truncation styling hook. */
  readonly truncate: boolean;
}

/** Public component instance state exposed by the Text primitive. */
export interface TextExpose extends TextSlotState {
  /** Rendered host element or component instance. */
  readonly element: TextElement | null;
}

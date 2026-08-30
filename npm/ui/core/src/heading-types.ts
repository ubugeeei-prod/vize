import type { PrimitiveElement } from "./primitive.ts";

/** Semantic native heading level used by {@link Heading}. */
export type HeadingLevel = 1 | 2 | 3 | 4 | 5 | 6;

/** Consumer visual-size tokens mirrored by {@link Heading} through `data-size`. */
export type HeadingSize = "xs" | "sm" | "md" | "lg" | "xl" | "2xl";

/** Consumer font-weight tokens mirrored by {@link Heading} through `data-weight`. */
export type HeadingWeight = "regular" | "medium" | "semibold" | "bold";

/** Consumer tone tokens mirrored by {@link Heading} through `data-tone`. */
export type HeadingTone = "accent" | "danger" | "muted" | "neutral" | "success" | "warning";

/** Rendered value exposed by {@link Heading}. */
export type HeadingElement = PrimitiveElement;

/** State exposed to the default Heading slot. */
export interface HeadingSlotState {
  /** Semantic heading level mirrored to `data-level`. */
  readonly level: HeadingLevel;

  /** Consumer visual-size token mirrored to `data-size`. */
  readonly size: HeadingSize;

  /** Consumer font-weight token mirrored to `data-weight`. */
  readonly weight: HeadingWeight;

  /** Consumer tone token mirrored to `data-tone`. */
  readonly tone: HeadingTone;

  /** Whether the consumer requested a truncation styling hook. */
  readonly truncate: boolean;
}

/** Public component instance state exposed by the Heading primitive. */
export interface HeadingExpose extends HeadingSlotState {
  /** Rendered host element or component instance. */
  readonly element: HeadingElement | null;
}

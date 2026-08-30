import type { PrimitiveElement } from "../../../primitive.ts";

/** Consumer sizing tokens mirrored by {@link Code} through `data-size`. */
export type CodeSize = "sm" | "md" | "lg";

/** Code presentation tokens mirrored by {@link Code} through `data-variant`. */
export type CodeVariant = "inline" | "block" | "snippet";

/** Consumer tone tokens mirrored by {@link Code} through `data-tone`. */
export type CodeTone = "accent" | "danger" | "muted" | "neutral" | "success" | "warning";

/** Rendered value exposed by {@link Code}. */
export type CodeElement = PrimitiveElement;

/** State exposed to the default Code slot. */
export interface CodeSlotState {
  /** Consumer size token mirrored to `data-size`. */
  readonly size: CodeSize;

  /** Presentation variant mirrored to `data-variant`. */
  readonly variant: CodeVariant;

  /** Consumer tone token mirrored to `data-tone`. */
  readonly tone: CodeTone;
}

/** Public component instance state exposed by the Code primitive. */
export interface CodeExpose extends CodeSlotState {
  /** Rendered host element or component instance. */
  readonly element: CodeElement | null;
}

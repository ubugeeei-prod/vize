import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Consumer sizing tokens mirrored by {@link Blockquote} through `data-size`. */
export type BlockquoteSize = "sm" | "md" | "lg";

/** Consumer tone tokens mirrored by {@link Blockquote} through `data-tone`. */
export type BlockquoteTone = "accent" | "danger" | "muted" | "neutral" | "success" | "warning";

/** Rendered value exposed by {@link Blockquote}. */
export type BlockquoteElement = PrimitiveElement;

/** State exposed to the default Blockquote slot. */
export interface BlockquoteSlotState {
  /** Consumer size token mirrored to `data-size`. */
  readonly size: BlockquoteSize;

  /** Consumer tone token mirrored to `data-tone`. */
  readonly tone: BlockquoteTone;

  /** Native citation URL mirrored to the root `cite` attribute. */
  readonly cite: string | undefined;
}

/** Public component instance state exposed by the Blockquote primitive. */
export interface BlockquoteExpose extends BlockquoteSlotState {
  /** Rendered host element or component instance. */
  readonly element: BlockquoteElement | null;
}

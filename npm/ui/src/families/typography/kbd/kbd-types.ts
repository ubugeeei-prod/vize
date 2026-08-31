import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Visual size tokens mirrored by {@link Kbd} through `data-size`. */
export type KbdSize = "sm" | "md" | "lg";

/** Keyboard token presentation mirrored by {@link Kbd} through `data-variant`. */
export type KbdVariant = "key" | "shortcut" | "sequence";

/** Consumer tone tokens mirrored by {@link Kbd} through `data-tone`. */
export type KbdTone = "accent" | "danger" | "muted" | "neutral" | "success" | "warning";

/** Rendered value exposed by {@link Kbd}. */
export type KbdElement = PrimitiveElement;

/** State exposed to the default Kbd slot. */
export interface KbdSlotState {
  /** Visual size token mirrored to `data-size`. */
  readonly size: KbdSize;

  /** Presentation variant mirrored to `data-variant`. */
  readonly variant: KbdVariant;

  /** Consumer tone token mirrored to `data-tone`. */
  readonly tone: KbdTone;
}

/** Public component instance state exposed by the Kbd primitive. */
export interface KbdExpose extends KbdSlotState {
  /** Rendered host element or component instance. */
  readonly element: KbdElement | null;
}

import type { PrimitiveElement } from "../../../primitive.ts";

/** Styling variants mirrored by {@link Badge} through `data-variant`. */
export type BadgeVariant = "count" | "label" | "status";

/** Consumer styling tones mirrored by {@link Badge} through `data-tone`. */
export type BadgeTone = "accent" | "danger" | "info" | "neutral" | "success" | "warning";

/** Rendered value exposed by {@link Badge}. */
export type BadgeElement = PrimitiveElement;

/** State exposed to the default Badge slot. */
export interface BadgeSlotState {
  /** Badge usage variant mirrored to `data-variant`. */
  readonly variant: BadgeVariant;

  /** Consumer styling tone mirrored to `data-tone`. */
  readonly tone: BadgeTone;
}

/** Public component instance state exposed by the Badge primitive. */
export interface BadgeExpose extends BadgeSlotState {
  /** Rendered host element or component instance. */
  readonly element: BadgeElement | null;
}

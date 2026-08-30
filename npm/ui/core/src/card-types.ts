import type { PrimitiveElement } from "./primitive.ts";

/** Usage variants mirrored by {@link Card} through `data-variant`. */
export type CardVariant = "card" | "panel" | "surface";

/** Spacing density token mirrored by {@link Card} through `data-density`. */
export type CardDensity = "compact" | "comfortable" | "spacious";

/** Consumer styling tone mirrored by {@link Card} through `data-tone`. */
export type CardTone = "neutral" | "accent" | "info" | "success" | "warning" | "danger";

/** Rendered value exposed by {@link Card}. */
export type CardElement = PrimitiveElement;

/** State exposed to the default Card slot. */
export interface CardSlotState {
  /** Card usage variant mirrored to `data-variant`. */
  readonly variant: CardVariant;

  /** Card density mirrored to `data-density`. */
  readonly density: CardDensity;

  /** Consumer styling tone mirrored to `data-tone`. */
  readonly tone: CardTone;
}

/** Public component instance state exposed by the Card primitive. */
export interface CardExpose extends CardSlotState {
  /** Rendered host element or component instance. */
  readonly element: CardElement | null;
}

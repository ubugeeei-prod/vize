import type { PrimitiveAs } from "../../families/foundations/primitive/primitive.ts";

/** Visual treatment exposed as a stable styling hook. */
export type ButtonVariant = "solid" | "outline" | "quiet";

/** Semantic emphasis used by optional visual presets. */
export type ButtonTone = "accent" | "neutral" | "danger";

/** Density-independent control size. */
export type ButtonSize = "sm" | "md" | "lg";

/** Public props shared by the SFC, source registry, and consumer type tests. */
export interface ButtonProps {
  /**
   * Native element, custom element, or component to render.
   *
   * @default "button"
   */
  readonly as?: PrimitiveAs;

  /**
   * Whether the rendered target already implements native button semantics.
   *
   * @default true when `as` is "button"; otherwise false
   */
  readonly native?: boolean;

  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Remove the control from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Announce in-progress work and prevent repeated activation while preserving focus.
   *
   * @default false
   */
  readonly loading?: boolean;

  /**
   * Stable styling hook for filled, bordered, or low-emphasis actions.
   *
   * @default "solid"
   */
  readonly variant?: ButtonVariant;

  /**
   * Semantic styling hook. It never changes button behavior or accessible semantics.
   *
   * @default "accent"
   */
  readonly tone?: ButtonTone;

  /**
   * Stable sizing hook. Coarse pointers retain the accessible minimum target size.
   *
   * @default "md"
   */
  readonly size?: ButtonSize;
}

/** State passed to the default slot. */
export interface ButtonSlotProps {
  readonly disabled: boolean;
  readonly loading: boolean;
  readonly unavailable: boolean;
}

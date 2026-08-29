import type { ComponentPublicInstance } from "vue";

/** Logical axis controlled by {@link Spacer}. */
export type SpacerAxis = "block" | "inline" | "both";

/** CSS display modes supported by {@link Spacer}. */
export type SpacerDisplay =
  | "block"
  | "inline-block"
  | "flex"
  | "inline-flex"
  | "grid"
  | "inline-grid";

/** Native CSS length, percentage, keyword, or custom property used by {@link Spacer}. */
export type SpacerSize = string;

/** Rendered data state for {@link Spacer}. */
export type SpacerState = "sized";

/** Rendered value exposed by {@link Spacer}. */
export type SpacerElement = Element | ComponentPublicInstance;

/** Inline style applied to the rendered host for native logical sizing. */
export interface SpacerStyle {
  /** Native CSS custom property read by the intrinsic inline-size style. */
  readonly "--vize-ui-spacer-inline-size": SpacerSize;

  /** Native CSS custom property read by the intrinsic block-size style. */
  readonly "--vize-ui-spacer-block-size": SpacerSize;

  /** CSS display mode required for the selected spacer axis. */
  readonly display: SpacerDisplay;

  /** Native logical inline size declaration. */
  readonly inlineSize: string;

  /** Native logical block size declaration. */
  readonly blockSize: string;
}

/** Resolved layout state published by {@link Spacer}. */
export interface SpacerResolvedLayout {
  /** Logical axis controlled by the spacer. */
  readonly axis: SpacerAxis;

  /** Resolved logical inline size. */
  readonly inlineSize: SpacerSize;

  /** Resolved logical block size. */
  readonly blockSize: SpacerSize;

  /** CSS display mode applied to the host. */
  readonly display: SpacerDisplay;

  /** Rendered data state. */
  readonly state: SpacerState;

  /** Native logical-size style object applied to the host. */
  readonly style: SpacerStyle;
}

/** Public instance state exposed by the spacer component. */
export interface SpacerExpose {
  /** Rendered host element or component instance. */
  readonly element: SpacerElement | null;

  /** Logical axis controlled by the spacer. */
  readonly axis: SpacerAxis;

  /** Resolved logical inline size. */
  readonly inlineSize: SpacerSize;

  /** Resolved logical block size. */
  readonly blockSize: SpacerSize;

  /** CSS display mode applied to the host. */
  readonly display: SpacerDisplay;
}

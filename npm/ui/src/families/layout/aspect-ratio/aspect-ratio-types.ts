import type { ComponentPublicInstance } from "vue";

/** Rendered value exposed by {@link AspectRatio}. */
export type AspectRatioElement = Element | ComponentPublicInstance;

/** State exposed to the default slot. */
export interface AspectRatioSlotState {
  /** Ratio used for the rendered box after validation. */
  readonly ratio: number;

  /** Whether the provided ratio fell back to the default square ratio. */
  readonly invalid: boolean;
}

/** Public instance state exposed by the aspect ratio component. */
export interface AspectRatioExpose {
  /** Rendered host element or component instance. */
  readonly element: AspectRatioElement | null;

  /** Ratio used for the rendered box after validation. */
  readonly ratio: number;

  /** Whether the provided ratio fell back to the default square ratio. */
  readonly invalid: boolean;
}

/** Inline style applied to the rendered host for intrinsic sizing. */
export interface AspectRatioStyle {
  /** Native CSS custom property read by the intrinsic aspect-ratio style. */
  readonly "--vize-ui-aspect-ratio": string;

  /** Native CSS aspect-ratio declaration. */
  readonly aspectRatio: string;
}

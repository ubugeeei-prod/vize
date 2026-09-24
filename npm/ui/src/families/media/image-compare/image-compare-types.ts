/** Axis along which the divider moves. */
export type ImageCompareOrientation = "horizontal" | "vertical";

/** Reading direction used to map horizontal keys and pointer positions. */
export type ImageCompareDirection = "ltr" | "rtl";

/**
 * How the pointer moves the divider.
 *
 * - `drag`: press anywhere on the root and drag.
 * - `hover`: the divider follows the pointer without pressing.
 */
export type ImageCompareMode = "drag" | "hover";

/** Interaction state mirrored through `data-state`. */
export type ImageCompareState = "disabled" | "dragging" | "idle";

/** Which image a part belongs to. */
export type ImageCompareSide = "after" | "before";

/** Cause of a position change. */
export type ImageCompareChangeSource = "api" | "keyboard" | "pointer";

/** Localizable accessible strings. Omitted entries use English defaults. */
export interface ImageCompareMessages {
  /** Handle name. @default "Comparison position" */
  readonly handleLabel?: string;

  /** Handle `aria-valuetext`. @default `${Math.round(position)}%` */
  readonly valueText?: (position: number) => string;
}

/** State exposed to every ImageCompare slot. */
export interface ImageCompareSlotState {
  /** Divider position from `0` (all after) to `100` (all before), in percent. */
  readonly position: number;

  /** Divider axis. */
  readonly orientation: ImageCompareOrientation;

  /** Interaction state. */
  readonly state: ImageCompareState;
}

/** Public instance exposed by ImageCompareRoot. */
export interface ImageCompareRootExpose extends ImageCompareSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Request a position (clamped and snapped). Reports whether it changed. */
  readonly setPosition: (position: number) => boolean;

  /** Move focus to the handle. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by ImageCompareHandle. */
export interface ImageCompareHandleExpose {
  /** Rendered slider element. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by ImageCompareBefore, ImageCompareAfter, and ImageCompareLabel. */
export interface ImageComparePartExpose {
  /** Rendered part element. */
  readonly element: HTMLElement | null;
}

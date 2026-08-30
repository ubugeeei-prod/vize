import type { ComponentPublicInstance } from "vue";

/** Accessibility role requested by the Spinner primitive. */
export type SpinnerRole = "progressbar" | "status";

/** Accessibility semantics resolved from Spinner props. */
export type SpinnerAriaState = "decorative" | SpinnerRole;

/** Progress value state exposed when Spinner uses `role="progressbar"`. */
export type SpinnerProgressState = "determinate" | "indeterminate" | "none";

/** Visibility and loading state mirrored to `data-state`. */
export type SpinnerState = "complete" | "hidden" | "idle" | "loading";

/** Rendered value exposed by {@link Spinner}. */
export type SpinnerElement = Element | ComponentPublicInstance;

/** State exposed to the default Spinner slot. */
export interface SpinnerSlotState {
  /** Whether the spinner represents pending work. */
  readonly loading: boolean;

  /** Whether the spinner remains visible in layout. */
  readonly visible: boolean;

  /** Stable state token for styling and tests. */
  readonly state: SpinnerState;

  /** Resolved accessibility semantics for the rendered host. */
  readonly ariaState: SpinnerAriaState;

  /** Progress value policy used by the rendered host. */
  readonly progressState: SpinnerProgressState;

  /** Current normalized value, or `null` when indeterminate. */
  readonly value: number | null;

  /** Normalized lower progress bound. */
  readonly min: number;

  /** Normalized upper progress bound. */
  readonly max: number;

  /** Current completion percentage from 0 to 100, or `null` when indeterminate. */
  readonly percent: number | null;

  /** Whether determinate progress has reached the normalized maximum. */
  readonly complete: boolean;
}

/** Public component instance exposed by the Spinner primitive. */
export interface SpinnerExpose extends SpinnerSlotState {
  /** Rendered host element or component instance. */
  readonly element: SpinnerElement | null;
}

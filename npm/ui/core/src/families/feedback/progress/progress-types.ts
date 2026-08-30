/** Visual and accessibility state exposed by the Progress primitive. */
export type ProgressState = "complete" | "indeterminate" | "loading";

/** Public props accepted by the legacy native Progress primitive. */
export interface ProgressProps {
  /**
   * Consumer-owned progressbar id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Current determinate value. `null`, `undefined`, and non-finite numbers render indeterminate.
   *
   * @default null
   */
  readonly value?: number | null;

  /**
   * Positive maximum value. Non-positive and non-finite numbers fall back to 100.
   *
   * @default 100
   */
  readonly max?: number | null;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the progressbar.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the progressbar.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Human-readable value text for assistive technology.
   *
   * @default undefined
   */
  readonly ariaValueText?: string;
}

/** State exposed to the default Progress slot and component instance. */
export interface ProgressSlotState {
  /** Current normalized value, or `null` when the progressbar is indeterminate. */
  readonly value: number | null;

  /** Positive normalized maximum value used by the native progressbar. */
  readonly max: number;

  /** Current completion percentage from 0 to 100, or `null` when indeterminate. */
  readonly percent: number | null;

  /** Whether no determinate value is available. */
  readonly indeterminate: boolean;

  /** Whether the normalized value has reached the normalized maximum. */
  readonly complete: boolean;

  /** Stable state token for styling and tests. */
  readonly state: ProgressState;
}

/** Slots accepted by the legacy native Progress primitive. */
export interface ProgressSlots {
  /** Optional fallback contents. Receives normalized Progress state for composition. */
  readonly default?: (props: ProgressSlotState) => unknown;
}

/** Public component instance exposed by the Progress primitive. */
export interface ProgressExpose extends ProgressSlotState {
  /** Rendered native progress element. */
  readonly element: HTMLProgressElement | null;
}

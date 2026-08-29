/** Visual and accessibility state exposed by the Progress primitive. */
export type ProgressState = "complete" | "indeterminate" | "loading";

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

/** Public component instance exposed by the Progress primitive. */
export interface ProgressExpose extends ProgressSlotState {
  /** Rendered native progress element. */
  readonly element: HTMLProgressElement | null;
}

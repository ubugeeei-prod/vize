import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

/** Reading direction used for inline-start progress fill. */
export type ProgressBarDirection = "ltr" | "rtl";

/** Visual and accessibility state exposed by the ProgressBar primitive. */
export type ProgressBarState = "complete" | "indeterminate" | "loading";

/** CSS custom properties applied to the ProgressBar root. */
export interface ProgressBarStyle {
  /** Normalized lower bound. */
  readonly "--vize-ui-progress-bar-min": string;

  /** Normalized upper bound. */
  readonly "--vize-ui-progress-bar-max": string;

  /** Normalized value, or the lower bound when indeterminate. */
  readonly "--vize-ui-progress-bar-value": string;

  /** Completion percentage with a `%` unit. */
  readonly "--vize-ui-progress-bar-percent": string;

  /** Unitless completion ratio from 0 to 1. */
  readonly "--vize-ui-progress-bar-ratio": string;
}

/** Public props accepted by the ProgressBar primitive. */
export interface ProgressBarProps {
  /** Native element, custom element, or component rendered as the root. */
  readonly as?: PrimitiveAs;

  /** Consumer-owned progressbar id. `null` and `undefined` select a deterministic fallback. */
  readonly id?: string | null;

  /** Current determinate value. `null`, `undefined`, and non-finite numbers render indeterminate. */
  readonly value?: number | null;

  /** Lower bound. Non-finite values fall back to 0. */
  readonly min?: number | null;

  /** Upper bound. Values less than or equal to `min` are repaired to `min + 100`. */
  readonly max?: number | null;

  /** Reading direction for inline-start fill and indeterminate motion. */
  readonly dir?: ProgressBarDirection;

  /** Optional visible label rendered inside the root. */
  readonly label?: string;

  /** Optional visible value text rendered inside the root and reused as `aria-valuetext`. */
  readonly valueLabel?: string;

  /** Accessible name when no visible label or `aria-labelledby` supplies one. */
  readonly ariaLabel?: string;

  /** Space-separated ids that label the progressbar. */
  readonly ariaLabelledby?: string;

  /** Space-separated ids that describe the progressbar. */
  readonly ariaDescribedby?: string;

  /** Human-readable value text for assistive technology. Overrides `valueLabel`. */
  readonly ariaValueText?: string;
}

/** State exposed to ProgressBar slots and the public component instance. */
export interface ProgressBarSlotState {
  /** Current normalized value, or `null` when the progressbar is indeterminate. */
  readonly value: number | null;

  /** Finite lower bound. */
  readonly min: number;

  /** Finite upper bound, always greater than `min`. */
  readonly max: number;

  /** Current completion percentage from 0 to 100, or `null` when indeterminate. */
  readonly percent: number | null;

  /** Unitless completion ratio from 0 to 1, or `null` when indeterminate. */
  readonly ratio: number | null;

  /** Reading direction reflected to `dir` and `data-dir`. */
  readonly dir: ProgressBarDirection;

  /** Whether no determinate value is available. */
  readonly indeterminate: boolean;

  /** Whether the normalized value has reached the normalized maximum. */
  readonly complete: boolean;

  /** Whether raw inputs had to be repaired before reaching ARIA or CSS. */
  readonly invalid: boolean;

  /** Stable state token for styling and tests. */
  readonly state: ProgressBarState;

  /** Deterministic id used by the root progressbar. */
  readonly id: string;

  /** Deterministic id used by the internal label node when rendered. */
  readonly labelId: string;

  /** Deterministic id used by the internal value node when rendered. */
  readonly valueId: string;

  /** Native CSS custom property hooks applied to the root. */
  readonly style: ProgressBarStyle;
}

/** Slots accepted by the ProgressBar primitive. */
export interface ProgressBarSlots {
  /** Render consumer-owned content inside the root. */
  readonly default?: (props: ProgressBarSlotState) => unknown;

  /** Render a visible label in the label part. */
  readonly label?: (props: ProgressBarSlotState) => unknown;

  /** Render visible value text in the value part. */
  readonly value?: (props: ProgressBarSlotState) => unknown;

  /** Render optional content inside the indicator. */
  readonly indicator?: (props: ProgressBarSlotState) => unknown;
}

/** Public component instance state exposed by the ProgressBar primitive. */
export interface ProgressBarExpose extends ProgressBarSlotState {
  /** Rendered root element or component instance. */
  readonly root: PrimitiveElement | null;

  /** Rendered track element. */
  readonly track: HTMLSpanElement | null;

  /** Rendered indicator element. */
  readonly indicator: HTMLSpanElement | null;

  /** Moves DOM focus to the root when the rendered host supports it. */
  readonly focus: (options?: FocusOptions) => void;
}

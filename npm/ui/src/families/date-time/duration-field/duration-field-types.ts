import type { SegmentedFieldState } from "../date-field/field-segment-runtime.ts";
import type { DurationUnit, DurationValue } from "./duration.ts";

/** Width of the unit labels rendered next to each segment. */
export type DurationUnitDisplay = "long" | "short" | "narrow";

/** State of one duration segment. */
export interface DurationSegmentState {
  /** Unit edited by the segment. */
  readonly unit: DurationUnit;

  /** Current amount, or `null` while empty. */
  readonly value: number | null;

  /** Inclusive bounds. */
  readonly min: number;
  readonly max: number;

  /** Visible amount, typed prefix, or placeholder. */
  readonly text: string;

  /** Localized text rendered before and after the amount, such as ` hr`. */
  readonly prefix: string;
  readonly suffix: string;

  /** Whether the segment shows its placeholder. */
  readonly placeholder: boolean;

  /** Localized unit name used as the accessible label. */
  readonly label: string;

  /** Localized amount with unit announced as `aria-valuetext`. */
  readonly valueText: string;
}

/** State exposed to DurationField slots and its instance. */
export interface DurationFieldSlotState {
  /** Committed duration, or `null` while incomplete. */
  readonly value: DurationValue | null;

  /** Segments in largest-to-smallest order. */
  readonly segments: readonly DurationSegmentState[];

  /** Value-state token. */
  readonly state: SegmentedFieldState;

  /** Whether segments are disabled. */
  readonly disabled: boolean;

  /** Whether segments are immutable. */
  readonly readOnly: boolean;

  /** Whether a complete value is required. */
  readonly required: boolean;
}

/** Public instance exposed by DurationField. */
export interface DurationFieldExpose extends DurationFieldSlotState {
  /** Rendered group element. */
  readonly root: HTMLDivElement | null;

  /** Focus a unit's segment, or the first empty segment; returns whether focus moved. */
  readonly focus: (unit?: DurationUnit, options?: FocusOptions) => boolean;

  /** Replace every segment; returns whether the value changed. */
  readonly setValue: (value: DurationValue | null) => boolean;

  /** Clear every segment; returns whether the value changed. */
  readonly clear: () => boolean;
}

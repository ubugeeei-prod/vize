import type { StyleValue } from "vue";

/** Values accepted by the `aria-invalid` attribute. */
export type KnobAriaInvalid = boolean | "grammar" | "spelling";

/** Source of a committed rotary change. */
export type KnobChangeSource = "api" | "keyboard" | "pointer" | "wheel";

/** State published through the rotary `data-state` contract. */
export type KnobState = "disabled" | "dragging" | "idle" | "readonly";

/** CSS custom properties authored on the rotary root. */
export type KnobStyle = StyleValue & {
  readonly "--vize-knob-angle": string;
  readonly "--vize-knob-percent": string;
};

/** State exposed to Knob slots and instances. */
export interface KnobSlotState {
  /** Current value. */
  readonly value: number;

  /** Indicator angle in degrees (clockwise from 12 o'clock). */
  readonly angle: number;

  /** Position within the bounds, 0–100. */
  readonly percent: number;

  /** Normalized lower bound. */
  readonly min: number;

  /** Normalized upper bound. */
  readonly max: number;

  /** Whether a pointer drag is active. */
  readonly dragging: boolean;

  /** Whether interaction is disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: KnobState;
}

/** Public instance API shared by Knob and AnglePicker. */
export interface KnobExpose extends KnobSlotState {
  /** Rendered `role="slider"` element. */
  readonly element: HTMLSpanElement | null;

  /** Focus the rotary element. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a value (snapped/clamped or wrapped); returns whether it changed. */
  readonly setValue: (value: number) => boolean;

  /** Restore the default value; returns whether it changed. */
  readonly reset: () => boolean;
}

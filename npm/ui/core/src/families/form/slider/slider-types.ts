import type { StyleValue } from "vue";

/** Values accepted by the native `aria-invalid` attribute. */
export type SliderAriaInvalid = boolean | "grammar" | "spelling";

/** Logical orientation published by the Slider primitive. */
export type SliderOrientation = "horizontal" | "vertical";

/** Text direction used for native range directionality and styling hooks. */
export type SliderDirection = "ltr" | "rtl";

/** Positive native range step or browser-owned arbitrary precision. */
export type SliderStep = number | "any";

/** State exposed through the Slider Native CSS data contract. */
export type SliderState = "disabled" | "in-range" | "invalid" | "max" | "min" | "readonly";

/** Headless CSS custom properties authored by the Slider root. */
export type SliderStyle = StyleValue & {
  readonly "--vize-slider-value": string;
  readonly "--vize-slider-min": string;
  readonly "--vize-slider-max": string;
  readonly "--vize-slider-step": string;
  readonly "--vize-slider-percent": string;
};

/** Public props accepted by the Slider component. */
export interface SliderProps {
  /**
   * Consumer-owned control id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name submitted by the range input.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Controlled numeric value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: number;

  /**
   * Initial value for uncontrolled use and the value restored by form reset.
   *
   * @default undefined
   */
  readonly defaultValue?: number;

  /**
   * Native lower bound.
   *
   * @default 0
   */
  readonly min?: number;

  /**
   * Native upper bound. Values less than or equal to `min` are repaired to `min + 1`.
   *
   * @default 100
   */
  readonly max?: number;

  /**
   * Native positive step, or `"any"` for browser-owned arbitrary precision.
   *
   * @default 1
   */
  readonly step?: SliderStep;

  /**
   * Disable editing, focus, and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the slider focusable while preventing user value changes.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Mark the slider as required for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Logical orientation exposed to ARIA and data attributes.
   *
   * @default "horizontal"
   */
  readonly orientation?: SliderOrientation;

  /**
   * Text direction applied to the native range input.
   *
   * @default "ltr"
   */
  readonly dir?: SliderDirection;

  /**
   * Accessible name when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the slider.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the slider.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Id of the validation error message used while invalid.
   *
   * @default undefined
   */
  readonly ariaErrormessage?: string;

  /**
   * Human-readable value text for assistive technology.
   *
   * @default undefined
   */
  readonly ariaValueText?: string;

  /**
   * Invalid state announced to assistive technology.
   *
   * @default false
   */
  readonly ariaInvalid?: SliderAriaInvalid;
}

/** Emits published by the Slider component. */
export interface SliderEmits {
  /** Fired when the value requests a new controlled number. */
  "update:modelValue": [value: number];

  /** Fired after a native input event with the next number and native `Event`. */
  input: [value: number, nativeEvent: Event];

  /** Fired after native change/commit with the current number and native `Event`. */
  change: [value: number, nativeEvent: Event];
}

/** Slots exposed by the Slider component. */
export interface SliderSlots {
  /** Renders optional marks or output with the normalized Slider state. */
  default(props: SliderSlotState): unknown;
}

/** State exposed to the default Slider slot and component instance. */
export interface SliderSlotState {
  /** Current normalized numeric value. */
  readonly value: number;

  /** Normalized lower bound. */
  readonly min: number;

  /** Normalized upper bound. */
  readonly max: number;

  /** Normalized positive step, or `"any"` for browser-owned precision. */
  readonly step: SliderStep;

  /** Current position from 0 to 100. */
  readonly percent: number;

  /** Logical orientation. */
  readonly orientation: SliderOrientation;

  /** Text direction used by the native range input. */
  readonly direction: SliderDirection;

  /** Whether native activation, focus, and form submission are disabled. */
  readonly disabled: boolean;

  /** Whether the slider remains focusable while user value changes are locked. */
  readonly readOnly: boolean;

  /** Whether native constraint validation marks the field required. */
  readonly required: boolean;

  /** Whether assistive technology should announce the slider as invalid. */
  readonly invalid: boolean;

  /** Stable state token for styling and tests. */
  readonly state: SliderState;
}

/** Public component instance exposed by the Slider primitive. */
export interface SliderExpose extends SliderSlotState {
  /** Rendered root element that owns the data and CSS custom property contract. */
  readonly root: HTMLSpanElement | null;

  /** Rendered native range input. */
  readonly element: HTMLInputElement | null;

  /** Move focus to the native range input. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a value update and report whether it differs. */
  readonly setValue: (value: number) => boolean;

  /** Increase the value by one or more normalized steps. */
  readonly stepUp: (steps?: number) => boolean;

  /** Decrease the value by one or more normalized steps. */
  readonly stepDown: (steps?: number) => boolean;

  /** Restore the current default value and report whether it changed. */
  readonly reset: () => boolean;
}

import type { StyleValue } from "vue";

import type {
  SliderAriaInvalid,
  SliderDirection,
  SliderOrientation,
} from "../slider/slider-types.ts";

/** Values accepted by the `aria-invalid` attribute. */
export type RangeSliderAriaInvalid = SliderAriaInvalid;

/** Logical orientation of the RangeSlider track. */
export type RangeSliderOrientation = SliderOrientation;

/** Text direction; RTL flips horizontal arrow keys and pointer mapping. */
export type RangeSliderDirection = SliderDirection;

/** Ordered thumb values, one number per thumb. */
export type RangeSliderValue = readonly number[];

/** State published through the RangeSlider `data-state` contract. */
export type RangeSliderState = "disabled" | "dragging" | "idle" | "invalid";

/** Source of a RangeSlider value change. */
export type RangeSliderChangeSource = "api" | "keyboard" | "pointer";

/** Builds the accessible value text announced for one thumb. */
export type RangeSliderValueText = (value: number, index: number) => string;

/** Headless CSS custom properties authored by the RangeSlider root. */
export type RangeSliderStyle = StyleValue & {
  readonly "--vize-range-slider-range-start": string;
  readonly "--vize-range-slider-range-end": string;
};

/** Headless CSS custom properties authored by each RangeSlider thumb. */
export type RangeSliderThumbStyle = StyleValue & {
  readonly "--vize-range-slider-thumb-percent": string;
};

/** Public props accepted by the RangeSlider root. */
export interface RangeSliderProps {
  /**
   * Native form field name. Each thumb submits one hidden value under this name.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of a form owner outside the component tree.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Controlled thumb values. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: RangeSliderValue;

  /**
   * Initial uncontrolled thumb values, also restored by form reset. The number
   * of values sets the number of thumbs.
   *
   * @default [min, max]
   */
  readonly defaultValue?: RangeSliderValue;

  /**
   * Lower bound.
   *
   * @default 0
   */
  readonly min?: number;

  /**
   * Upper bound. Values less than or equal to `min` are repaired to `min + 1`.
   *
   * @default 100
   */
  readonly max?: number;

  /**
   * Positive step between selectable values.
   *
   * @default 1
   */
  readonly step?: number;

  /**
   * Positive step used by Page Up and Page Down.
   *
   * @default step * 10
   */
  readonly largeStep?: number;

  /**
   * Minimum number of steps kept between neighboring thumbs.
   *
   * @default 0
   */
  readonly minStepsBetweenThumbs?: number;

  /**
   * Logical orientation exposed to ARIA, keyboard, and pointer mapping.
   *
   * @default "horizontal"
   */
  readonly orientation?: RangeSliderOrientation;

  /**
   * Text direction. RTL mirrors horizontal pointer mapping and arrow keys.
   *
   * @default "ltr"
   */
  readonly dir?: RangeSliderDirection;

  /**
   * Disable interaction, focus, and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Formats `aria-valuetext` for each thumb.
   *
   * @default undefined
   */
  readonly getValueText?: RangeSliderValueText;

  /**
   * Accessible name of the group when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the group and every thumb.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe every thumb.
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
   * Invalid state announced to assistive technology.
   *
   * @default false
   */
  readonly ariaInvalid?: RangeSliderAriaInvalid;
}

/** Emits published by the RangeSlider root. */
export interface RangeSliderEmits {
  /** Fired whenever a thumb requests new values, including every drag frame. */
  "update:modelValue": [value: RangeSliderValue];

  /** Fired once a change is committed (key press, pointer release, or API call). */
  change: [value: RangeSliderValue, thumbIndex: number, source: RangeSliderChangeSource];
}

/** State exposed to RangeSlider slots and instances. */
export interface RangeSliderSlotState {
  /** Current normalized ascending thumb values. */
  readonly values: RangeSliderValue;

  /** Thumb positions from 0 to 100. */
  readonly percents: readonly number[];

  /** Normalized lower bound. */
  readonly min: number;

  /** Normalized upper bound. */
  readonly max: number;

  /** Normalized positive step. */
  readonly step: number;

  /** Normalized positive large step. */
  readonly largeStep: number;

  /** Normalized minimum gap between neighboring thumbs, in value units. */
  readonly minDistance: number;

  /** Logical orientation. */
  readonly orientation: RangeSliderOrientation;

  /** Text direction. */
  readonly direction: RangeSliderDirection;

  /** Whether interaction is disabled. */
  readonly disabled: boolean;

  /** Whether the value is announced as invalid. */
  readonly invalid: boolean;

  /** Index of the thumb being dragged, or `null`. */
  readonly activeThumb: number | null;

  /** Stable state token. */
  readonly state: RangeSliderState;
}

/** Slots exposed by the RangeSlider root. */
export interface RangeSliderSlots {
  /** Renders the track, range, and one thumb per value. */
  default(props: RangeSliderSlotState): unknown;
}

/** Public instance API of the RangeSlider root. */
export interface RangeSliderExpose extends RangeSliderSlotState {
  /** Rendered root element. */
  readonly root: HTMLSpanElement | null;

  /** Move one thumb (clamped between its neighbors) and report whether values changed. */
  readonly setThumbValue: (index: number, value: number) => boolean;

  /** Replace every thumb value (normalized and sorted) and report whether values changed. */
  readonly setValue: (value: RangeSliderValue) => boolean;

  /** Focus one thumb. */
  readonly focusThumb: (index: number, options?: FocusOptions) => void;

  /** Restore the default values and report whether they changed. */
  readonly reset: () => boolean;
}

/** Props accepted by RangeSliderThumb. */
export interface RangeSliderThumbProps {
  /**
   * Zero-based index of the value this thumb controls.
   *
   * @default required
   */
  readonly index: number;

  /**
   * Accessible name of this thumb, for example "Minimum price".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}

/** Slot state for RangeSlider thumb contents. */
export interface RangeSliderThumbSlotState {
  /** Thumb index. */
  readonly index: number;

  /** Thumb value. */
  readonly value: number;

  /** Thumb position from 0 to 100. */
  readonly percent: number;

  /** Lowest value this thumb may take given its neighbor. */
  readonly min: number;

  /** Highest value this thumb may take given its neighbor. */
  readonly max: number;

  /** Whether this thumb is being dragged. */
  readonly active: boolean;

  /** Whether interaction is disabled. */
  readonly disabled: boolean;
}

import type { StyleValue } from "vue";

/** Selected Rating value. `null` represents no rating. */
export type RatingValue = number | null;

/** Text direction used for keyboard movement and styling hooks. */
export type RatingDirection = "ltr" | "rtl";

/** Values accepted by the native `aria-invalid` attribute. */
export type RatingAriaInvalid = boolean | "grammar" | "spelling";

/** State exposed through the Rating root Native CSS data contract. */
export type RatingState = "disabled" | "empty" | "invalid" | "readonly" | "selected";

/** State exposed by each generated Rating item. */
export type RatingItemState = "checked" | "disabled" | "readonly" | "unchecked";

/** Headless CSS custom properties authored by the Rating root. */
export type RatingStyle = StyleValue & {
  readonly "--vize-rating-value": string;
  readonly "--vize-rating-min": string;
  readonly "--vize-rating-max": string;
  readonly "--vize-rating-count": string;
  readonly "--vize-rating-percent": string;
};

/** Public props accepted by the Rating component. */
export interface RatingProps {
  /**
   * Consumer-owned group id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native radio field name submitted with the selected rating.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Controlled rating value. `undefined` selects uncontrolled behavior; `null` clears selection.
   *
   * @default undefined
   */
  readonly modelValue?: RatingValue;

  /**
   * Initial uncontrolled rating and the value restored by form reset.
   *
   * @default null
   */
  readonly defaultValue?: RatingValue;

  /**
   * Lowest generated rating value.
   *
   * @default 1
   */
  readonly min?: number;

  /**
   * Highest generated rating value. When omitted, `count` derives the upper bound.
   *
   * @default undefined
   */
  readonly max?: number;

  /**
   * Number of generated rating choices when `max` is omitted.
   *
   * @default 5
   */
  readonly count?: number;

  /**
   * Allow user activation of the currently selected item, Escape, Delete, or Backspace to clear.
   *
   * @default false
   */
  readonly clearable?: boolean;

  /**
   * Disable native focus, activation, and form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the selected rating focusable and submittable while preventing user changes.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Mark the native radio group as required for constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Text direction used by horizontal arrow keys and exposed as `data-dir`.
   *
   * @default "ltr"
   */
  readonly dir?: RatingDirection;

  /**
   * Prefix for each generated radio's accessible name.
   *
   * @default "Rating"
   */
  readonly itemLabel?: string;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the rating group.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the rating group.
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
  readonly ariaInvalid?: RatingAriaInvalid;
}

/** Emits published by the Rating component. */
export interface RatingEmits {
  /** Fired when the rating requests a new controlled value. */
  "update:modelValue": [value: RatingValue];

  /** Fired after user activation requests a distinct rating value. */
  change: [value: RatingValue, previous: RatingValue, nativeEvent: Event];

  /** Fired after user activation clears a previously selected rating. */
  clear: [previous: number, nativeEvent: Event];
}

/** State exposed to the Rating default slot and component instance. */
export interface RatingSlotState {
  /** Current normalized rating value, or `null` when empty. */
  readonly value: RatingValue;

  /** Normalized lower bound. */
  readonly min: number;

  /** Normalized upper bound. */
  readonly max: number;

  /** Number of generated rating choices. */
  readonly count: number;

  /** Generated rating values in DOM order. */
  readonly items: readonly number[];

  /** Current selected progress from 0 to 100. */
  readonly percent: number;

  /** Text direction used by horizontal arrow keys. */
  readonly direction: RatingDirection;

  /** Whether user activation, focus, and form submission are disabled. */
  readonly disabled: boolean;

  /** Whether the rating is focusable but user changes are locked. */
  readonly readOnly: boolean;

  /** Whether native constraint validation marks the radio group required. */
  readonly required: boolean;

  /** Whether assistive technology should announce the rating as invalid. */
  readonly invalid: boolean;

  /** Whether user gestures may clear the current value. */
  readonly clearable: boolean;

  /** Stable state token for styling and tests. */
  readonly state: RatingState;
}

/** State exposed to the Rating item slot. */
export interface RatingItemSlotState extends Omit<
  RatingSlotState,
  "items" | "percent" | "state" | "value"
> {
  /** Item value submitted when this radio is selected. */
  readonly value: number;

  /** Zero-based item index in DOM order. */
  readonly index: number;

  /** Current group value, or `null` when no rating is selected. */
  readonly currentValue: RatingValue;

  /** Whether this item is the checked radio. */
  readonly checked: boolean;

  /** Whether this item is visually included in the current rating. */
  readonly active: boolean;

  /** Item progress from 0 to 100. */
  readonly percent: number;

  /** Stable item state token for styling and tests. */
  readonly state: RatingItemState;
}

/** Slots accepted by the Rating component. */
export interface RatingSlots {
  /** Renders optional summary or output with the normalized Rating state. */
  default(props: RatingSlotState): unknown;

  /** Renders each generated rating indicator with item and group state. */
  item(props: RatingItemSlotState): unknown;
}

/** Public component instance exposed by the Rating primitive. */
export interface RatingExpose extends RatingSlotState {
  /** Rendered root element that owns the data and CSS custom property contract. */
  readonly root: HTMLSpanElement | null;

  /** Rendered native radio inputs in DOM order. */
  readonly elements: readonly HTMLInputElement[];

  /** Move focus to the checked item, or to the first enabled item. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a rating update and report whether it differs. */
  readonly setValue: (value: RatingValue) => boolean;

  /** Clear the current rating and report whether it changed. */
  readonly clear: () => boolean;

  /** Restore the current default rating and report whether it changed. */
  readonly reset: () => boolean;
}

/** Selection mode exposed by the NativeSelect primitive. */
export type NativeSelectSelectionMode = "single" | "multiple";

/** Reading direction reflected on the native select element. */
export type NativeSelectDirection = "ltr" | "rtl";

/** Values accepted by the native `aria-invalid` attribute. */
export type NativeSelectAriaInvalid = boolean | "grammar" | "spelling";

/** Selected value for single native selects. */
export type NativeSelectSingleValue = string;

/** Selected values for multiple native selects. */
export type NativeSelectMultipleValue = readonly string[];

/** Selected value held by NativeSelect. */
export type NativeSelectValue = NativeSelectSingleValue | NativeSelectMultipleValue;

/** Option descriptor rendered by the `options` prop. */
export interface NativeSelectOption {
  /** Submitted option value. */
  readonly value: string;

  /** Visible option label. */
  readonly label: string;

  /**
   * Disable this option without disabling the select.
   *
   * @default false
   */
  readonly disabled?: boolean;
}

/** State exposed by the root data contract. */
export type NativeSelectState = "disabled" | "empty" | "selected";

/** State exposed by option data attributes for options rendered from props. */
export type NativeSelectOptionState = "disabled" | "selected" | "unselected";

/** Public props accepted by the NativeSelect primitive. */
export interface NativeSelectProps {
  /**
   * Consumer-owned select id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Controlled selected value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: NativeSelectValue;

  /**
   * Initial value for uncontrolled use and the value restored by form reset.
   *
   * @default undefined
   */
  readonly defaultValue?: NativeSelectValue;

  /**
   * Flat option descriptors rendered before the default slot.
   *
   * @default []
   */
  readonly options?: readonly NativeSelectOption[];

  /**
   * Use the native multiple-selection mode.
   *
   * @default false
   */
  readonly multiple?: boolean;

  /**
   * Native visible row count. Values below one are ignored by the browser.
   *
   * @default undefined
   */
  readonly size?: number;

  /**
   * Disable focus, selection, and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Mark the native select as required for constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Reading direction reflected with `dir` and `data-direction`.
   *
   * @default "ltr"
   */
  readonly direction?: NativeSelectDirection;

  /**
   * Accessible name when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the select.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the select.
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
  readonly ariaInvalid?: NativeSelectAriaInvalid;
}

/** Emits published by the NativeSelect component. */
export interface NativeSelectEmits {
  /** Fired when selection requests a new controlled value. */
  "update:modelValue": [value: NativeSelectValue];

  /** Fired after native change/commit with the next value, previous value, and native Event. */
  change: [value: NativeSelectValue, previous: NativeSelectValue, nativeEvent: Event];
}

/** Slots exposed by the NativeSelect component. */
export interface NativeSelectSlots {
  /** Custom native `<option>` or `<optgroup>` children. */
  default?(props: NativeSelectSlotState): unknown;
}

/** State exposed to the default option slot. */
export interface NativeSelectSlotState {
  /** Current selected value, or an array when `multiple` is true. */
  readonly value: NativeSelectValue;

  /** Selected values as a stable readonly array. */
  readonly selectedValues: readonly string[];

  /** Whether the native select is disabled. */
  readonly disabled: boolean;

  /** Whether the native select participates in required validation. */
  readonly required: boolean;

  /** Whether the native select is currently marked invalid. */
  readonly invalid: boolean;

  /** Current selection mode. */
  readonly selectionMode: NativeSelectSelectionMode;

  /** Whether native multiple selection is enabled. */
  readonly multiple: boolean;

  /** Reading direction reflected on the element. */
  readonly direction: NativeSelectDirection;

  /** Stable state token for styling and tests. */
  readonly state: NativeSelectState;
}

/** Public instance exposed by NativeSelect. */
export interface NativeSelectExpose extends NativeSelectSlotState {
  /** Rendered native select element. */
  readonly element: HTMLSelectElement | null;

  /** Root-owned id for the select. */
  readonly id: string;

  /** Move DOM focus to the native select. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a selected value update and report whether it differs. */
  readonly setValue: (value: NativeSelectValue) => boolean;

  /** Clear the current selection and report whether it changed. */
  readonly clear: () => boolean;

  /** Restore the current default value and report whether it changed. */
  readonly reset: () => boolean;
}

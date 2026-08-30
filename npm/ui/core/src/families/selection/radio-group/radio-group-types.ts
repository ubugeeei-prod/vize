/** Selected value held by a RadioGroup. `null` represents no selection. */
export type RadioGroupValue = string | null;

/** Directional layout hint exposed to Native CSS and slots. */
export type RadioGroupOrientation = "horizontal" | "vertical";

/** Values accepted by the native `aria-invalid` attribute. */
export type RadioGroupAriaInvalid = boolean | "grammar" | "spelling";

/** State exposed by the RadioGroup root data contract. */
export type RadioGroupState = "disabled" | "empty" | "selected";

/** State exposed by each native radio item data contract. */
export type RadioGroupItemState = "checked" | "disabled" | "unchecked";

/** State exposed to the RadioGroup default slot. */
export interface RadioGroupSlotState {
  /** Current selected value, or `null` when no item is selected. */
  readonly value: RadioGroupValue;

  /** Whether every item is disabled by the group. */
  readonly disabled: boolean;

  /** Whether the group participates in native required validation. */
  readonly required: boolean;

  /** Whether the group is currently marked invalid. */
  readonly invalid: boolean;

  /** Directional layout hint for consumer-owned styling and keyboard help. */
  readonly orientation: RadioGroupOrientation;

  /** Stable state token for styling and tests. */
  readonly state: RadioGroupState;
}

/** Public instance exposed by RadioGroup. */
export interface RadioGroupExpose extends RadioGroupSlotState {
  /** Root-owned id for the radio group. */
  readonly id: string;

  /** Move focus to the checked item, or to the first enabled item. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a selected value update and report whether it differs. */
  readonly setValue: (value: RadioGroupValue) => boolean;

  /** Restore the current default value and report whether it changed. */
  readonly reset: () => boolean;
}

/** Public instance exposed by RadioGroupItem. */
export interface RadioGroupItemExpose {
  /** Rendered native radio input. */
  readonly element: HTMLInputElement | null;

  /** Item value submitted when this radio is selected. */
  readonly value: string;

  /** Whether this item is currently selected. */
  readonly checked: boolean;

  /** Whether this item is disabled by itself or the group. */
  readonly disabled: boolean;

  /** Move focus to the native radio input. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Selection model owned by a Listbox root. */
export type ListboxSelectionMode = "single" | "multiple";

/** Directional layout hint exposed to ARIA, slots, and consumer-owned styles. */
export type ListboxOrientation = "horizontal" | "vertical";

/** Reading direction used for horizontal arrow-key navigation. */
export type ListboxDirection = "ltr" | "rtl";

/** Values accepted by the native `aria-invalid` attribute. */
export type ListboxAriaInvalid = boolean | "grammar" | "spelling";

/** Selected value for single-selection listboxes. */
export type ListboxSingleValue = string | null;

/** Selected values for multiple-selection listboxes. */
export type ListboxMultipleValue = readonly string[];

/** Selected value held by a Listbox. */
export type ListboxValue = ListboxSingleValue | ListboxMultipleValue;

/** State exposed by the Listbox root data contract. */
export type ListboxState = "disabled" | "empty" | "selected";

/** State exposed by each option data contract. */
export type ListboxItemState = "disabled" | "selected" | "unselected";

/** Public props accepted by the Listbox primitive. */
export interface ListboxProps {
  /**
   * Consumer-owned listbox id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled selected value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: ListboxValue;

  /**
   * Initial value for uncontrolled use and the value restored by reset.
   *
   * @default undefined
   */
  readonly defaultValue?: ListboxValue;

  /**
   * Disable every option and remove the listbox from sequential focus order.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Mark the listbox as required for accessibility and validation summaries.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Select one option or a set of options.
   *
   * @default "single"
   */
  readonly selectionMode?: ListboxSelectionMode;

  /**
   * Directional layout hint used by arrow-key navigation.
   *
   * @default "vertical"
   */
  readonly orientation?: ListboxOrientation;

  /**
   * Reading direction used by horizontal arrow-key navigation.
   *
   * @default "ltr"
   */
  readonly direction?: ListboxDirection;

  /**
   * Wrap arrow-key navigation at collection boundaries.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Enable locale-aware typeahead over option text.
   *
   * @default true
   */
  readonly typeahead?: boolean;

  /**
   * Idle time before buffered typeahead starts a new query.
   *
   * @default 500
   */
  readonly typeaheadTimeout?: number;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the listbox.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the listbox.
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
  readonly ariaInvalid?: ListboxAriaInvalid;
}

/** State exposed to the Listbox root slots. */
export interface ListboxSlotState {
  /** Current selected value, or an array when `selectionMode` is `"multiple"`. */
  readonly value: ListboxValue;

  /** Selected values as a stable readonly array. */
  readonly selectedValues: readonly string[];

  /** Current active option value, or `null` when no option is navigable. */
  readonly activeValue: string | null;

  /** Whether every item is disabled by the listbox. */
  readonly disabled: boolean;

  /** Whether the listbox participates in required validation semantics. */
  readonly required: boolean;

  /** Whether the listbox is currently marked invalid. */
  readonly invalid: boolean;

  /** Whether single or multiple selection is active. */
  readonly selectionMode: ListboxSelectionMode;

  /** Directional layout hint for ARIA and consumer-owned styles. */
  readonly orientation: ListboxOrientation;

  /** Reading direction used by horizontal keyboard navigation. */
  readonly direction: ListboxDirection;

  /** Stable state token for styling and tests. */
  readonly state: ListboxState;
}

/** State exposed to each ListboxItem slot. */
export interface ListboxItemSlotState {
  /** Item value used for selection and active-descendant ownership. */
  readonly value: string;

  /** Whether this item is the active option. */
  readonly active: boolean;

  /** Whether this item is currently selected. */
  readonly selected: boolean;

  /** Whether this item is disabled by itself or the listbox. */
  readonly disabled: boolean;

  /** Whether single or multiple selection is active. */
  readonly selectionMode: ListboxSelectionMode;

  /** Stable state token for styling and tests. */
  readonly state: ListboxItemState;
}

/** Public instance exposed by Listbox. */
export interface ListboxExpose extends ListboxSlotState {
  /** Rendered listbox element. */
  readonly element: HTMLDivElement | null;

  /** Root-owned id for the listbox. */
  readonly id: string;

  /** Move DOM focus to the listbox focus owner. */
  readonly focus: (options?: FocusOptions) => void;

  /** Move the active option by command and report the resulting option value. */
  readonly navigate: (
    command: "first" | "last" | "next" | "page-next" | "page-previous" | "previous",
    nativeEvent?: Event | null,
  ) => string | null;

  /** Request an active option update and report whether it changed. */
  readonly setActiveValue: (value: string | null) => boolean;

  /** Request a selected value update and report whether it differs. */
  readonly setValue: (value: ListboxValue) => boolean;

  /** Select an option value and report whether the selection changed. */
  readonly selectValue: (value: string, nativeEvent?: Event | null) => boolean;

  /** Toggle an option value in the current selection model. */
  readonly toggleValue: (value: string, nativeEvent?: Event | null) => boolean;

  /** Clear the current selection and report whether it changed. */
  readonly clear: () => boolean;

  /** Restore the current default value and report whether it changed. */
  readonly reset: () => boolean;
}

/** Public instance exposed by ListboxItem. */
export interface ListboxItemExpose extends ListboxItemSlotState {
  /** Rendered option element. */
  readonly element: HTMLDivElement | null;

  /** Move active-descendant focus to this item through the root listbox. */
  readonly focus: (options?: FocusOptions) => void;

  /** Select this item and report whether the selection changed. */
  readonly select: () => boolean;
}

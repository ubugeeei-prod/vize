import type { Placement, PositionerStrategy } from "../../overlays/positioner/positioner.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../../overlays/dismissable-layer/dismissable-layer.ts";
import type { SelectBy, SelectModelValue } from "./select-model.ts";

export type {
  SelectBy,
  SelectModelValue,
  SelectValueComparator,
  SelectValueKey,
} from "./select-model.ts";

/** Open state mirrored to the Select data contract. */
export type SelectState = "closed" | "open";

/** Selection mode derived from the `multiple` prop. */
export type SelectSelectionMode = "multiple" | "single";

/** Reading direction used for typeahead-independent layout hooks. */
export type SelectDirection = "ltr" | "rtl";

/**
 * Popup placement strategy.
 *
 * - `popper` anchors the listbox below (or above) the trigger with collision handling.
 * - `item-aligned` overlays the listbox so the selected option sits on top of the trigger,
 *   like a native macOS select.
 */
export type SelectPosition = "item-aligned" | "popper";

/** State token for each option. */
export type SelectItemState = "checked" | "unchecked";

/** Values accepted by the native `aria-invalid` attribute. */
export type SelectAriaInvalid = boolean | "grammar" | "spelling";

/** Public props accepted by `SelectRoot`. */
export interface SelectRootProps<T, Multiple extends boolean = false> {
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled selection: `T | null` in single mode, `readonly T[]` when `multiple` is `true`.
   * `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: SelectModelValue<T, Multiple>;

  /**
   * Initial selection for uncontrolled use and the value restored by reset.
   *
   * @default undefined
   */
  readonly defaultValue?: SelectModelValue<T, Multiple>;

  /**
   * Allow selecting several options. The literal type of this prop decides the model type.
   *
   * @default false
   */
  readonly multiple?: Multiple;

  /**
   * Every option value, in display order. Optional for compound use; enables typed
   * inference, closed-state labels, and virtualized navigation.
   *
   * @default undefined
   */
  readonly items?: readonly T[];

  /**
   * Compare values by a property key or with a custom equality function.
   *
   * @default undefined
   */
  readonly by?: SelectBy<T>;

  /**
   * Human-readable text for a value, used by `SelectValue`, typeahead, and SSR labels.
   *
   * @default undefined
   */
  readonly itemText?: (value: T) => string;

  /**
   * Disable individual values declared through `items`.
   *
   * @default undefined
   */
  readonly itemDisabled?: (value: T) => boolean;

  /**
   * Serialize a value for native form submission.
   *
   * @default undefined
   */
  readonly formValue?: (value: T) => string;

  /**
   * Controlled open state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial open state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Disable the trigger, options, and form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Require a selection for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Form control name used by the hidden native select.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of the form owner when the select renders outside its `<form>`.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Native autofill hint forwarded to the hidden select.
   *
   * @default undefined
   */
  readonly autocomplete?: string;

  /**
   * Text shown by `SelectValue` while nothing is selected.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Wrap arrow-key navigation at the first and last option.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Idle time before buffered typeahead starts a new query.
   *
   * @default 500
   */
  readonly typeaheadTimeout?: number;

  /**
   * Commit the highlighted option when Tab closes a single-selection popup (APG select-only combobox).
   *
   * @default false
   */
  readonly selectOnTab?: boolean;

  /**
   * Close the popup after an option is chosen. `undefined` closes in single mode only.
   *
   * @default undefined
   */
  readonly closeOnSelect?: boolean;

  /**
   * Reading direction published to parts.
   *
   * @default "ltr"
   */
  readonly dir?: SelectDirection;

  /**
   * Invalid state announced to assistive technology.
   *
   * @default false
   */
  readonly ariaInvalid?: SelectAriaInvalid;
}

/** State exposed to `SelectRoot` slots. */
export interface SelectSlotState<T> {
  /** Selected values in selection order. */
  readonly selected: readonly T[];

  /** Display text for each selected value. */
  readonly selectedText: readonly string[];

  /** Whether the popup is open. */
  readonly open: boolean;

  /** Whether the select is disabled. */
  readonly disabled: boolean;

  /** Whether a selection is required. */
  readonly required: boolean;

  /** Whether the select is invalid (prop or failed native validation). */
  readonly invalid: boolean;

  /** Single or multiple selection. */
  readonly selectionMode: SelectSelectionMode;

  /** Stable state token. */
  readonly state: SelectState;
}

/** State exposed to `SelectValue` slots. */
export interface SelectValueSlotState<T> {
  /** Selected values in selection order. */
  readonly selected: readonly T[];

  /** Display text for each selected value. */
  readonly selectedText: readonly string[];

  /** Placeholder text, when configured. */
  readonly placeholder: string | undefined;

  /** Whether nothing is selected. */
  readonly empty: boolean;
}

/** State exposed to `SelectContent` slots. */
export interface SelectContentSlotState {
  /** Whether the popup is open. */
  readonly open: boolean;

  /** Resolved placement after collision handling. */
  readonly placement: Placement;

  /** Active positioning strategy. */
  readonly position: SelectPosition;
}

/** State exposed to `SelectItem` slots. */
export interface SelectItemSlotState<T> {
  /** Option value. */
  readonly value: T;

  /** Whether this option is selected. */
  readonly selected: boolean;

  /** Whether this option is the highlighted (active-descendant) option. */
  readonly active: boolean;

  /** Whether this option is disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: SelectItemState;
}

/** State exposed to `SelectVirtualizer` item slots. */
export interface SelectVirtualItemSlotState<T> {
  /** Option value at this index. */
  readonly item: T;

  /** Absolute index inside the full item list. */
  readonly index: number;
}

/** Public instance exposed by `SelectRoot`. */
export interface SelectRootExpose<T> extends SelectSlotState<T> {
  /** Root-owned base id. */
  readonly id: string;

  /** Id of the trigger element. */
  readonly triggerId: string;

  /** Id of the listbox element. */
  readonly listboxId: string;

  /** Request a specific open state. */
  readonly setOpen: (open: boolean, event?: Event | null) => boolean;

  /** Select a value (toggle in multiple mode) and report whether it changed. */
  readonly select: (value: T) => boolean;

  /** Deselect a value and report whether it changed. */
  readonly deselect: (value: T) => boolean;

  /** Clear the selection and report whether it changed. */
  readonly clear: () => boolean;

  /** Restore `defaultValue` and report whether it changed. */
  readonly reset: () => boolean;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by `SelectTrigger`. */
export interface SelectTriggerExpose {
  /** Rendered trigger button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by `SelectContent`. */
export interface SelectContentExpose {
  /** Rendered listbox element. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by `SelectItem`. */
export interface SelectItemExpose<T> {
  /** Rendered option element. */
  readonly element: HTMLDivElement | null;

  /** Option value. */
  readonly value: T;

  /** Whether this option is selected. */
  readonly selected: boolean;

  /** Whether this option is highlighted. */
  readonly active: boolean;

  /** Choose this option as if clicked. */
  readonly select: () => boolean;
}

/** Preventable Escape event emitted by `SelectContent`. */
export type SelectEscapeKeyDownEvent = DismissableLayerEscapeKeyDownEvent;

/** Preventable outside pointer event emitted by `SelectContent`. */
export type SelectPointerDownOutsideEvent = DismissableLayerPointerDownOutsideEvent;

/** Dismissal notification emitted by `SelectContent`. */
export type SelectDismissEvent = DismissableLayerDismissEvent;

/** Placement accepted by `SelectContent`. */
export type SelectPlacement = Placement;

/** CSS strategy accepted by `SelectContent`. */
export type SelectPositionerStrategy = PositionerStrategy;

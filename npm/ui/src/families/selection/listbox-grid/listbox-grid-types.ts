import type { ListboxGridBy, ListboxGridModelValue } from "./listbox-grid-model.ts";

export type {
  GridMove,
  GridMoveOptions,
  ListboxGridBy,
  ListboxGridModelValue,
  ListboxGridValueKey,
} from "./listbox-grid-model.ts";

/** State token for the grid root. */
export type ListboxGridState = "disabled" | "empty" | "selected";

/** State token for each option. */
export type ListboxGridItemState = "checked" | "disabled" | "unchecked";

/** Public props accepted by `ListboxGrid`. */
export interface ListboxGridProps<T, Multiple extends boolean = false> {
  /**
   * Consumer-owned listbox id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled selection: `T | null`, or `readonly T[]` when `multiple` is `true`.
   *
   * @default undefined
   */
  readonly modelValue?: ListboxGridModelValue<T, Multiple>;

  /**
   * Initial selection for uncontrolled use and the value restored by reset.
   *
   * @default undefined
   */
  readonly defaultValue?: ListboxGridModelValue<T, Multiple>;

  /**
   * Allow selecting several options. The literal type decides the model type.
   *
   * @default false
   */
  readonly multiple?: Multiple;

  /**
   * Option values in display order; used for inference and select-all.
   *
   * @default undefined
   */
  readonly items?: readonly T[];

  /**
   * Compare values by a property key or with a custom equality function.
   *
   * @default undefined
   */
  readonly by?: ListboxGridBy<T>;

  /**
   * Options per row; must match the consumer's CSS grid so arrow keys follow the layout.
   *
   * @default 4
   */
  readonly columns?: number;

  /**
   * Rows traversed by PageUp and PageDown.
   *
   * @default 3
   */
  readonly pageRows?: number;

  /**
   * Select the option that becomes active through arrow keys (single mode, swatch pickers).
   *
   * @default false
   */
  readonly selectionFollowsFocus?: boolean;

  /**
   * Disable every option and remove the grid from the tab order.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Mark a selection as required for assistive technology.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Form field name; each selected value submits one hidden input.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of the owning form when rendered outside it.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Serialize a value for form submission.
   *
   * @default undefined
   */
  readonly formValue?: (value: T) => string;

  /**
   * Reading direction; `rtl` mirrors ArrowLeft and ArrowRight.
   *
   * @default "ltr"
   */
  readonly dir?: "ltr" | "rtl";

  /**
   * Idle time before buffered typeahead starts a new query.
   *
   * @default 500
   */
  readonly typeaheadTimeout?: number;

  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the grid.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the grid.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}

/** State exposed to the `ListboxGrid` default slot. */
export interface ListboxGridSlotState<T> {
  /** Selected values in selection order. */
  readonly selected: readonly T[];

  /** Id of the highlighted option, or `null`. */
  readonly activeId: string | null;

  /** Options per row. */
  readonly columns: number;

  /** Whether the grid is disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: ListboxGridState;
}

/** State exposed to each `ListboxGridItem` slot. */
export interface ListboxGridItemSlotState<T> {
  /** Option value. */
  readonly value: T;

  /** Zero-based position in the grid. */
  readonly index: number;

  /** Zero-based row of this option. */
  readonly row: number;

  /** Zero-based column of this option. */
  readonly column: number;

  /** Whether this option is highlighted. */
  readonly active: boolean;

  /** Whether this option is selected. */
  readonly selected: boolean;

  /** Whether this option is disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: ListboxGridItemState;
}

/** Public instance exposed by `ListboxGrid`. */
export interface ListboxGridExpose<T> {
  /** Selected values. */
  readonly selected: readonly T[];

  /** Id of the highlighted option. */
  readonly activeId: string | null;

  /** Move DOM focus to the grid. */
  readonly focus: (options?: FocusOptions) => void;

  /** Select (single) or toggle (multiple) a value. */
  readonly select: (value: T) => boolean;

  /** Clear the selection. */
  readonly clear: () => boolean;

  /** Restore `defaultValue`. */
  readonly reset: () => boolean;
}

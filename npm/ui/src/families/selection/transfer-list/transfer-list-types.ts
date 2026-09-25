import type {
  TransferListBy,
  TransferListFilter,
  TransferListOrderMode,
  TransferListSide,
} from "./transfer-list-model.ts";

export type {
  TransferListAction,
  TransferListBy,
  TransferListDirection,
  TransferListFilter,
  TransferListOrderMode,
  TransferListSide,
  TransferListValueKey,
} from "./transfer-list-model.ts";

/** Public props accepted by `TransferListRoot`. */
export interface TransferListRootProps<T> {
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Every item, in canonical order; the source panel shows items not in the target. @default required */
  readonly items: readonly T[];

  /**
   * Controlled target (right-hand) values. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: readonly T[];

  /**
   * Initial target values for uncontrolled use and the value restored by reset.
   *
   * @default []
   */
  readonly defaultValue?: readonly T[];

  /**
   * Compare values by a property key or with a custom equality function.
   *
   * @default undefined
   */
  readonly by?: TransferListBy<T>;

  /**
   * Human-readable text used by search and typeahead.
   *
   * @default undefined
   */
  readonly itemText?: (item: T) => string;

  /**
   * Keep individual items in place.
   *
   * @default undefined
   */
  readonly itemDisabled?: (item: T) => boolean;

  /**
   * Search filter. Defaults to an accent- and case-insensitive "contains" match.
   *
   * @default undefined
   */
  readonly filter?: TransferListFilter<T>;

  /**
   * Controlled source search text.
   *
   * @default undefined
   */
  readonly sourceQuery?: string;

  /**
   * Controlled target search text.
   *
   * @default undefined
   */
  readonly targetQuery?: string;

  /**
   * Maximum number of target values.
   *
   * @default Infinity
   */
  readonly max?: number;

  /**
   * `append` adds moved items at the end; `source-order` keeps `items` order.
   *
   * @default "append"
   */
  readonly orderMode?: TransferListOrderMode;

  /**
   * Disable every panel, item, and action.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Form field name; each target value submits one hidden input.
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
  readonly formValue?: (item: T) => string;
}

/** State exposed to the root slot. */
export interface TransferListSlotState<T> {
  /** Items in the source panel (unfiltered). */
  readonly source: readonly T[];

  /** Items in the target panel (unfiltered). */
  readonly target: readonly T[];

  /** Source items after the source search filter. */
  readonly visibleSource: readonly T[];

  /** Target items after the target search filter. */
  readonly visibleTarget: readonly T[];

  /** Checked source items. */
  readonly checkedSource: readonly T[];

  /** Checked target items. */
  readonly checkedTarget: readonly T[];

  /** Whether the target reached `max`. */
  readonly full: boolean;

  /** Whether the transfer list is disabled. */
  readonly disabled: boolean;
}

/** State exposed to each panel slot. */
export interface TransferListPanelSlotState<T> {
  /** Which side this panel shows. */
  readonly side: TransferListSide;

  /** Visible (filtered) items: render one `TransferListItem` per entry. */
  readonly items: readonly T[];

  /** Checked items in this panel. */
  readonly checked: readonly T[];

  /** Number of items on this side before filtering. */
  readonly total: number;

  /** Current search text. */
  readonly query: string;
}

/** State exposed to each item slot. */
export interface TransferListItemSlotState<T> {
  /** Item value. */
  readonly value: T;

  /** Whether the item is checked in its panel. */
  readonly checked: boolean;

  /** Whether the item is highlighted. */
  readonly active: boolean;

  /** Whether the item cannot move. */
  readonly disabled: boolean;

  /** Which side the item is on. */
  readonly side: TransferListSide;
}

/** Public instance exposed by `TransferListRoot`. */
export interface TransferListRootExpose<T> {
  /** Current target values. */
  readonly target: readonly T[];

  /** Move checked items to the target. */
  readonly moveToTarget: () => readonly T[];

  /** Move checked items back to the source. */
  readonly moveToSource: () => readonly T[];

  /** Restore `defaultValue`. */
  readonly reset: () => boolean;
}

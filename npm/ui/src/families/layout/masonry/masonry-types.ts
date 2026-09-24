/** Stable key for a masonry item. */
export type MasonryKey = string | number;

/** Slot props for each rendered item. */
export interface MasonryItemSlotState<Item> {
  /** Source item. */
  readonly item: Item;
  /** Index in the source list. */
  readonly index: number;
  /** Zero-based column the item is placed in. */
  readonly column: number;
}

/** Imperative API exposed by `Masonry`. */
export interface MasonryExpose {
  /** Root element. */
  readonly element: HTMLElement | null;
  /** Re-measure rendered items and rebalance the columns. */
  readonly measure: () => void;
}

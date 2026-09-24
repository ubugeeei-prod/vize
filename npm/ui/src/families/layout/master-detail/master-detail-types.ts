/** Layout chosen by `MasterDetail`: side by side, or one pane at a time. */
export type MasterDetailLayout = "split" | "stacked";

/** Slot props of the `master` slot. */
export interface MasterDetailMasterSlotState<Key> {
  /** Current selection, or `null`. */
  readonly selected: Key | null;
  /** Current layout. */
  readonly layout: MasterDetailLayout;
  /** Select an item (shows its detail; in stacked layout the detail replaces the list). */
  readonly select: (key: Key) => void;
}

/** Slot props of the `detail` slot. */
export interface MasterDetailDetailSlotState<Key> {
  /** Selected key (never `null` inside the `detail` slot). */
  readonly selected: Key;
  /** Current layout. */
  readonly layout: MasterDetailLayout;
  /** Clear the selection (returns to the list in stacked layout). */
  readonly back: () => void;
}

/** Slot props of the `empty` slot. */
export interface MasterDetailEmptySlotState {
  /** Current layout (the empty slot only renders in split layout). */
  readonly layout: MasterDetailLayout;
}

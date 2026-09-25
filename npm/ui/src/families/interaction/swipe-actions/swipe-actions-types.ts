/** Logical side of a swipe tray. `leading` is the inline-start edge (left in LTR). */
export type SwipeActionsSide = "leading" | "trailing";

/** Which tray is open, or `null` when closed. */
export type SwipeActionsOpen = SwipeActionsSide | null;

/** State published through the SwipeActions `data-state` contract. */
export type SwipeActionsState = "closed" | "dragging" | "open";

/** State exposed to SwipeActions slots and instances. */
export interface SwipeActionsSlotState {
  /** Open tray, or `null`. */
  readonly open: SwipeActionsOpen;

  /** Current horizontal content offset in CSS px (positive moves toward inline-end). */
  readonly offset: number;

  /** Whether a drag is active. */
  readonly dragging: boolean;

  /** Tray that a release would full-swipe commit, or `null`. */
  readonly fullSwipeSide: SwipeActionsSide | null;

  /** Stable state token. */
  readonly state: SwipeActionsState;
}

/** Public instance API of SwipeActions. */
export interface SwipeActionsExpose extends SwipeActionsSlotState {
  /** Open a tray (moving focus into it when `focus` is true). */
  readonly openSide: (side: SwipeActionsSide, focus?: boolean) => boolean;

  /** Close any open tray. */
  readonly close: () => boolean;
}

/** Resolved writing direction shared by direction-aware families. */
export type Direction = "ltr" | "rtl";

/** State exposed to the DirectionProvider slot. */
export interface DirectionSlotState {
  /** Direction published to the subtree. */
  readonly dir: Direction;
}

/** Public instance exposed by DirectionProvider. */
export interface DirectionProviderExpose extends DirectionSlotState {
  /** Rendered wrapper element, or `null` when `as` is `null`. */
  readonly element: HTMLElement | null;
}

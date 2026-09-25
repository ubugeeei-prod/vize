/** Slot props of `StickyStack`. */
export interface StickyStackSlotState {
  /** Total height of the stack in CSS pixels (base offset included). */
  readonly total: number;
}

/** Slot props of `StickyStackItem`. */
export interface StickyStackItemSlotState {
  /** Resolved `top` offset in CSS pixels. */
  readonly top: number;
  /** Whether the item is currently stuck to its offset. */
  readonly stuck: boolean;
}

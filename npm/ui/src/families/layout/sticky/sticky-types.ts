/** Edge the Sticky element pins to while its scroll container scrolls. */
export type StickySide = "bottom" | "top";

/** Stuck state mirrored to the Sticky data contract. */
export type StickyState = "idle" | "stuck";

/** State exposed to the Sticky slot. */
export interface StickySlotState {
  /** Whether the element is currently pinned at its offset. */
  readonly stuck: boolean;

  /** Stable state token for styling and tests. */
  readonly state: StickyState;

  /** Pinned edge. */
  readonly side: StickySide;
}

/** Public instance exposed by Sticky. */
export interface StickyExpose extends StickySlotState {
  /** Rendered sticky element or component instance root. */
  readonly element: HTMLElement | null;

  /** Re-read geometry and update `stuck` immediately. Returns the new value. */
  readonly refresh: () => boolean;
}

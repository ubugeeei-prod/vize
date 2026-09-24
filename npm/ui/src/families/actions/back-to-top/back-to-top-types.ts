/** Scroll animation requested when returning to the top. */
export type BackToTopBehavior = "auto" | "smooth";

/** Visibility state mirrored to the BackToTop data contract. */
export type BackToTopState = "hidden" | "visible";

/** Scroll container accepted by BackToTop: an element, a CSS selector, or `null` for the window. */
export type BackToTopTarget = HTMLElement | string | null;

/** State exposed to the BackToTop slot. */
export interface BackToTopSlotState {
  /** Whether the container is scrolled past `threshold`. */
  readonly visible: boolean;

  /** Stable state token for styling and tests. */
  readonly state: BackToTopState;

  /** Latest observed scroll offset in CSS pixels. */
  readonly scrollTop: number;
}

/** Public instance exposed by BackToTop. */
export interface BackToTopExpose extends BackToTopSlotState {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Scroll the container to the top and move focus like an activation would. */
  readonly scrollToTop: () => boolean;

  /** Re-read the scroll offset immediately. */
  readonly refresh: () => number;
}

/** Scroll container an AppBar follows: an element, a selector, or `null` for the window. */
export type AppBarTarget = HTMLElement | string | null;

/** Stable visibility state exposed for consumer-owned styling and tests. */
export type AppBarState = "visible" | "hidden";

/** State exposed to every AppBar slot. */
export interface AppBarSlotState {
  /** Whether the followed container has scrolled past `elevateThreshold`. */
  readonly elevated: boolean;

  /** Whether `hideOnScroll` currently asks the consumer to move the bar out of view. */
  readonly hidden: boolean;

  /** Stable visibility token for styling and tests. */
  readonly state: AppBarState;

  /** Current scroll offset of the followed container in CSS pixels. */
  readonly scrollTop: number;
}

/** Public instance exposed by AppBar. */
export interface AppBarExpose extends AppBarSlotState {
  /** Rendered root element. */
  readonly element: HTMLElement | null;

  /** Bring a hidden bar back into view. */
  readonly show: () => void;

  /** Re-read the scroll offset, for containers that scroll without a scroll event. */
  readonly refresh: () => number;
}

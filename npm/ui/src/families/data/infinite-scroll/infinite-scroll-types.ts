/**
 * Loading state mirrored by every InfiniteScroll part through `data-state`.
 *
 * - `idle`: more items exist and the sentinel may request them.
 * - `loading`: a request is in flight (`loader` pending or controlled `loading`).
 * - `error`: the last `loader` call rejected; only explicit retries load again.
 * - `complete`: `hasMore` is `false`.
 * - `disabled`: every request is suppressed.
 */
export type InfiniteScrollState = "complete" | "disabled" | "error" | "idle" | "loading";

/** What requested the next page. */
export type InfiniteScrollTrigger = "api" | "button" | "sentinel";

/** Intersection root used by the sentinel. */
export type InfiniteScrollObserverRoot = "self" | "viewport";

/** Consumer page loader. A returned promise keeps the state `loading` until it settles. */
export type InfiniteScrollLoader = (trigger: InfiniteScrollTrigger) => unknown;

/** State exposed to every InfiniteScroll slot. */
export interface InfiniteScrollSlotState {
  /** Current loading state. */
  readonly state: InfiniteScrollState;

  /** Whether a request is in flight. */
  readonly busy: boolean;

  /** Whether more items can be requested. */
  readonly hasMore: boolean;

  /** Rejection reason of the last failed `loader` call, or `undefined`. */
  readonly error: unknown;
}

/** State exposed to InfiniteScrollItem slots. */
export interface InfiniteScrollItemSlotState {
  /** One-based position announced through `aria-posinset`. */
  readonly position: number;

  /** Announced set size; `-1` while the total is unknown. */
  readonly setSize: number;
}

/** Public instance exposed by InfiniteScrollRoot. */
export interface InfiniteScrollRootExpose extends InfiniteScrollSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Request the next page like the sentinel would. Reports whether a request started. */
  readonly loadMore: () => boolean;

  /** Clear an error and request again. Reports whether a request started. */
  readonly retry: () => boolean;

  /** Re-check sentinel visibility, e.g. after a layout change without a state change. */
  readonly refresh: () => void;
}

/** Public instance exposed by InfiniteScrollSentinel. */
export interface InfiniteScrollSentinelExpose {
  /** Rendered sentinel element. */
  readonly element: HTMLDivElement | null;

  /** Whether the sentinel currently intersects its root. */
  readonly intersecting: boolean;
}

/** Public instance exposed by InfiniteScrollLoadMore. */
export interface InfiniteScrollLoadMoreExpose extends InfiniteScrollSlotState {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;
}

/** Public instance exposed by InfiniteScrollStatus. */
export interface InfiniteScrollStatusExpose extends InfiniteScrollSlotState {
  /** Rendered live region. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by InfiniteScrollItem. */
export interface InfiniteScrollItemExpose extends InfiniteScrollItemSlotState {
  /** Rendered article element. */
  readonly element: HTMLElement | null;
}

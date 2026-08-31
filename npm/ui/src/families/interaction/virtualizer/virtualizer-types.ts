import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Scroll axis virtualized by one controller. */
export type VirtualizerOrientation = "horizontal" | "vertical";

/** Target placement for {@link VirtualizerController.scrollToIndex}. */
export type VirtualizerAlignment = "auto" | "center" | "end" | "start";

/** One materialized item inside the rendered window. */
export interface VirtualItem {
  /** Zero-based item index inside the collection. */
  readonly index: number;

  /** Stable render key resolved through `getItemKey`. */
  readonly key: string | number;

  /** Lane this item occupies, `0` for single-lane lists. */
  readonly lane: number;

  /** Main-axis start offset in CSS pixels, including leading padding. */
  readonly start: number;

  /** Main-axis extent in CSS pixels. */
  readonly size: number;

  /** Main-axis end offset in CSS pixels. */
  readonly end: number;

  /** Whether this index is declared sticky through `stickyIndexes`. */
  readonly isSticky: boolean;

  /** Whether `size` comes from a dynamic measurement instead of an estimate. */
  readonly isMeasured: boolean;
}

/** Inclusive visible index range, before overscan is applied. */
export interface VirtualRange {
  /** First index intersecting the viewport. */
  readonly startIndex: number;

  /** Last index intersecting the viewport. */
  readonly endIndex: number;
}

/** Viewport extent used before the platform reports a measured size. */
export interface VirtualizerRect {
  /** Inline-axis extent in CSS pixels. */
  readonly width: number;

  /** Block-axis extent in CSS pixels. */
  readonly height: number;
}

/** Serializable scroll position captured for later restoration. */
export interface VirtualizerScrollSnapshot {
  /** Raw main-axis scroll offset when the snapshot was taken. */
  readonly offset: number;

  /** First visible index, or `null` when nothing was visible. */
  readonly anchorIndex: number | null;

  /** Distance from the anchor item start to the viewport start. */
  readonly anchorGap: number;
}

/** Options shared by {@link createVirtualizer} and {@link useVirtualizer}. */
export interface VirtualizerOptions {
  /** Number of items in the collection. Reactive values relayout on change. */
  readonly count: MaybeRefOrGetter<number>;

  /**
   * Scroll axis to virtualize.
   *
   * @default "vertical"
   */
  readonly orientation?: MaybeRefOrGetter<VirtualizerOrientation | undefined>;

  /**
   * Exact main-axis item size: a fixed pixel size or a per-index resolver.
   * When provided, dynamic measurements are ignored.
   *
   * @default undefined
   */
  readonly itemSize?: number | ((index: number) => number);

  /**
   * Estimated main-axis item size used until an item is measured.
   * Required when `itemSize` is not provided.
   *
   * @default undefined
   */
  readonly estimateItemSize?: number | ((index: number) => number);

  /**
   * Main-axis gap between adjacent items in one lane.
   *
   * @default 0
   */
  readonly gap?: MaybeRefOrGetter<number | undefined>;

  /**
   * Parallel lanes laid out across the cross axis, as in a masonry grid.
   * Item `i` occupies lane `i % lanes`.
   *
   * @default 1
   */
  readonly lanes?: MaybeRefOrGetter<number | undefined>;

  /**
   * Extra items rendered before and after the visible range in every lane.
   *
   * @default 2
   */
  readonly overscan?: MaybeRefOrGetter<number | undefined>;

  /**
   * Leading main-axis padding in CSS pixels.
   *
   * @default 0
   */
  readonly paddingStart?: number;

  /**
   * Trailing main-axis padding in CSS pixels.
   *
   * @default 0
   */
  readonly paddingEnd?: number;

  /**
   * Stable key resolver for rendered items.
   *
   * @default the item index
   */
  readonly getItemKey?: (index: number) => string | number;

  /**
   * Indexes that stay rendered while scrolled past, newest wins.
   *
   * @default []
   */
  readonly stickyIndexes?: MaybeRefOrGetter<readonly number[] | undefined>;

  /**
   * Keep the viewport visually stable when items before it change size.
   *
   * @default true
   */
  readonly anchorScroll?: boolean;

  /**
   * Viewport rect assumed until a real viewport is attached and measured.
   * This is what server rendering and hydration lay out against.
   *
   * @default { width: 0, height: 0 }
   */
  readonly initialRect?: VirtualizerRect;

  /**
   * Main-axis scroll offset assumed before the viewport reports one.
   *
   * @default 0
   */
  readonly initialScrollOffset?: number;

  /** Called after the visible range changes, including to `null`. */
  readonly onRangeChange?: (range: VirtualRange | null) => void;
}

/** Windowing controller for one scroll axis. */
export interface VirtualizerController {
  /** Items to render, visible range plus overscan plus the active sticky item. */
  readonly virtualItems: Readonly<ShallowRef<readonly VirtualItem[]>>;

  /** Visible index range before overscan, or `null` for an empty collection. */
  readonly range: Readonly<ShallowRef<VirtualRange | null>>;

  /** Total scrollable main-axis size, including padding. */
  readonly totalSize: Readonly<ShallowRef<number>>;

  /** Current main-axis scroll offset. */
  readonly scrollOffset: Readonly<ShallowRef<number>>;

  /** Current main-axis viewport extent. */
  readonly viewportSize: Readonly<ShallowRef<number>>;

  /** Sticky index currently pinned at the viewport start, or `null`. */
  readonly activeStickyIndex: Readonly<ShallowRef<number | null>>;

  /** Attach the scrollable viewport element, or detach with `null`. */
  readonly setViewport: (element: Element | null) => void;

  /** Track one rendered item element for dynamic measurement, or stop with `null`. */
  readonly measureElement: (element: Element | null, index: number) => void;

  /** Record an explicit dynamic measurement for one index. */
  readonly resizeItem: (index: number, size: number) => void;

  /** Drop dynamic measurements from `fromIndex` onward and relayout. */
  readonly invalidateMeasurements: (fromIndex?: number) => void;

  /** Shift state after `prepended` items were inserted at index `0`. */
  readonly notifyPrepended: (prepended: number) => void;

  /** Scroll to a clamped main-axis offset. */
  readonly scrollToOffset: (offset: number) => void;

  /** Scroll until the item at `index` satisfies the alignment. */
  readonly scrollToIndex: (index: number, alignment?: VirtualizerAlignment) => void;

  /** Capture the current position for later {@link restoreScroll}. */
  readonly createScrollSnapshot: () => VirtualizerScrollSnapshot;

  /** Restore a captured position, preferring the anchored item. */
  readonly restoreScroll: (snapshot: VirtualizerScrollSnapshot) => void;

  /** Release observers, listeners, and watchers, then freeze the controller. */
  readonly dispose: () => void;
}

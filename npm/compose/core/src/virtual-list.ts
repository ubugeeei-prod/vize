import { computed, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useElementRef } from "./element-ref.ts";
import type { ElementRefSetter } from "./element-ref.ts";
import { useResizeObserver } from "./resize-observer.ts";
import type { ResizeObserverHost } from "./resize-observer.ts";

/** Item size: a fixed size in CSS pixels or a per-item function. */
export type VirtualItemSize<Item> = number | ((index: number, item: Item) => number);

/** Scroll axis of {@link useVirtualList}. */
export type VirtualListOrientation = "vertical" | "horizontal";

/** Options for {@link useVirtualList}. */
export interface UseVirtualListOptions<Item> {
  /** Item size along the scroll axis. */
  readonly itemSize: VirtualItemSize<Item>;

  /**
   * Extra items rendered before and after the visible range.
   *
   * @default 5
   */
  readonly overscan?: number;

  /**
   * Scroll axis.
   *
   * @default "vertical"
   */
  readonly orientation?: VirtualListOrientation;

  /**
   * Items rendered before the container is measured (every server render).
   * Keeps SSR output deterministic and non-empty for crawlers.
   *
   * @default 10
   */
  readonly initialItemCount?: number;

  /**
   * Resize-observer capability used to measure the container.
   *
   * @default globalThis when it provides `ResizeObserver`
   */
  readonly host?: MaybeRefOrGetter<ResizeObserverHost | null | undefined>;
}

/** One rendered row/column of {@link useVirtualList}. */
export interface VirtualListItem<Item> {
  /** Source item. */
  readonly data: Item;
  /** Index in the source list. */
  readonly index: number;
  /** Start offset along the scroll axis. */
  readonly offset: number;
  /** Size along the scroll axis. */
  readonly size: number;
}

/** Inline style for the scroll container. */
export interface VirtualContainerStyle {
  /** Vertical overflow. */
  readonly overflowY: "auto" | "hidden";
  /** Horizontal overflow. */
  readonly overflowX: "auto" | "hidden";
}

/** Inline style for the inner wrapper that holds the rendered items. */
export interface VirtualWrapperStyle {
  /** Wrapper width. */
  readonly width: string;
  /** Wrapper height. */
  readonly height: string;
  /** Leading offset (`margin-top` or `margin-left`). */
  readonly marginTop: string;
  /** Leading offset for horizontal lists. */
  readonly marginLeft: string;
  /** Horizontal lists lay items out in a row. */
  readonly display: "block" | "flex";
}

/** State and bindings returned by {@link useVirtualList}. */
export interface VirtualListControls<Item> {
  /** Items to render. */
  readonly list: ComputedRef<readonly VirtualListItem<Item>[]>;
  /** Total content size along the scroll axis. */
  readonly totalSize: ComputedRef<number>;
  /** Bind on the scroll container: `v-bind="containerProps"`. */
  readonly containerProps: {
    readonly ref: ElementRefSetter;
    readonly onScroll: () => void;
    readonly style: VirtualContainerStyle;
  };
  /** Bind on the inner wrapper: `v-bind="wrapperProps"`. */
  readonly wrapperProps: ComputedRef<{ readonly style: VirtualWrapperStyle }>;
  /** Scroll so that the item at `index` starts at the container edge. */
  readonly scrollTo: (index: number) => void;
}

interface ScrollableElement extends Element {
  scrollTop: number;
  scrollLeft: number;
  readonly clientHeight: number;
  readonly clientWidth: number;
}

/**
 * Render only the visible slice of a long list (fixed or variable item sizes).
 *
 * Offsets are prefix sums recomputed when the items change; the visible range
 * is found by binary search, so scrolling is O(log n). Until the container is
 * measured (always on the server) the first `initialItemCount` items render,
 * so server and first client render match. The container is measured on
 * scroll and via `ResizeObserver`, released with the owning reactive scope.
 *
 * @param items Reactive source list.
 * @param options Item size, overscan, axis, initial count, and capability.
 * @returns Items to render plus container/wrapper bindings.
 */
export function useVirtualList<Item>(
  items: MaybeRefOrGetter<readonly Item[]>,
  options: UseVirtualListOptions<Item>,
): VirtualListControls<Item> {
  const overscan = Math.max(0, Math.floor(options.overscan ?? 5));
  const vertical = (options.orientation ?? "vertical") === "vertical";
  const initialCount = Math.max(0, Math.floor(options.initialItemCount ?? 10));
  const { element: container, setRef } = useElementRef();
  const viewport = shallowRef<number | null>(null);
  const scrollOffset = shallowRef(0);

  const layout = computed(() => {
    const source = toValue(items);
    const offsets = new Float64Array(source.length + 1);
    const size = options.itemSize;
    for (const [index, item] of source.entries()) {
      const next = typeof size === "number" ? size : size(index, item);
      offsets[index + 1] = (offsets[index] ?? 0) + Math.max(0, next);
    }
    return { source, offsets };
  });
  const totalSize = computed(() => layout.value.offsets[layout.value.source.length] ?? 0);

  const measure = (): void => {
    const element = container.value;
    if (!element || !isScrollable(element)) return;
    viewport.value = vertical ? element.clientHeight : element.clientWidth;
    scrollOffset.value = vertical ? element.scrollTop : element.scrollLeft;
  };
  useResizeObserver(
    container,
    measure,
    options.host === undefined ? { flush: "sync" } : { host: options.host, flush: "sync" },
  );

  const range = computed(() => {
    const { source, offsets } = layout.value;
    if (viewport.value === null) return { start: 0, end: Math.min(source.length, initialCount) };
    const first = findIndex(offsets, source.length, scrollOffset.value);
    const last = findIndex(offsets, source.length, scrollOffset.value + viewport.value);
    return {
      start: Math.max(0, first - overscan),
      end: Math.min(source.length, last + 1 + overscan),
    };
  });

  const list = computed<readonly VirtualListItem<Item>[]>(() => {
    const { source, offsets } = layout.value;
    const { start, end } = range.value;
    return source.slice(start, end).map((data, position) => {
      const index = start + position;
      const offset = offsets[index] ?? 0;
      return { data, index, offset, size: (offsets[index + 1] ?? offset) - offset };
    });
  });

  const wrapperProps = computed(() => {
    const leading = layout.value.offsets[range.value.start] ?? 0;
    const remaining = `${totalSize.value - leading}px`;
    return {
      style: {
        width: vertical ? "100%" : remaining,
        height: vertical ? remaining : "100%",
        marginTop: vertical ? `${leading}px` : "0px",
        marginLeft: vertical ? "0px" : `${leading}px`,
        display: vertical ? ("block" as const) : ("flex" as const),
      },
    };
  });

  return {
    list,
    totalSize,
    containerProps: {
      ref: setRef,
      onScroll: measure,
      style: vertical
        ? { overflowY: "auto", overflowX: "hidden" }
        : { overflowY: "hidden", overflowX: "auto" },
    },
    wrapperProps,
    scrollTo: (index) => {
      const element = container.value;
      if (!element || !isScrollable(element)) return;
      const { offsets, source } = layout.value;
      const clamped = Math.min(Math.max(0, Math.floor(index)), source.length);
      const offset = offsets[clamped] ?? 0;
      if (vertical) element.scrollTop = offset;
      else element.scrollLeft = offset;
      measure();
    },
  };
}

/** Largest index whose start offset is ≤ `position`. */
function findIndex(offsets: Float64Array, count: number, position: number): number {
  let low = 0;
  let high = Math.max(0, count - 1);
  while (low < high) {
    const middle = (low + high + 1) >> 1;
    if ((offsets[middle] ?? 0) <= position) low = middle;
    else high = middle - 1;
  }
  return low;
}

function isScrollable(element: Element): element is ScrollableElement {
  return "scrollTop" in element && "clientHeight" in element;
}

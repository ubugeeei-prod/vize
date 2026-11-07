/** Bounded viewport state for large terminal collections. */

import { computed, ref, type ComputedRef, type Ref } from "vue";

export type VirtualScrollAlignment = "nearest" | "start" | "center" | "end";

export interface VirtualViewportRange {
  /** First item occupying the viewport. */
  visibleStart: number;
  /** Exclusive end of items occupying the viewport. */
  visibleEnd: number;
  /** First retained item, including leading overscan. */
  overscanStart: number;
  /** Exclusive end of retained items, including trailing overscan. */
  overscanEnd: number;
}

export interface VirtualViewportOptions {
  itemCount: number;
  viewportSize: number;
  overscan?: number;
  initialOffset?: number;
}

export interface VirtualViewport {
  readonly itemCount: Readonly<Ref<number>>;
  readonly viewportSize: Readonly<Ref<number>>;
  readonly overscan: Readonly<Ref<number>>;
  readonly offset: Readonly<Ref<number>>;
  readonly maxOffset: ComputedRef<number>;
  readonly atStart: ComputedRef<boolean>;
  readonly atEnd: ComputedRef<boolean>;
  readonly range: ComputedRef<VirtualViewportRange>;
  /** Replace the collection size and clamp the current offset. */
  setItemCount(value: number): boolean;
  /** Replace the visible row count and clamp the current offset. */
  setViewportSize(value: number): boolean;
  /** Replace the number of retained rows on each side of the viewport. */
  setOverscan(value: number): boolean;
  /** Move to an absolute, bounded offset. */
  scrollTo(offset: number): boolean;
  /** Move by a signed row delta. */
  scrollBy(delta: number): boolean;
  /** Reveal one item using the requested viewport alignment. */
  scrollIntoView(index: number, alignment?: VirtualScrollAlignment): boolean;
}

function count(value: number, name: string): number {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new RangeError(`${name} must be a non-negative safe integer`);
  }
  return value;
}

/** Create viewport state whose retained range scales with visible rows, not item count. */
export function createVirtualViewport(options: VirtualViewportOptions): VirtualViewport {
  const itemCount = ref(count(options.itemCount, "itemCount"));
  const viewportSize = ref(count(options.viewportSize, "viewportSize"));
  const overscan = ref(count(options.overscan ?? 2, "overscan"));
  const offset = ref(0);
  const maxOffset = computed(() => Math.max(0, itemCount.value - viewportSize.value));
  const atStart = computed(() => offset.value === 0);
  const atEnd = computed(() => offset.value === maxOffset.value);
  const range = computed<VirtualViewportRange>(() => {
    const visibleStart = Math.min(offset.value, itemCount.value);
    const visibleEnd = Math.min(itemCount.value, visibleStart + viewportSize.value);
    return {
      visibleStart,
      visibleEnd,
      overscanStart: Math.max(0, visibleStart - overscan.value),
      overscanEnd: Math.min(itemCount.value, visibleEnd + overscan.value),
    };
  });

  const scrollTo = (value: number) => {
    const next = Math.min(maxOffset.value, count(value, "offset"));
    if (next === offset.value) return false;
    offset.value = next;
    return true;
  };
  const clamp = () => scrollTo(Math.min(offset.value, maxOffset.value));

  const setItemCount = (value: number) => {
    const next = count(value, "itemCount");
    if (next === itemCount.value) return false;
    itemCount.value = next;
    clamp();
    return true;
  };
  const setViewportSize = (value: number) => {
    const next = count(value, "viewportSize");
    if (next === viewportSize.value) return false;
    viewportSize.value = next;
    clamp();
    return true;
  };
  const setOverscan = (value: number) => {
    const next = count(value, "overscan");
    if (next === overscan.value) return false;
    overscan.value = next;
    return true;
  };
  const scrollBy = (delta: number) => {
    if (!Number.isSafeInteger(delta)) throw new RangeError("delta must be a safe integer");
    return scrollTo(Math.max(0, offset.value + delta));
  };
  const scrollIntoView = (index: number, alignment: VirtualScrollAlignment = "nearest") => {
    count(index, "index");
    if (index >= itemCount.value || viewportSize.value === 0) return false;
    let next = offset.value;
    if (alignment === "start") next = index;
    else if (alignment === "center") next = Math.max(0, index - Math.floor(viewportSize.value / 2));
    else if (alignment === "end") next = Math.max(0, index + 1 - viewportSize.value);
    else if (index < offset.value) next = index;
    else if (index >= offset.value + viewportSize.value) {
      next = index + 1 - viewportSize.value;
    }
    return scrollTo(next);
  };

  scrollTo(options.initialOffset ?? 0);

  return {
    itemCount,
    viewportSize,
    overscan,
    offset,
    maxOffset,
    atStart,
    atEnd,
    range,
    setItemCount,
    setViewportSize,
    setOverscan,
    scrollTo,
    scrollBy,
    scrollIntoView,
  };
}

/** Composition API alias for {@link createVirtualViewport}. */
export const useVirtualViewport = createVirtualViewport;

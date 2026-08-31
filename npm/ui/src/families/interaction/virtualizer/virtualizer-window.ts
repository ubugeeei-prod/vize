/**
 * Pure window computation for the virtualizer: visible range, overscan
 * expansion, and sticky-item injection over the lane-aware measure cache.
 */

import type { MeasureCache } from "./virtualizer-measure-cache.ts";
import type { VirtualItem, VirtualRange } from "./virtualizer-types.ts";

/** Inputs for one window computation pass. */
export interface VirtualWindowInput {
  readonly cache: MeasureCache;
  readonly count: number;
  readonly lanes: number;
  readonly overscan: number;
  readonly scrollOffset: number;
  readonly viewportSize: number;
  readonly stickyIndexes: readonly number[];
  readonly getItemKey: (index: number) => string | number;
}

/** One computed rendering window. */
export interface VirtualWindow {
  readonly items: readonly VirtualItem[];
  readonly range: VirtualRange | null;
  readonly activeStickyIndex: number | null;
}

const emptyWindow: VirtualWindow = Object.freeze({
  items: Object.freeze([]),
  range: null,
  activeStickyIndex: null,
});

interface LaneSpan {
  readonly firstVisible: number;
  readonly lastVisible: number;
}

function laneVisibleSpan(
  input: VirtualWindowInput,
  lane: number,
  viewportEnd: number,
): LaneSpan | null {
  const { cache, scrollOffset } = input;
  const length = cache.laneLength(lane);
  if (length === 0) return null;

  const first = cache.firstVisiblePosition(lane, scrollOffset);
  const zeroViewport = viewportEnd === scrollOffset;
  let last = -1;
  for (let position = first; position < length; position++) {
    const placement = cache.placement(cache.laneIndexAt(lane, position));
    const visible =
      placement.end > scrollOffset &&
      (placement.start < viewportEnd || (zeroViewport && placement.start <= scrollOffset));
    if (!visible) break;
    last = position;
  }
  return last < first ? null : { firstVisible: first, lastVisible: last };
}

function activeSticky(input: VirtualWindowInput, stickySet: ReadonlySet<number>): number | null {
  let active: number | null = null;
  for (const index of stickySet) {
    if (input.cache.placement(index).start > input.scrollOffset) continue;
    if (active === null || index > active) active = index;
  }
  return active;
}

function buildItem(input: VirtualWindowInput, index: number, isSticky: boolean): VirtualItem {
  const placement = input.cache.placement(index);
  return Object.freeze({
    index,
    key: input.getItemKey(index),
    lane: placement.lane,
    start: placement.start,
    size: placement.size,
    end: placement.end,
    isSticky,
    isMeasured: placement.isMeasured,
  });
}

/** Compute the rendered window for the current scroll position. */
export function computeVirtualWindow(input: VirtualWindowInput): VirtualWindow {
  const { cache, count, lanes, overscan, scrollOffset, viewportSize } = input;
  const stickySet = new Set(
    input.stickyIndexes.filter((index) => Number.isInteger(index) && index >= 0 && index < count),
  );
  if (count <= 0) return emptyWindow;

  const viewportEnd = scrollOffset + viewportSize;
  const indexes = new Set<number>();
  let rangeStart = Number.POSITIVE_INFINITY;
  let rangeEnd = Number.NEGATIVE_INFINITY;

  for (let lane = 0; lane < Math.min(lanes, count); lane++) {
    const span = laneVisibleSpan(input, lane, viewportEnd);
    if (!span) continue;
    rangeStart = Math.min(rangeStart, cache.laneIndexAt(lane, span.firstVisible));
    rangeEnd = Math.max(rangeEnd, cache.laneIndexAt(lane, span.lastVisible));
    const firstRendered = Math.max(0, span.firstVisible - overscan);
    const lastRendered = Math.min(cache.laneLength(lane) - 1, span.lastVisible + overscan);
    for (let position = firstRendered; position <= lastRendered; position++) {
      indexes.add(cache.laneIndexAt(lane, position));
    }
  }

  const pinned = activeSticky(input, stickySet);
  if (pinned !== null) indexes.add(pinned);

  const items = [...indexes]
    .sort((left, right) => left - right)
    .map((index) => buildItem(input, index, index === pinned || stickySet.has(index)));

  return Object.freeze({
    items: Object.freeze(items),
    range:
      rangeStart <= rangeEnd ? Object.freeze({ startIndex: rangeStart, endIndex: rangeEnd }) : null,
    activeStickyIndex: pinned,
  });
}

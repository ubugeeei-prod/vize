/**
 * Lane-aware measurement cache for the virtualizer.
 *
 * Items are assigned round-robin to lanes; each lane keeps a lazily built
 * prefix-offset chain so estimated sizes, exact sizes, and dynamic
 * measurements resolve to stable main-axis placements. Mutations truncate
 * only the offsets they invalidate.
 */

const invalidOptionDiagnostic = "VIZE_UI_VIRTUALIZER_OPTION";

/** Layout inputs resolved on demand so reactive options stay live. */
export interface MeasureCacheConfig {
  readonly getCount: () => number;
  readonly getLanes: () => number;
  readonly getGap: () => number;
  readonly getPaddingStart: () => number;

  /** Exact or estimated main-axis size for one index. */
  readonly resolveBaseSize: (index: number) => number;

  /** Whether base sizes are exact, which makes dynamic measurements no-ops. */
  readonly usesExactSizes: () => boolean;
}

/** Resolved main-axis placement for one item. */
export interface ItemPlacement {
  readonly start: number;
  readonly size: number;
  readonly end: number;
  readonly lane: number;
  readonly isMeasured: boolean;
}

/** Mutable measurement and offset store consumed by the virtualizer runtime. */
export interface MeasureCache {
  readonly placement: (index: number) => ItemPlacement;
  readonly contentEnd: () => number;
  readonly laneLength: (lane: number) => number;
  readonly laneIndexAt: (lane: number, position: number) => number;
  readonly firstVisiblePosition: (lane: number, offset: number) => number;
  readonly measuredSize: (index: number) => number | undefined;
  readonly setMeasured: (index: number, size: number) => number;
  readonly clearMeasuredFrom: (fromIndex: number) => void;
  readonly invalidateFrom: (index: number) => void;
  readonly shiftMeasurements: (by: number) => void;
}

function assertSize(size: number, source: string): number {
  if (typeof size !== "number" || !Number.isFinite(size) || size < 0) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: ${source} must resolve to a finite non-negative number`,
    );
  }
  return size;
}

/** Create one measurement cache bound to live layout config. */
export function createMeasureCache(config: MeasureCacheConfig): MeasureCache {
  /** `measured[index]` overrides the estimated base size once an item is measured. */
  let measured = new Map<number, number>();

  /** `laneOffsets[lane][position]` is the start offset of that lane slot. */
  let laneOffsets: number[][] = [];

  const laneOf = (index: number): number => index % config.getLanes();
  const positionOf = (index: number): number => Math.trunc(index / config.getLanes());

  const laneIndexAt = (lane: number, position: number): number =>
    position * config.getLanes() + lane;

  const laneLength = (lane: number): number => {
    const count = config.getCount();
    if (lane >= count) return 0;
    return Math.trunc((count - 1 - lane) / config.getLanes()) + 1;
  };

  const sizeOf = (index: number): number => {
    const base = () => assertSize(config.resolveBaseSize(index), "the item size");
    if (config.usesExactSizes()) return base();
    return measured.get(index) ?? base();
  };

  /** Build the whole prefix chain for one lane; results are cached until invalidated. */
  const laneChain = (lane: number): readonly number[] => {
    const length = laneLength(lane);
    const chain = (laneOffsets[lane] ??= []);
    if (chain.length === 0) chain.push(config.getPaddingStart());
    while (chain.length < length) {
      const position = chain.length - 1;
      const previous = chain[position] ?? 0;
      chain.push(previous + sizeOf(laneIndexAt(lane, position)) + config.getGap());
    }
    return chain;
  };

  const assertIndex = (index: number): void => {
    const count = config.getCount();
    if (!Number.isInteger(index) || index < 0 || index >= count) {
      throw new RangeError(
        `${invalidOptionDiagnostic}: index ${index} is outside the collection of ${count}`,
      );
    }
  };

  const placement = (index: number): ItemPlacement => {
    assertIndex(index);
    const lane = laneOf(index);
    const start = laneChain(lane)[positionOf(index)] ?? config.getPaddingStart();
    const size = sizeOf(index);
    return Object.freeze({
      start,
      size,
      end: start + size,
      lane,
      isMeasured: !config.usesExactSizes() && measured.has(index),
    });
  };

  const truncateFrom = (index: number): void => {
    for (let lane = 0; lane < laneOffsets.length; lane++) {
      const chain = laneOffsets[lane];
      if (!chain) continue;
      const keep = index <= lane ? 1 : Math.ceil((index - lane) / config.getLanes()) + 1;
      if (chain.length > keep) chain.length = keep;
    }
  };

  return Object.freeze<MeasureCache>({
    placement,
    laneLength,
    laneIndexAt,
    contentEnd() {
      const count = config.getCount();
      if (count === 0) return config.getPaddingStart();
      let end = config.getPaddingStart();
      const laneCount = Math.min(config.getLanes(), count);
      for (let lane = 0; lane < laneCount; lane++) {
        const last = laneIndexAt(lane, laneLength(lane) - 1);
        end = Math.max(end, placement(last).end);
      }
      return end;
    },
    firstVisiblePosition(lane, offset) {
      const length = laneLength(lane);
      if (length === 0) return 0;
      const chain = laneChain(lane);
      let low = 0;
      let high = length - 1;
      while (low < high) {
        const middle = Math.trunc((low + high) / 2);
        const end = (chain[middle] ?? 0) + sizeOf(laneIndexAt(lane, middle));
        if (end > offset) high = middle;
        else low = middle + 1;
      }
      return low;
    },
    measuredSize: (index) => measured.get(index),
    setMeasured(index, size) {
      assertIndex(index);
      assertSize(size, "the measured size");
      if (config.usesExactSizes()) return 0;
      const previous = sizeOf(index);
      measured.set(index, size);
      const delta = size - previous;
      if (delta !== 0) truncateFrom(index);
      return delta;
    },
    clearMeasuredFrom(fromIndex) {
      for (const index of measured.keys()) {
        if (index >= fromIndex) measured.delete(index);
      }
      truncateFrom(fromIndex);
    },
    invalidateFrom(index) {
      if (index <= 0) laneOffsets = [];
      else truncateFrom(index);
    },
    shiftMeasurements(by) {
      measured = new Map([...measured.entries()].map(([index, size]) => [index + by, size]));
      laneOffsets = [];
    },
  });
}

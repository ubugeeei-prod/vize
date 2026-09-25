/** Position of one item in a masonry layout. */
export interface MasonryPlacement {
  /** Item index in the source list. */
  readonly index: number;
  /** Zero-based column. */
  readonly column: number;
  /** Top offset in CSS pixels within the column. */
  readonly top: number;
  /** Item height in CSS pixels used for placement. */
  readonly height: number;
}

/** Result of {@link computeMasonryLayout}. */
export interface MasonryLayout {
  /** Item indexes per column, in source order. */
  readonly columns: readonly (readonly number[])[];
  /** Placement per item, indexed like the source list. */
  readonly placements: readonly MasonryPlacement[];
  /** Total height of each column. */
  readonly columnHeights: readonly number[];
  /** Height of the tallest column. */
  readonly height: number;
}

const invalidColumns = "VIZE_UI_MASONRY_COLUMNS";

/**
 * Distribute items over columns, always appending to the shortest column
 * (ties go to the leftmost), which keeps columns balanced and preserves
 * reading order row by row.
 *
 * Pure and deterministic: with estimated heights it produces the SSR
 * distribution; with measured heights it produces the client layout.
 *
 * @param heights Item heights in CSS pixels (non-finite or negative values count as `0`).
 * @param columnCount Number of columns (a positive integer).
 * @param gap Vertical gap between items in CSS pixels.
 * @default gap 0
 * @returns Column assignment, placements, and heights.
 */
export function computeMasonryLayout(
  heights: readonly number[],
  columnCount: number,
  gap = 0,
): MasonryLayout {
  if (!Number.isInteger(columnCount) || columnCount < 1) {
    throw new RangeError(
      `${invalidColumns}: columns must be a positive integer, got ${columnCount}`,
    );
  }
  const spacing = Number.isFinite(gap) && gap > 0 ? gap : 0;
  const columns: number[][] = Array.from({ length: columnCount }, () => []);
  const columnHeights: number[] = Array.from({ length: columnCount }, () => 0);
  const placements: MasonryPlacement[] = [];

  heights.forEach((raw, index) => {
    const height = Number.isFinite(raw) && raw > 0 ? raw : 0;
    let column = 0;
    for (let candidate = 1; candidate < columnCount; candidate += 1) {
      if ((columnHeights[candidate] ?? 0) < (columnHeights[column] ?? 0)) column = candidate;
    }
    const current = columnHeights[column] ?? 0;
    const top = columns[column]?.length ? current + spacing : current;
    placements.push({ index, column, top, height });
    columns[column]?.push(index);
    columnHeights[column] = top + height;
  });

  return {
    columns,
    placements,
    columnHeights,
    height: Math.max(0, ...columnHeights),
  };
}

/**
 * Indexes of the items intersecting a vertical window, for virtualization.
 *
 * @param layout Layout from {@link computeMasonryLayout}.
 * @param start Top of the visible window in CSS pixels.
 * @param end Bottom of the visible window in CSS pixels.
 * @param overscan Extra pixels rendered above and below the window.
 * @default overscan 0
 * @returns Visible item indexes in source order.
 */
export function visibleMasonryItems(
  layout: MasonryLayout,
  start: number,
  end: number,
  overscan = 0,
): number[] {
  const top = start - Math.max(0, overscan);
  const bottom = end + Math.max(0, overscan);
  return layout.placements
    .filter((placement) => placement.top + placement.height >= top && placement.top <= bottom)
    .map((placement) => placement.index);
}

/** One widget placed on the dashboard grid, in grid cells. */
export interface DashboardItem {
  /** Stable widget id. */
  readonly id: string;
  /** Zero-based column. */
  readonly x: number;
  /** Zero-based row. */
  readonly y: number;
  /** Width in columns. */
  readonly w: number;
  /** Height in rows. */
  readonly h: number;
  /** Minimum width in columns. */
  readonly minW?: number;
  /** Minimum height in rows. */
  readonly minH?: number;
  /** Maximum width in columns. */
  readonly maxW?: number;
  /** Maximum height in rows. */
  readonly maxH?: number;
  /** Static widgets never move; others flow around them. */
  readonly static?: boolean;
}

/** Every widget on the dashboard. */
export type DashboardLayout = readonly DashboardItem[];

/** Compaction strategy after a change. */
export type DashboardCompaction = "vertical" | "none";

/**
 * Attach an id to a cell placement.
 *
 * @param id Widget id.
 * @param position Placement without an id.
 * @returns A dashboard item.
 */
export function createDashboardItem(
  id: string,
  position: Omit<DashboardItem, "id">,
): DashboardItem {
  return { ...position, id };
}

/**
 * Whether two items overlap.
 *
 * @param left First item.
 * @param right Second item.
 * @returns Whether their cells intersect (an item never collides with itself).
 */
export function dashboardItemsCollide(left: DashboardItem, right: DashboardItem): boolean {
  if (left.id === right.id) return false;
  return (
    left.x < right.x + right.w &&
    left.x + left.w > right.x &&
    left.y < right.y + right.h &&
    left.y + left.h > right.y
  );
}

/**
 * Normalize items: integer cells, sizes within their min/max and the column
 * count, and x inside the grid.
 *
 * @param layout Items to normalize.
 * @param columns Column count.
 * @returns Normalized items in the same order.
 */
export function normalizeDashboardLayout(
  layout: DashboardLayout,
  columns: number,
): DashboardItem[] {
  const cols = Math.max(1, Math.floor(columns));
  return layout.map((item) => {
    const w = clamp(
      Math.round(item.w),
      Math.max(1, item.minW ?? 1),
      Math.min(cols, item.maxW ?? cols),
    );
    const h = clamp(
      Math.round(item.h),
      Math.max(1, item.minH ?? 1),
      item.maxH ?? Number.POSITIVE_INFINITY,
    );
    return {
      ...item,
      w,
      h,
      x: clamp(Math.round(item.x), 0, cols - w),
      y: Math.max(0, Math.round(item.y)),
    };
  });
}

/**
 * Remove gaps: move every non-static item as far up as it can go without
 * overlapping items placed before it (sorted by row, then column).
 *
 * @param layout Items to compact.
 * @param compaction `"vertical"` compacts; `"none"` only resolves overlaps by pushing down.
 * @returns Compacted items in the original order.
 */
export function compactDashboardLayout(
  layout: DashboardLayout,
  compaction: DashboardCompaction = "vertical",
): DashboardItem[] {
  const statics = layout.filter((item) => item.static);
  const placed: DashboardItem[] = [...statics];
  const result = new Map<string, DashboardItem>(statics.map((item) => [item.id, item]));
  const sorted = layout
    .filter((item) => !item.static)
    .sort((left, right) => left.y - right.y || left.x - right.x);
  for (const item of sorted) {
    let candidate = item;
    if (compaction === "vertical") {
      while (
        candidate.y > 0 &&
        !placed.some((other) => dashboardItemsCollide({ ...candidate, y: candidate.y - 1 }, other))
      ) {
        candidate = { ...candidate, y: candidate.y - 1 };
      }
    }
    for (;;) {
      const blocker = placed.find((other) => dashboardItemsCollide(candidate, other));
      if (!blocker) break;
      candidate = { ...candidate, y: blocker.y + blocker.h };
    }
    placed.push(candidate);
    result.set(candidate.id, candidate);
  }
  return layout.map((item) => result.get(item.id) ?? item);
}

/**
 * Move a widget to a cell, pushing colliding widgets down, then compact.
 *
 * Moves onto static widgets are rejected (the original layout is returned).
 *
 * @param layout Current items.
 * @param id Widget to move.
 * @param x Target column.
 * @param y Target row.
 * @param columns Column count.
 * @param compaction Compaction strategy.
 * @default compaction "vertical"
 * @returns The next layout (or the original when nothing changed or the move is invalid).
 */
export function moveDashboardItem(
  layout: DashboardLayout,
  id: string,
  x: number,
  y: number,
  columns: number,
  compaction: DashboardCompaction = "vertical",
): DashboardLayout {
  const current = layout.find((item) => item.id === id);
  if (!current || current.static) return layout;
  const cols = Math.max(1, Math.floor(columns));
  const moved: DashboardItem = {
    ...current,
    x: clamp(Math.round(x), 0, cols - current.w),
    y: Math.max(0, Math.round(y)),
  };
  if (moved.x === current.x && moved.y === current.y) return layout;
  return place(layout, moved, compaction);
}

/**
 * Resize a widget (keeping its top-left cell), pushing colliding widgets
 * down, then compact.
 *
 * @param layout Current items.
 * @param id Widget to resize.
 * @param w Target width in columns.
 * @param h Target height in rows.
 * @param columns Column count.
 * @param compaction Compaction strategy.
 * @default compaction "vertical"
 * @returns The next layout (or the original when nothing changed or the resize is invalid).
 */
export function resizeDashboardItem(
  layout: DashboardLayout,
  id: string,
  w: number,
  h: number,
  columns: number,
  compaction: DashboardCompaction = "vertical",
): DashboardLayout {
  const current = layout.find((item) => item.id === id);
  if (!current || current.static) return layout;
  const cols = Math.max(1, Math.floor(columns));
  const resized: DashboardItem = {
    ...current,
    w: clamp(
      Math.round(w),
      Math.max(1, current.minW ?? 1),
      Math.min(cols - current.x, current.maxW ?? cols),
    ),
    h: clamp(
      Math.round(h),
      Math.max(1, current.minH ?? 1),
      current.maxH ?? Number.POSITIVE_INFINITY,
    ),
  };
  if (resized.w === current.w && resized.h === current.h) return layout;
  return place(layout, resized, compaction);
}

function place(
  layout: DashboardLayout,
  target: DashboardItem,
  compaction: DashboardCompaction,
): DashboardLayout {
  if (layout.some((other) => other.static && dashboardItemsCollide(target, other))) return layout;
  const items = new Map<string, DashboardItem>(layout.map((item) => [item.id, item]));
  items.set(target.id, target);
  const push = (mover: DashboardItem, depth: number): void => {
    if (depth > layout.length) return;
    for (const other of items.values()) {
      if (other.static || other.id === mover.id || other.id === target.id) continue;
      if (!dashboardItemsCollide(mover, other)) continue;
      const pushed = { ...other, y: mover.y + mover.h };
      items.set(other.id, pushed);
      push(pushed, depth + 1);
    }
  };
  push(target, 0);
  const next = layout.map((item) => items.get(item.id) ?? item);
  if (compaction === "none") return next;
  // Keep the dragged widget anchored while others compact around it.
  const anchored = compactDashboardLayout(
    next.map((item) => (item.id === target.id ? { ...item, static: true } : item)),
    compaction,
  );
  return anchored.map((item) => (item.id === target.id ? target : item));
}

/**
 * Height of the layout in rows.
 *
 * @param layout Items.
 * @returns The bottom-most occupied row plus one.
 */
export function dashboardLayoutRows(layout: DashboardLayout): number {
  return layout.reduce((rows, item) => Math.max(rows, item.y + item.h), 0);
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(Math.max(value, minimum), Math.max(minimum, maximum));
}

import type { SplitterLayout, SplitterPanelConstraints } from "./splitter-types.ts";

const precision = 1e-6;

/** Round a size so floating-point drift never leaks into ARIA values or storage. */
export function roundSplitterSize(size: number): number {
  return Math.round(size * 1000) / 1000;
}

/** Whether two layouts are equal within rounding precision. */
export function splitterLayoutsEqual(
  left: SplitterLayout | undefined,
  right: SplitterLayout | undefined,
): boolean {
  if (left === right) return true;
  if (left === undefined || right === undefined || left.length !== right.length) return false;
  return left.every((size, index) => Math.abs(size - (right[index] ?? Number.NaN)) < precision);
}

/** Whether a value is a usable layout for `count` panels: finite, non-negative, summing to 100. */
export function isValidSplitterLayout(value: unknown, count: number): boolean {
  if (!Array.isArray(value) || value.length !== count || count === 0) return false;
  let total = 0;
  for (const size of value) {
    if (typeof size !== "number" || !Number.isFinite(size) || size < 0) return false;
    total += size;
  }
  return Math.abs(total - 100) < 0.01;
}

/**
 * Resolve the layout a group renders before any explicit layout exists.
 *
 * Panels take `defaultLayout[i]`, then their own `defaultSize`; the remaining
 * space is shared equally by panels without either. Only registered panels are
 * considered, so server rendering is exact when every panel declares a size.
 */
export function resolveDefaultSplitterLayout(
  defaults: readonly (number | undefined)[],
): SplitterLayout {
  let known = 0;
  let unknown = 0;
  for (const size of defaults) {
    if (size === undefined) unknown += 1;
    else known += size;
  }
  const share = unknown === 0 ? 0 : Math.max(0, 100 - known) / unknown;
  return defaults.map((size) => roundSplitterSize(size ?? share));
}

/** Clamp every panel into its constraints, then rebalance so the layout sums to 100. */
export function clampSplitterLayout(
  layout: SplitterLayout,
  constraints: readonly SplitterPanelConstraints[],
): SplitterLayout {
  const next = layout.map((size, index) => {
    const constraint = constraints[index];
    if (constraint === undefined) return size;
    if (constraint.collapsible && size <= constraint.collapsedSize + precision) {
      return constraint.collapsedSize;
    }
    return Math.min(constraint.maxSize, Math.max(constraint.minSize, size));
  });
  let difference = 100 - next.reduce((total, size) => total + size, 0);
  for (let index = next.length - 1; index >= 0 && Math.abs(difference) > precision; index--) {
    const constraint = constraints[index];
    const size = next[index];
    if (constraint === undefined || size === undefined) continue;
    const collapsed = constraint.collapsible && size === constraint.collapsedSize;
    if (collapsed) continue;
    const target = Math.min(constraint.maxSize, Math.max(constraint.minSize, size + difference));
    difference -= target - size;
    next[index] = target;
  }
  return next.map(roundSplitterSize);
}

/**
 * Move the boundary after panel `handleIndex` by `delta` percent.
 *
 * A positive delta grows the panel before the handle and takes space from the
 * panels after it, nearest first; a negative delta does the reverse. Panels
 * never leave their `[minSize, maxSize]` range except that a collapsible panel
 * adjacent to the handle snaps to `collapsedSize` once the requested size
 * passes halfway between `collapsedSize` and `minSize`, and a collapsed panel
 * grows straight to `minSize` once the drag passes the same midpoint.
 */
export function resizeSplitterLayout(
  layout: SplitterLayout,
  constraints: readonly SplitterPanelConstraints[],
  handleIndex: number,
  delta: number,
): SplitterLayout {
  const before = handleIndex;
  const after = handleIndex + 1;
  if (before < 0 || after >= layout.length || Math.abs(delta) < precision) return layout;
  const next = [...layout];
  const growIndex = delta > 0 ? before : after;
  const shrinkIndexes: number[] = [];
  if (delta > 0) for (let index = after; index < next.length; index++) shrinkIndexes.push(index);
  else for (let index = before; index >= 0; index--) shrinkIndexes.push(index);

  const grow = constraints[growIndex];
  const growSize = next[growIndex];
  if (grow === undefined || growSize === undefined) return layout;
  let requested = Math.abs(delta);
  if (grow.collapsible && growSize < grow.minSize - precision) {
    const needed = grow.minSize - growSize;
    if (requested < needed / 2) return layout;
    requested = Math.max(requested, needed);
  }
  const capacity = grow.maxSize - growSize;
  requested = Math.min(requested, capacity);
  if (requested <= precision) return layout;

  let taken = 0;
  shrinkIndexes.forEach((index, position) => {
    const remaining = requested - taken;
    const constraint = constraints[index];
    const size = next[index];
    if (remaining <= precision || constraint === undefined || size === undefined) return;
    let target = size - remaining;
    if (target < constraint.minSize) {
      const snapPoint = (constraint.minSize + constraint.collapsedSize) / 2;
      const canCollapse =
        position === 0 &&
        constraint.collapsible &&
        size > constraint.collapsedSize + precision &&
        target <= snapPoint &&
        size - constraint.collapsedSize <= capacity + precision;
      target = canCollapse ? constraint.collapsedSize : Math.min(size, constraint.minSize);
    }
    taken += size - target;
    next[index] = target;
  });

  if (grow.collapsible && growSize < grow.minSize - precision && growSize + taken < grow.minSize) {
    return layout;
  }
  next[growIndex] = growSize + taken;
  return next.map(roundSplitterSize);
}

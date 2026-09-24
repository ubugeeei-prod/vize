/** Offsets resolved by {@link computeStickyOffsets}. */
export interface StickyOffsets {
  /** `top` offset for each item, in stacking order. */
  readonly tops: readonly number[];
  /** Total height of the stack (base offset plus every enabled item). */
  readonly total: number;
}

/**
 * Stack sticky items: each item sticks below the base offset plus the
 * heights of every enabled item before it.
 *
 * Pure and deterministic, so the server can render estimated offsets.
 *
 * @param heights Item heights in stacking order (non-finite or negative count as `0`).
 * @param base Offset of the first item from the top of the scroll port.
 * @param enabled Whether each item participates (disabled items get the running offset but add nothing).
 * @default base 0
 * @default enabled all true
 * @returns Per-item `top` offsets and the total stack height.
 */
export function computeStickyOffsets(
  heights: readonly number[],
  base = 0,
  enabled: readonly boolean[] = [],
): StickyOffsets {
  let running = Number.isFinite(base) ? base : 0;
  const tops = heights.map((raw, index) => {
    const top = running;
    const height = Number.isFinite(raw) && raw > 0 ? raw : 0;
    if (enabled[index] !== false) running += height;
    return top;
  });
  return { tops, total: running };
}

/**
 * Sort items by document order; detached items follow in registration order.
 *
 * @param entries Registered entries with their elements.
 * @returns A new array in stacking order.
 */
export function sortByDocumentOrder<Entry extends { readonly element: Element | null }>(
  entries: readonly Entry[],
): Entry[] {
  return entries
    .map((entry, index) => ({ entry, index }))
    .sort((left, right) => {
      const a = left.entry.element;
      const b = right.entry.element;
      if (a && !b) return -1;
      if (!a && b) return 1;
      if (a && b && a !== b) {
        const position = a.compareDocumentPosition(b);
        if (position & 4) return -1;
        if (position & 2) return 1;
      }
      return left.index - right.index;
    })
    .map(({ entry }) => entry);
}

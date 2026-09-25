import type { StickySide } from "./sticky-types.ts";

/** Viewport-relative edges used by {@link isStickyStuck}. */
export interface StickyEdges {
  readonly top: number;
  readonly bottom: number;
}

/**
 * Decide whether a `position: sticky` box is pinned.
 *
 * A pinned box rests exactly on its offset line (within one pixel) while its
 * opposite edge still lies inside the scroll container; an unpinned box either
 * sits further from the edge or has scrolled away with its containing block.
 */
export function isStickyStuck(
  side: StickySide,
  offset: number,
  box: StickyEdges,
  container: StickyEdges,
): boolean {
  if (side === "top") {
    const distance = box.top - container.top;
    return distance <= offset + 1 && distance >= offset - 1 && box.bottom > container.top + offset;
  }
  const distance = container.bottom - box.bottom;
  return distance <= offset + 1 && distance >= offset - 1 && box.top < container.bottom - offset;
}

/** Root margin that shrinks the observed area by `offset + 1` pixels on the pinned edge. */
export function stickyRootMargin(side: StickySide, offset: number): string {
  const inset = `${-(offset + 1)}px`;
  return side === "top" ? `${inset} 0px 0px 0px` : `0px 0px ${inset} 0px`;
}

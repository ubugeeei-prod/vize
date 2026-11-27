/**
 * Next index in an enabled segmented item list.
 *
 * Horizontal and vertical arrows both move, so the key does not depend on CSS.
 * Movement wraps. Home and End jump to the ends of the enabled list.
 */
export function nextSegmentIndex(current: number, count: number, key: string): number | undefined {
  if (count <= 0) return undefined;
  if (key === "Home") return 0;
  if (key === "End") return count - 1;
  const delta =
    key === "ArrowRight" || key === "ArrowDown"
      ? 1
      : key === "ArrowLeft" || key === "ArrowUp"
        ? -1
        : 0;
  if (delta === 0) return undefined;
  const index = current < 0 ? 0 : current;
  return (index + delta + count) % count;
}

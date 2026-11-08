/** Minimal shape shared by keyboard-navigable collection items. */
export interface NavigableCollectionItem {
  readonly disabled?: boolean;
}

/** Return the first enabled item, or -1 when the collection cannot be focused. */
export function firstEnabledIndex(items: readonly NavigableCollectionItem[]): number {
  return items.findIndex((item) => !item.disabled);
}

/** Return the last enabled item, or -1 when the collection cannot be focused. */
export function lastEnabledIndex(items: readonly NavigableCollectionItem[]): number {
  for (let index = items.length - 1; index >= 0; index--) {
    if (!items[index]?.disabled) return index;
  }
  return -1;
}

/**
 * Move to the next enabled item, wrapping at collection boundaries.
 *
 * A missing current index starts before the first item when moving forward and
 * after the last item when moving backward.
 */
export function adjacentEnabledIndex(
  items: readonly NavigableCollectionItem[],
  current: number,
  direction: -1 | 1,
): number {
  if (items.length === 0) return -1;

  const start = current >= 0 && current < items.length ? current : direction > 0 ? -1 : 0;
  for (let distance = 1; distance <= items.length; distance++) {
    const index = (start + direction * distance + items.length) % items.length;
    if (!items[index]?.disabled) return index;
  }
  return -1;
}

/** Find the next enabled item whose label starts with the typeahead query. */
export function typeaheadIndex<T extends NavigableCollectionItem>(
  items: readonly T[],
  current: number,
  query: string,
  label: (item: T) => string,
): number {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  if (!normalizedQuery || items.length === 0) return current;

  for (let distance = 1; distance <= items.length; distance++) {
    const index = (Math.max(current, -1) + distance) % items.length;
    const item = items[index];
    if (!item || item.disabled) continue;
    if (label(item).toLocaleLowerCase().startsWith(normalizedQuery)) return index;
  }
  return current;
}

/** Keep a highlighted item inside a bounded visible window. */
export function visibleCollectionOffset(
  highlightedIndex: number,
  currentOffset: number,
  windowSize: number | undefined,
  itemCount: number,
): number {
  if (!windowSize || windowSize <= 0 || itemCount <= windowSize || highlightedIndex < 0) return 0;

  const maximumOffset = itemCount - windowSize;
  if (highlightedIndex < currentOffset) return highlightedIndex;
  if (highlightedIndex >= currentOffset + windowSize) {
    return Math.min(highlightedIndex - windowSize + 1, maximumOffset);
  }
  return Math.min(currentOffset, maximumOffset);
}

import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** Comparator with `Array.prototype.sort` semantics. */
export type SortComparator<Item> = (left: Item, right: Item) => number;

/** Values with a natural order used by the default comparator. */
export type NaturallyOrdered = number | bigint | string | Date;

function naturalOrder(left: NaturallyOrdered, right: NaturallyOrdered): number {
  const a = left instanceof Date ? left.getTime() : left;
  const b = right instanceof Date ? right.getTime() : right;
  if (a < b) return -1;
  if (a > b) return 1;
  return 0;
}

/**
 * Sorted copy of a reactive array; the source is never mutated.
 *
 * Numbers, bigints, strings, and dates sort in natural ascending order by
 * default (numerically, unlike `Array.prototype.sort`); any other item type
 * requires a comparator. The sort is stable. Pure derived state: SSR-safe
 * and nothing to dispose.
 *
 * @example
 * ```ts
 * const ranked = useSorted(players, (a, b) => b.score - a.score);
 * const ascending = useSorted([10, 9, 100]); // [9, 10, 100]
 * ```
 *
 * @param source Reactive array.
 * @param compare Comparator; optional for naturally ordered items.
 * @returns Computed sorted copy.
 */
export function useSorted<Item extends NaturallyOrdered>(
  source: MaybeRefOrGetter<readonly Item[]>,
  compare?: SortComparator<Item>,
): ComputedRef<Item[]>;
export function useSorted<Item>(
  source: MaybeRefOrGetter<readonly Item[]>,
  compare: SortComparator<Item>,
): ComputedRef<Item[]>;
export function useSorted(
  source: MaybeRefOrGetter<readonly NaturallyOrdered[]>,
  compare: SortComparator<NaturallyOrdered> = naturalOrder,
): ComputedRef<NaturallyOrdered[]> {
  return computed(() => [...toValue(source)].sort(compare));
}

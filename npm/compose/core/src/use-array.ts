import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** Callback shape shared by the reactive array helpers. */
export type ArrayCallback<Item, Result> = (
  item: Item,
  index: number,
  array: readonly Item[],
) => Result;

/**
 * Reactive `Array.prototype.filter`. A type-guard predicate narrows the
 * element type.
 *
 * @example
 * ```ts
 * const active = useArrayFilter(users, (user) => user.active);
 * ```
 *
 * @param list Reactive array.
 * @param predicate Keeps items for which it returns `true`.
 * @returns Computed filtered copy.
 */
export function useArrayFilter<Item, Narrowed extends Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  predicate: (item: Item, index: number, array: readonly Item[]) => item is Narrowed,
): ComputedRef<Narrowed[]>;
export function useArrayFilter<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  predicate: ArrayCallback<Item, boolean>,
): ComputedRef<Item[]>;
export function useArrayFilter<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  predicate: ArrayCallback<Item, boolean>,
): ComputedRef<Item[]> {
  return computed(() => toValue(list).filter(predicate));
}

/**
 * Reactive `Array.prototype.map`.
 *
 * @example
 * ```ts
 * const names = useArrayMap(users, (user) => user.name);
 * ```
 *
 * @param list Reactive array.
 * @param mapper Produces the mapped item.
 * @returns Computed mapped copy.
 */
export function useArrayMap<Item, Mapped>(
  list: MaybeRefOrGetter<readonly Item[]>,
  mapper: ArrayCallback<Item, Mapped>,
): ComputedRef<Mapped[]> {
  return computed(() => toValue(list).map(mapper));
}

/**
 * Reactive `Array.prototype.find`. A type-guard predicate narrows the
 * result.
 *
 * @example
 * ```ts
 * const admin = useArrayFind(users, (user) => user.role === "admin");
 * ```
 *
 * @param list Reactive array.
 * @param predicate Matches the wanted item.
 * @returns Computed first match, or `undefined`.
 */
export function useArrayFind<Item, Narrowed extends Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  predicate: (item: Item, index: number, array: readonly Item[]) => item is Narrowed,
): ComputedRef<Narrowed | undefined>;
export function useArrayFind<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  predicate: ArrayCallback<Item, boolean>,
): ComputedRef<Item | undefined>;
export function useArrayFind<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  predicate: ArrayCallback<Item, boolean>,
): ComputedRef<Item | undefined> {
  return computed(() => toValue(list).find(predicate));
}

/**
 * Reactive `Array.prototype.reduce`.
 *
 * With an initial value the result has the accumulator type. Without one,
 * the first item seeds the accumulator and an empty array yields
 * `undefined` (instead of throwing like `reduce`).
 *
 * @example
 * ```ts
 * const total = useArrayReduce(cart, (sum, line) => sum + line.price, 0);
 * ```
 *
 * @param list Reactive array.
 * @param reducer Folds one item into the accumulator.
 * @param initial Reactive initial accumulator.
 * @returns Computed accumulated value.
 */
export function useArrayReduce<Item, Accumulator>(
  list: MaybeRefOrGetter<readonly Item[]>,
  reducer: (
    accumulator: Accumulator,
    item: Item,
    index: number,
    array: readonly Item[],
  ) => Accumulator,
  initial: MaybeRefOrGetter<Accumulator>,
): ComputedRef<Accumulator>;
export function useArrayReduce<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  reducer: (accumulator: Item, item: Item, index: number, array: readonly Item[]) => Item,
): ComputedRef<Item | undefined>;
export function useArrayReduce<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  reducer: (accumulator: unknown, item: Item, index: number, array: readonly Item[]) => unknown,
  ...initial: readonly [] | readonly [MaybeRefOrGetter<unknown>]
): ComputedRef<unknown> {
  return computed(() => {
    const items = toValue(list);
    const seeded = initial.length === 1;
    if (!seeded && items.length === 0) return undefined;
    let accumulator: unknown = seeded ? toValue(initial[0]) : items[0];
    for (const [index, item] of items.entries()) {
      if (seeded || index > 0) accumulator = reducer(accumulator, item, index, items);
    }
    return accumulator;
  });
}

/**
 * Reactive `Array.prototype.some`.
 *
 * @example
 * ```ts
 * const hasErrors = useArraySome(fields, (field) => field.error !== null);
 * ```
 *
 * @param list Reactive array.
 * @param predicate Condition tested per item.
 * @returns Computed flag.
 */
export function useArraySome<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  predicate: ArrayCallback<Item, boolean>,
): ComputedRef<boolean> {
  return computed(() => toValue(list).some(predicate));
}

/**
 * Reactive `Array.prototype.every`.
 *
 * @example
 * ```ts
 * const allDone = useArrayEvery(tasks, (task) => task.done);
 * ```
 *
 * @param list Reactive array.
 * @param predicate Condition tested per item.
 * @returns Computed flag.
 */
export function useArrayEvery<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  predicate: ArrayCallback<Item, boolean>,
): ComputedRef<boolean> {
  return computed(() => toValue(list).every(predicate));
}

/**
 * Reactive de-duplication that keeps the first occurrence of each item.
 *
 * Without `isSame`, items are compared with `SameValueZero` (like `Set`) in
 * linear time; with it, pairwise in quadratic time.
 *
 * @example
 * ```ts
 * const tags = useArrayUnique(allTags);
 * const people = useArrayUnique(rows, (left, right) => left.id === right.id);
 * ```
 *
 * @param list Reactive array.
 * @param isSame Custom equality.
 * @returns Computed de-duplicated copy.
 */
export function useArrayUnique<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  isSame?: (left: Item, right: Item) => boolean,
): ComputedRef<Item[]> {
  return computed(() => {
    const items = toValue(list);
    if (isSame === undefined) return [...new Set(items)];
    const unique: Item[] = [];
    for (const item of items) {
      if (!unique.some((kept) => isSame(kept, item))) unique.push(item);
    }
    return unique;
  });
}

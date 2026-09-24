import { computed, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef, WritableComputedRef } from "vue";

/** Options for {@link useCycleList}. */
export interface UseCycleListOptions<Item> {
  /**
   * Item selected initially.
   *
   * @default the first item
   */
  readonly initialValue?: Item;

  /**
   * Index used when the current item is not in the list.
   *
   * @default 0
   */
  readonly fallbackIndex?: number;

  /**
   * Locate an item in the list, for example by id.
   *
   * @default Array.prototype.indexOf
   */
  readonly getIndexOf?: (item: Item, list: readonly Item[]) => number;
}

/** Cursor over a list returned by {@link useCycleList}. */
export interface CycleList<Current, Item> {
  /** Current item. Writable: assigning moves the cursor to that item. */
  readonly state: ShallowRef<Current>;

  /** Current index. Writable: assigning wraps around the list. */
  readonly index: WritableComputedRef<number>;

  /**
   * Move forward `steps` items (default `1`), wrapping around.
   *
   * @returns The new current item.
   */
  readonly next: (steps?: number) => Current;

  /**
   * Move backward `steps` items (default `1`), wrapping around.
   *
   * @returns The new current item.
   */
  readonly prev: (steps?: number) => Current;

  /**
   * Jump to `index`, wrapping around.
   *
   * @returns The new current item.
   */
  readonly go: (index: number) => Current;

  /** The list items, for convenience. */
  readonly list: Readonly<ShallowRef<readonly Item[]>>;
}

/**
 * Cycle through a list with wrap-around navigation.
 *
 * Non-empty tuple sources (`["light", "dark", "system"] as const`) yield a
 * `state` typed as the element union; general arrays may be empty, so their
 * `state` includes `undefined`. When the list changes, the cursor keeps the
 * current item if it is still present and otherwise moves to
 * `fallbackIndex`. Synchronous state: SSR-safe; the list watcher follows the
 * owning scope.
 *
 * @example
 * ```ts
 * const { state, next } = useCycleList(["light", "dark", "system"] as const);
 * next(); // "dark"
 * ```
 *
 * @param list Reactive list to cycle through.
 * @param options Initial item, fallback index, and item lookup.
 * @default options {}
 * @returns The cursor and navigation controls.
 */
export function useCycleList<const List extends readonly [unknown, ...unknown[]]>(
  list: MaybeRefOrGetter<List>,
  options?: UseCycleListOptions<List[number]>,
): CycleList<List[number], List[number]>;
export function useCycleList<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  options?: UseCycleListOptions<Item>,
): CycleList<Item | undefined, Item>;
export function useCycleList<Item>(
  list: MaybeRefOrGetter<readonly Item[]>,
  options: UseCycleListOptions<Item> = {},
): CycleList<Item | undefined, Item> {
  const items = computed(() => toValue(list));
  const locate = options.getIndexOf ?? ((item: Item, all: readonly Item[]) => all.indexOf(item));
  const state = shallowRef<Item | undefined>(
    "initialValue" in options ? options.initialValue : items.value[0],
  );

  const wrap = (index: number): number => {
    const length = items.value.length;
    return length === 0 ? 0 : ((Math.trunc(index) % length) + length) % length;
  };

  const index = computed<number>({
    get: () => {
      const current = state.value;
      const found = current === undefined ? -1 : locate(current, items.value);
      return found === -1 ? wrap(options.fallbackIndex ?? 0) : found;
    },
    set: (next) => {
      state.value = items.value[wrap(next)];
    },
  });

  const go = (target: number): Item | undefined => {
    index.value = target;
    return state.value;
  };

  watch(items, () => {
    const current = state.value;
    if (current === undefined || locate(current, items.value) === -1) {
      state.value = items.value[wrap(options.fallbackIndex ?? 0)];
    }
  });

  return {
    state,
    index,
    next: (steps = 1) => go(index.value + steps),
    prev: (steps = 1) => go(index.value - steps),
    go,
    list: items,
  };
}

import { computed, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

/** Options for {@link useSelection}. */
export interface UseSelectionOptions<Item, Key, Multiple extends boolean> {
  /** Selectable items, in display order. Reactive. */
  readonly items: MaybeRefOrGetter<readonly Item[]>;

  /**
   * Allow several selected items.
   *
   * @default false
   */
  readonly multiple?: Multiple;

  /**
   * Stable identity of an item, for example its id. Keys survive item
   * objects being replaced (refetches, immutable updates).
   *
   * @default the item itself
   */
  readonly getKey?: (item: Item) => Key;

  /**
   * Keys selected initially.
   *
   * @default []
   */
  readonly initial?: readonly Key[];

  /**
   * Maximum number of selected items in multiple mode; selecting beyond it
   * is refused.
   *
   * @default Number.POSITIVE_INFINITY
   */
  readonly max?: number;

  /**
   * Refuse to deselect the last selected item.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Veto selecting an item (for example disabled rows).
   *
   * @default undefined (every item is selectable)
   */
  readonly isSelectable?: (item: Item) => boolean;
}

/** Selection state and commands returned by {@link useSelection}. */
export interface Selection<Item, Key, Multiple extends boolean> {
  /** Selected keys in selection order. */
  readonly selectedKeys: Readonly<ShallowRef<readonly Key[]>>;

  /**
   * Selected items in `items` order: an array in multiple mode, the single
   * item (or `undefined`) otherwise. Keys whose item is currently absent are
   * kept but not listed.
   */
  readonly selected: ComputedRef<Multiple extends true ? Item[] : Item | undefined>;

  /** Number of selected keys. */
  readonly count: ComputedRef<number>;

  /** Whether every selectable item is selected (multiple mode). */
  readonly isAllSelected: ComputedRef<boolean>;

  /** Whether some but not all selectable items are selected (tri-state checkboxes). */
  readonly isIndeterminate: ComputedRef<boolean>;

  /** Item that anchors range selection (the last explicitly selected one). */
  readonly anchor: Readonly<ShallowRef<Key | undefined>>;

  /** Whether `item` is selected. */
  readonly isSelected: (item: Item) => boolean;

  /**
   * Select `item` (replacing the selection in single mode).
   *
   * @returns Whether the selection changed.
   */
  readonly select: (item: Item) => boolean;

  /**
   * Deselect `item`.
   *
   * @returns Whether the selection changed.
   */
  readonly deselect: (item: Item) => boolean;

  /**
   * Toggle `item`.
   *
   * @returns Whether the selection changed.
   */
  readonly toggle: (item: Item) => boolean;

  /**
   * Select exactly `item`.
   *
   * @returns Whether the selection changed.
   */
  readonly selectOnly: (item: Item) => boolean;

  /**
   * Select every item between the anchor and `item` inclusive
   * (shift-click); in single mode this selects `item`.
   *
   * @returns Whether the selection changed.
   */
  readonly extendTo: (item: Item) => boolean;

  /** Select every selectable item, up to `max` (multiple mode). */
  readonly selectAll: () => void;

  /** Clear the selection (ignores `required`). */
  readonly clear: () => void;
}

/**
 * Generic single- or multi-select state for lists, tables, and grids.
 *
 * Keys (from `getKey`, the item itself by default) identify items, so the
 * selection survives items being replaced. `multiple: true` switches the
 * `selected` type to an array; `max`, `required`, and `isSelectable`
 * constrain changes, and `extendTo` implements shift-click ranges from the
 * anchor in display order. Synchronous state: SSR-safe and nothing to
 * dispose.
 *
 * @example
 * ```ts
 * const rows = useSelection({ items: users, multiple: true, getKey: (user) => user.id });
 * rows.toggle(user);
 * rows.extendTo(other); // shift-click
 * ```
 *
 * @param options Items, mode, identity, and constraints.
 * @returns Selection state and commands.
 */
export function useSelection<Item, Key = Item, const Multiple extends boolean = false>(
  options: UseSelectionOptions<Item, Key, Multiple>,
): Selection<Item, Key, Multiple>;
export function useSelection(
  options: UseSelectionOptions<unknown, unknown, boolean>,
): Selection<unknown, unknown, boolean> {
  const multiple = options.multiple ?? false;
  const max = multiple ? (options.max ?? Number.POSITIVE_INFINITY) : 1;
  const keyOf = options.getKey ?? ((item: unknown): unknown => item);
  const selectable = options.isSelectable ?? (() => true);
  const selectedKeys = shallowRef<readonly unknown[]>((options.initial ?? []).slice(0, max));
  const anchor = shallowRef<unknown>(selectedKeys.value.at(-1));
  const items = computed(() => toValue(options.items));

  const has = (key: unknown): boolean => selectedKeys.value.includes(key);
  const commit = (next: readonly unknown[]): boolean => {
    const current = selectedKeys.value;
    if (next.length === current.length && next.every((key, index) => key === current[index]))
      return false;
    selectedKeys.value = next;
    return true;
  };

  const select = (item: unknown): boolean => {
    if (!selectable(item)) return false;
    const key = keyOf(item);
    anchor.value = key;
    if (!multiple) return commit([key]);
    if (has(key) || selectedKeys.value.length >= max) return false;
    return commit([...selectedKeys.value, key]);
  };

  const deselect = (item: unknown): boolean => {
    const key = keyOf(item);
    if (!has(key)) return false;
    if ((options.required ?? false) && selectedKeys.value.length === 1) return false;
    return commit(selectedKeys.value.filter((candidate) => candidate !== key));
  };

  const selectableKeys = computed(() => items.value.filter(selectable).map(keyOf));
  const selectedItems = computed(() => items.value.filter((item) => has(keyOf(item))));

  return {
    selectedKeys,
    selected: computed(() => (multiple ? selectedItems.value : selectedItems.value[0])),
    count: computed(() => selectedKeys.value.length),
    isAllSelected: computed(
      () => selectableKeys.value.length > 0 && selectableKeys.value.every((key) => has(key)),
    ),
    isIndeterminate: computed(() => {
      const chosen = selectableKeys.value.filter((key) => has(key)).length;
      return chosen > 0 && chosen < selectableKeys.value.length;
    }),
    anchor,
    isSelected: (item) => has(keyOf(item)),
    select,
    deselect,
    toggle: (item) => (has(keyOf(item)) ? deselect(item) : select(item)),
    selectOnly: (item) => {
      if (!selectable(item)) return false;
      anchor.value = keyOf(item);
      return commit([keyOf(item)]);
    },
    extendTo: (item) => {
      if (!multiple) return select(item);
      const keys = items.value.map(keyOf);
      const from = keys.indexOf(anchor.value);
      const to = keys.indexOf(keyOf(item));
      if (to === -1) return false;
      if (from === -1) return select(item);
      const [start, end] = from <= to ? [from, to] : [to, from];
      const range = items.value
        .slice(start, end + 1)
        .filter(selectable)
        .map(keyOf)
        .filter((key) => !has(key));
      const room = Math.max(0, max - selectedKeys.value.length);
      return commit([...selectedKeys.value, ...range.slice(0, room)]);
    },
    selectAll: () => {
      if (!multiple) return;
      const missing = selectableKeys.value.filter((key) => !has(key));
      const room = Math.max(0, max - selectedKeys.value.length);
      commit([...selectedKeys.value, ...missing.slice(0, room)]);
    },
    clear: () => {
      commit([]);
    },
  };
}

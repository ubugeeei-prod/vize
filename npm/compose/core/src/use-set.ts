import { computed, shallowReactive, shallowReadonly } from "vue";
import type { ComputedRef } from "vue";

/** Reactive set wrapper returned by {@link useSet}. */
export interface SetControls<Item> {
  /**
   * Read-only reactive view of the set. `has`, iteration, and `size` on this
   * view are tracked. Members are stored shallowly.
   */
  readonly set: ReadonlySet<Item>;

  /** Number of members. */
  readonly size: ComputedRef<number>;

  /** Snapshot of the members in insertion order. */
  readonly values: ComputedRef<readonly Item[]>;

  /**
   * Whether `item` is a member. Reactive when read inside an effect.
   *
   * @param item Candidate member.
   * @returns Whether it is present.
   */
  readonly has: (item: Item) => boolean;

  /**
   * Add `item`.
   *
   * @param item Member to add.
   * @returns Whether it was newly added.
   */
  readonly add: (item: Item) => boolean;

  /**
   * Remove `item`.
   *
   * @param item Member to remove.
   * @returns Whether it was removed.
   */
  readonly delete: (item: Item) => boolean;

  /**
   * Add `item` when absent and remove it when present, or force membership.
   * Passing an explicit `undefined` behaves like passing no argument.
   *
   * @param item Member to toggle.
   * @param force Membership to force instead of inverting.
   * @returns Whether `item` is a member afterwards.
   */
  readonly toggle: (item: Item, force?: boolean) => boolean;

  /** Remove every member. */
  readonly clear: () => void;

  /**
   * Members of this set or `other`.
   *
   * @param other Items to combine with.
   * @returns A new, non-reactive set.
   */
  readonly union: (other: Iterable<Item>) => Set<Item>;

  /**
   * Members of this set that are also in `other`.
   *
   * @param other Items to intersect with.
   * @returns A new, non-reactive set.
   */
  readonly intersection: (other: Iterable<Item>) => Set<Item>;

  /**
   * Members of this set that are not in `other`.
   *
   * @param other Items to subtract.
   * @returns A new, non-reactive set.
   */
  readonly difference: (other: Iterable<Item>) => Set<Item>;

  /** Restore the creation-time members. */
  readonly reset: () => void;
}

/**
 * Create a reactive `Set` with typed helpers.
 *
 * The member type is inferred from the initial items (`useSet(["a"])` is a
 * `Set<string>`); pass a type argument for an initially empty set.
 * Membership checks are tracked per item. The set-algebra helpers return
 * new plain sets and do not require ES2025 `Set` methods.
 *
 * Purely synchronous state: safe during server rendering (no host globals,
 * no timers) and nothing to dispose.
 *
 * @example
 * ```ts
 * const selected = useSet<number>();
 * selected.toggle(3); // true
 * selected.has(3); // true
 * ```
 *
 * @typeParam Item Member type, inferred from `initial`.
 * @param initial Members added at creation and restored by `reset`.
 * @default initial []
 * @returns The reactive set and its helpers.
 */
export function useSet<Item = unknown>(initial: Iterable<Item> = []): SetControls<Item> {
  const baseline: readonly Item[] = [...initial];
  const set = shallowReactive(new Set<Item>(baseline));

  const add = (item: Item): boolean => {
    if (set.has(item)) return false;
    set.add(item);
    return true;
  };

  return {
    set: shallowReadonly(set),
    size: computed(() => set.size),
    values: computed(() => [...set.values()]),
    has: (item) => set.has(item),
    add,
    delete: (item) => set.delete(item),
    toggle: (item, force) => {
      const next = force ?? !set.has(item);
      if (next) add(item);
      else set.delete(item);
      return next;
    },
    clear: () => {
      set.clear();
    },
    union: (other) => {
      const result = new Set<Item>(set);
      for (const item of other) result.add(item);
      return result;
    },
    intersection: (other) => {
      const lookup = new Set<Item>(other);
      const result = new Set<Item>();
      for (const item of set) if (lookup.has(item)) result.add(item);
      return result;
    },
    difference: (other) => {
      const lookup = new Set<Item>(other);
      const result = new Set<Item>();
      for (const item of set) if (!lookup.has(item)) result.add(item);
      return result;
    },
    reset: () => {
      set.clear();
      for (const item of baseline) set.add(item);
    },
  };
}

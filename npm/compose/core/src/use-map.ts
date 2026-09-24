import { computed, shallowReactive, shallowReadonly } from "vue";
import type { ComputedRef } from "vue";

/** Reactive map wrapper returned by {@link useMap}. */
export interface MapControls<Key, Value> {
  /**
   * Read-only reactive view of the map. `get`, `has`, iteration, and `size`
   * on this view are tracked per key. Values are stored shallowly.
   */
  readonly map: ReadonlyMap<Key, Value>;

  /** Number of entries. */
  readonly size: ComputedRef<number>;

  /** Snapshot of the entries in insertion order. */
  readonly entries: ComputedRef<readonly (readonly [Key, Value])[]>;

  /**
   * Read the value for `key`. Reactive when read inside an effect.
   *
   * @param key Key to read.
   * @returns The value, or `undefined` when absent.
   */
  readonly get: (key: Key) => Value | undefined;

  /**
   * Whether `key` is present. Reactive when read inside an effect.
   *
   * @param key Key to test.
   * @returns Whether the key exists.
   */
  readonly has: (key: Key) => boolean;

  /**
   * Store `value` under `key`.
   *
   * @param key Key to write.
   * @param value Value to store.
   */
  readonly set: (key: Key, value: Value) => void;

  /**
   * Remove `key`.
   *
   * @param key Key to remove.
   * @returns Whether an entry was removed.
   */
  readonly delete: (key: Key) => boolean;

  /** Remove every entry. */
  readonly clear: () => void;

  /**
   * Replace the value under `key` with the result of `updater`.
   *
   * @param key Key to update.
   * @param updater Receives the current value (`undefined` when absent).
   * @returns The stored value.
   */
  readonly update: (key: Key, updater: (previous: Value | undefined) => Value) => Value;

  /**
   * Return the value under `key`, inserting `create(key)` first when absent.
   *
   * @param key Key to read or insert.
   * @param create Factory for a missing value.
   * @returns The existing or inserted value.
   */
  readonly getOrInsert: (key: Key, create: (key: Key) => Value) => Value;

  /** Restore the creation-time entries. */
  readonly reset: () => void;
}

/**
 * Narrow a `get` result using the matching `has` answer: when the key is
 * present the result is a stored `Value`, even if that value is `undefined`.
 */
function isStored<Value>(value: Value | undefined, present: boolean): value is Value {
  return present;
}

/**
 * Create a reactive `Map` with typed helpers.
 *
 * Key and value types are inferred from the initial entries
 * (`useMap([["a", 1]])` is a `Map<string, number>`); pass type arguments
 * for an initially empty map. Reads through `get`, `has`, and `map` are
 * tracked per key, so an effect reading one key does not re-run when an
 * unrelated key changes. Values are stored as-is (not deeply reactive).
 *
 * Purely synchronous state: safe during server rendering (no host globals,
 * no timers) and nothing to dispose.
 *
 * @example
 * ```ts
 * const counts = useMap<string, number>();
 * counts.update("clicks", (previous = 0) => previous + 1);
 * ```
 *
 * @typeParam Key Key type, inferred from `initial`.
 * @typeParam Value Value type, inferred from `initial`.
 * @param initial Entries inserted at creation and restored by `reset`.
 * @default initial []
 * @returns The reactive map and its helpers.
 */
export function useMap<Key = unknown, Value = unknown>(
  initial: Iterable<readonly [Key, Value]> = [],
): MapControls<Key, Value> {
  const baseline: readonly (readonly [Key, Value])[] = [...initial];
  const map = shallowReactive(new Map<Key, Value>(baseline));

  const reset = (): void => {
    map.clear();
    for (const [key, value] of baseline) map.set(key, value);
  };

  return {
    map: shallowReadonly(map),
    size: computed(() => map.size),
    entries: computed(() => [...map.entries()]),
    get: (key) => map.get(key),
    has: (key) => map.has(key),
    set: (key, value) => {
      map.set(key, value);
    },
    delete: (key) => map.delete(key),
    clear: () => {
      map.clear();
    },
    update: (key, updater) => {
      const next = updater(map.get(key));
      map.set(key, next);
      return next;
    },
    getOrInsert: (key, create) => {
      const existing = map.get(key);
      if (isStored(existing, map.has(key))) return existing;
      const created = create(key);
      map.set(key, created);
      return created;
    },
    reset,
  };
}

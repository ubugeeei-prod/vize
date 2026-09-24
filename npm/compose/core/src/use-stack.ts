import { computed, shallowRef } from "vue";
import type { ComputedRef } from "vue";

/** What happens when an item is pushed while a bounded stack is full. */
export type StackOverflowPolicy = "drop-oldest" | "reject";

/** Options for {@link useStack}. */
export interface UseStackOptions {
  /**
   * Maximum number of stacked items. A positive integer, or
   * `Number.POSITIVE_INFINITY` for an unbounded stack.
   *
   * @default Number.POSITIVE_INFINITY
   */
  readonly capacity?: number;

  /**
   * Policy applied when an item is pushed at capacity: `"drop-oldest"`
   * discards the bottom item to make room, `"reject"` refuses the new item.
   *
   * @default "drop-oldest"
   */
  readonly overflow?: StackOverflowPolicy;
}

/** Reactive last-in, first-out stack returned by {@link useStack}. */
export interface StackControls<Item> {
  /** Snapshot of the stacked items, bottom first (the top is the last element). */
  readonly items: ComputedRef<readonly Item[]>;

  /** Number of stacked items. */
  readonly size: ComputedRef<number>;

  /** Whether the stack holds no items. */
  readonly isEmpty: ComputedRef<boolean>;

  /**
   * Read the top without removing it. Reactive when read inside an effect.
   *
   * @returns The top item, or `undefined` when empty.
   */
  readonly peek: () => Item | undefined;

  /**
   * Push items in argument order (the last argument ends on top), applying
   * the overflow policy.
   *
   * @param items Items to push.
   * @returns How many of the given items were accepted.
   */
  readonly push: (...items: Item[]) => number;

  /**
   * Remove and return the top.
   *
   * @returns The removed item, or `undefined` when empty.
   */
  readonly pop: () => Item | undefined;

  /** Remove every item. */
  readonly clear: () => void;
}

function validateCapacity(capacity: number): number {
  if (capacity === Number.POSITIVE_INFINITY) return capacity;
  if (!Number.isSafeInteger(capacity) || capacity < 1) {
    throw new RangeError(
      `[VIZE_COMPOSE_STACK_INVALID_CAPACITY] capacity must be a positive integer or Infinity; received ${String(capacity)}`,
    );
  }
  return capacity;
}

/**
 * Create a typed reactive LIFO stack.
 *
 * `push` and `pop` are O(1). A bounded stack either drops its bottom item
 * or rejects new pushes when full. Reactivity is driven by a version
 * counter, so `items`, `size`, `isEmpty`, and `peek` re-evaluate after
 * every mutation.
 *
 * Purely synchronous state: safe during server rendering (no host globals,
 * no timers) and nothing to dispose.
 *
 * @example
 * ```ts
 * const breadcrumbs = useStack<string>(["home"]);
 * breadcrumbs.push("settings");
 * breadcrumbs.pop(); // "settings"
 * ```
 *
 * @typeParam Item Stacked item type, inferred from `initial` when given.
 * @param initial Items pushed at creation, bottom first (overflow applies).
 * @param options Capacity and overflow policy.
 * @default initial []
 * @default options {}
 * @throws {RangeError} `[VIZE_COMPOSE_STACK_INVALID_CAPACITY]` when `capacity`
 * is not a positive integer or `Infinity`.
 * @returns The reactive stack and its controls.
 */
export function useStack<Item>(
  initial: Iterable<Item> = [],
  options: UseStackOptions = {},
): StackControls<Item> {
  const capacity = validateCapacity(options.capacity ?? Number.POSITIVE_INFINITY);
  const overflow = options.overflow ?? "drop-oldest";
  let buffer: Item[] = [];
  const version = shallowRef(0);
  const touch = (): void => {
    version.value += 1;
  };

  const append = (items: readonly Item[]): number => {
    let accepted = 0;
    let dropped = 0;
    for (const item of items) {
      if (buffer.length - dropped >= capacity) {
        if (overflow === "reject") continue;
        dropped += 1;
      }
      buffer.push(item);
      accepted += 1;
    }
    if (dropped > 0) buffer = buffer.slice(dropped);
    return accepted;
  };

  append([...initial]);

  const items = computed<readonly Item[]>(() => {
    void version.value;
    return buffer.slice();
  });
  const size = computed(() => {
    void version.value;
    return buffer.length;
  });

  return {
    items,
    size,
    isEmpty: computed(() => size.value === 0),
    peek: () => {
      void version.value;
      return buffer.at(-1);
    },
    push: (...added) => {
      const accepted = append(added);
      if (accepted > 0) touch();
      return accepted;
    },
    pop: () => {
      if (buffer.length === 0) return undefined;
      const item = buffer.pop();
      touch();
      return item;
    },
    clear: () => {
      if (buffer.length === 0) return;
      buffer = [];
      touch();
    },
  };
}

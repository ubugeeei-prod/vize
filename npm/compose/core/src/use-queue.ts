import { computed, shallowRef } from "vue";
import type { ComputedRef } from "vue";

/** What happens when an item arrives while a bounded queue is full. */
export type QueueOverflowPolicy = "drop-oldest" | "reject";

/** Options for {@link useQueue}. */
export interface UseQueueOptions {
  /**
   * Maximum number of queued items. A positive integer, or
   * `Number.POSITIVE_INFINITY` for an unbounded queue.
   *
   * @default Number.POSITIVE_INFINITY
   */
  readonly capacity?: number;

  /**
   * Policy applied when an item arrives at capacity: `"drop-oldest"` evicts
   * the head to make room, `"reject"` refuses the new item.
   *
   * @default "drop-oldest"
   */
  readonly overflow?: QueueOverflowPolicy;
}

/** Reactive first-in, first-out queue returned by {@link useQueue}. */
export interface QueueControls<Item> {
  /** Snapshot of the queued items, head first. Recomputed after every change. */
  readonly items: ComputedRef<readonly Item[]>;

  /** Number of queued items. */
  readonly size: ComputedRef<number>;

  /** Whether the queue holds no items. */
  readonly isEmpty: ComputedRef<boolean>;

  /**
   * Read the head without removing it. Reactive when read inside an effect.
   *
   * @returns The head item, or `undefined` when empty.
   */
  readonly peek: () => Item | undefined;

  /**
   * Append items at the tail in argument order, applying the overflow policy.
   *
   * @param items Items to append.
   * @returns How many of the given items were accepted.
   */
  readonly enqueue: (...items: Item[]) => number;

  /**
   * Remove and return the head.
   *
   * @returns The removed item, or `undefined` when empty.
   */
  readonly dequeue: () => Item | undefined;

  /**
   * Remove and return every queued item, head first.
   *
   * @returns The removed items.
   */
  readonly drain: () => Item[];

  /** Remove every item. */
  readonly clear: () => void;
}

/** Compact the backing buffer once this many dequeued slots accumulate. */
const COMPACT_THRESHOLD = 32;

function validateCapacity(capacity: number): number {
  if (capacity === Number.POSITIVE_INFINITY) return capacity;
  if (!Number.isSafeInteger(capacity) || capacity < 1) {
    throw new RangeError(
      `[VIZE_COMPOSE_QUEUE_INVALID_CAPACITY] capacity must be a positive integer or Infinity; received ${String(capacity)}`,
    );
  }
  return capacity;
}

/**
 * Create a typed reactive FIFO queue.
 *
 * Dequeueing is amortized O(1): the head advances through a backing buffer
 * that is compacted only after enough slots have been consumed, so large
 * queues never pay for `Array.prototype.shift`. Reactivity is driven by a
 * single version counter; `items`, `size`, `isEmpty`, and `peek` all
 * re-evaluate after each mutation.
 *
 * Purely synchronous state: safe during server rendering (no host globals,
 * no timers) and nothing to dispose.
 *
 * @example
 * ```ts
 * const jobs = useQueue<string>([], { capacity: 100, overflow: "reject" });
 * jobs.enqueue("a", "b");
 * jobs.dequeue(); // "a"
 * ```
 *
 * @typeParam Item Queued item type, inferred from `initial` when given.
 * @param initial Items enqueued at creation, head first (overflow applies).
 * @param options Capacity and overflow policy.
 * @default initial []
 * @default options {}
 * @throws {RangeError} `[VIZE_COMPOSE_QUEUE_INVALID_CAPACITY]` when `capacity`
 * is not a positive integer or `Infinity`.
 * @returns The reactive queue and its controls.
 */
export function useQueue<Item>(
  initial: Iterable<Item> = [],
  options: UseQueueOptions = {},
): QueueControls<Item> {
  const capacity = validateCapacity(options.capacity ?? Number.POSITIVE_INFINITY);
  const overflow = options.overflow ?? "drop-oldest";
  let buffer: Item[] = [];
  let head = 0;
  const version = shallowRef(0);
  const touch = (): void => {
    version.value += 1;
  };
  const length = (): number => buffer.length - head;

  const compact = (): void => {
    if (head >= COMPACT_THRESHOLD && head * 2 >= buffer.length) {
      buffer = buffer.slice(head);
      head = 0;
    }
  };

  const append = (items: readonly Item[]): number => {
    let accepted = 0;
    for (const item of items) {
      if (length() >= capacity) {
        if (overflow === "reject") continue;
        head += 1;
      }
      buffer.push(item);
      accepted += 1;
    }
    compact();
    return accepted;
  };

  append([...initial]);

  const items = computed<readonly Item[]>(() => {
    void version.value;
    return buffer.slice(head);
  });
  const size = computed(() => {
    void version.value;
    return length();
  });

  return {
    items,
    size,
    isEmpty: computed(() => size.value === 0),
    peek: () => {
      void version.value;
      return length() > 0 ? buffer[head] : undefined;
    },
    enqueue: (...added) => {
      const accepted = append(added);
      if (accepted > 0) touch();
      return accepted;
    },
    dequeue: () => {
      if (length() === 0) return undefined;
      const item = buffer[head];
      head += 1;
      if (head === buffer.length) {
        buffer = [];
        head = 0;
      } else {
        compact();
      }
      touch();
      return item;
    },
    drain: () => {
      const drained = buffer.slice(head);
      buffer = [];
      head = 0;
      if (drained.length > 0) touch();
      return drained;
    },
    clear: () => {
      if (length() === 0) return;
      buffer = [];
      head = 0;
      touch();
    },
  };
}

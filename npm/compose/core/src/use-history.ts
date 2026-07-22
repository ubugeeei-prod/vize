import { computed, shallowRef, watch } from "vue";
import type { ComputedRef, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Options for {@link useHistory}. */
export interface UseHistoryOptions<Value> {
  /**
   * Maximum number of undo entries retained; recording a change beyond it
   * drops the oldest entry. The redo stack is bounded by construction, since
   * redo entries only ever come from undone changes. Must be an integer
   * greater than zero and is fixed at creation.
   *
   * @default 100
   */
  readonly capacity?: number;

  /**
   * Clone applied to every value captured into history and to every value
   * restored out of it, isolating snapshots from later in-place mutation.
   *
   * @default identity — values are stored and restored by reference
   */
  readonly clone?: (value: Value) => Value;
}

/** Reactive undo/redo controls returned by {@link useHistory}. */
export interface HistoryControls {
  /** Whether {@link HistoryControls.undo} currently has an entry to restore. */
  readonly canUndo: ComputedRef<boolean>;

  /** Whether {@link HistoryControls.redo} currently has an entry to restore. */
  readonly canRedo: ComputedRef<boolean>;

  /** Number of retained undo entries. */
  readonly undoCount: ComputedRef<number>;

  /** Number of retained redo entries. */
  readonly redoCount: ComputedRef<number>;

  /**
   * Restore the newest undo entry and move the current value onto the redo
   * stack. The restoring write is not recorded.
   *
   * @returns Whether an entry was restored.
   */
  readonly undo: () => boolean;

  /**
   * Restore the newest redo entry and move the current value back onto the
   * undo stack. The restoring write is not recorded.
   *
   * @returns Whether an entry was restored.
   */
  readonly redo: () => boolean;

  /**
   * Group every source write inside `run` into at most one undo entry.
   *
   * The entry restores the value from just before the batch. It is committed
   * only when the final value differs (`Object.is`) from the starting value,
   * and it is committed even when `run` throws, so a partially applied batch
   * stays undoable as one step. Nested calls collapse into the outermost
   * batch. The callback's return value is passed through.
   */
  readonly batch: <Result>(run: () => Result) => Result;

  /** Drop every undo and redo entry while keeping the current value. */
  readonly clear: () => void;
}

interface HistoryEntry<Value> {
  readonly value: Value;
}

/**
 * Record bounded undo/redo history over the writes of a ref.
 *
 * Recording is shallow and identity-based, matching Vue's own change
 * detection: assignments to `source.value` are recorded (observed with
 * `flush: "sync"`, so every synchronous write counts), writes that are
 * `Object.is`-equal to the current value are not changes, and in-place
 * mutations of object values are invisible — pair mutable values with
 * {@link UseHistoryOptions.clone} and reassign. Undoing and redoing restore
 * values through `clone` as well, so snapshots never share identity with the
 * live value unless the default identity clone is kept. When a
 * user-provided `clone` throws, the failed operation leaves history
 * unchanged and the error propagates.
 *
 * Safe during server rendering: no browser globals are read and no timers
 * start. Cleanup rule: when the owning reactive scope stops, recording stops
 * and every retained snapshot is released, so `undo`/`redo` return `false`
 * afterwards; call inside an active scope, or the watcher lives as long as
 * the source.
 *
 * @example
 * ```ts
 * const text = shallowRef("");
 * const { undo, redo, batch } = useHistory(text);
 * text.value = "a";
 * batch(() => {
 *   text.value = "ab";
 *   text.value = "abc";
 * });
 * undo(); // text.value === "a" (the batch is one step)
 * redo(); // text.value === "abc"
 * ```
 *
 * @param source Ref whose writes are recorded.
 * @param options Retention bound and snapshot cloning.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_HISTORY_INVALID_CAPACITY` when
 * the capacity is not an integer greater than zero.
 * @throws `Error` tagged `VIZE_COMPOSE_HISTORY_IN_BATCH` when `undo`,
 * `redo`, or `clear` is called inside {@link HistoryControls.batch}, where
 * stack movement would corrupt the pending group.
 * @returns Reactive undo/redo state and controls.
 */
export function useHistory<Value>(
  source: Ref<Value>,
  options: UseHistoryOptions<Value> = {},
): HistoryControls {
  const capacity = options.capacity ?? 100;
  if (!Number.isInteger(capacity) || capacity < 1) {
    throw new RangeError(
      `[VIZE_COMPOSE_HISTORY_INVALID_CAPACITY] capacity must be an integer greater than zero; received ${String(capacity)}`,
    );
  }
  const clone = options.clone ?? ((value: Value) => value);
  const undoStack: ShallowRef<readonly HistoryEntry<Value>[]> = shallowRef([]);
  const redoStack: ShallowRef<readonly HistoryEntry<Value>[]> = shallowRef([]);
  let restoring = false;
  let batchDepth = 0;
  let activeBatch: { readonly raw: Value; readonly entry: HistoryEntry<Value> } | undefined;

  const pushUndo = (entry: HistoryEntry<Value>): void => {
    const next = [...undoStack.value, entry];
    undoStack.value = next.length > capacity ? next.slice(next.length - capacity) : next;
    redoStack.value = [];
  };

  const writeSilently = (value: Value): void => {
    restoring = true;
    try {
      source.value = value;
    } finally {
      restoring = false;
    }
  };

  const requireOutsideBatch = (operation: string): void => {
    if (batchDepth > 0) {
      throw new Error(
        `[VIZE_COMPOSE_HISTORY_IN_BATCH] ${operation}() is not available inside batch()`,
      );
    }
  };

  const undo = (): boolean => {
    requireOutsideBatch("undo");
    const entry = undoStack.value.at(-1);
    if (entry === undefined) return false;
    const restored = clone(entry.value);
    const recorded: HistoryEntry<Value> = { value: clone(source.value) };
    undoStack.value = undoStack.value.slice(0, -1);
    redoStack.value = [...redoStack.value, recorded];
    writeSilently(restored);
    return true;
  };

  const redo = (): boolean => {
    requireOutsideBatch("redo");
    const entry = redoStack.value.at(-1);
    if (entry === undefined) return false;
    const restored = clone(entry.value);
    const recorded: HistoryEntry<Value> = { value: clone(source.value) };
    redoStack.value = redoStack.value.slice(0, -1);
    undoStack.value = [...undoStack.value, recorded];
    writeSilently(restored);
    return true;
  };

  const batch = <Result>(run: () => Result): Result => {
    if (batchDepth === 0) {
      activeBatch = { raw: source.value, entry: { value: clone(source.value) } };
    }
    batchDepth += 1;
    try {
      return run();
    } finally {
      batchDepth -= 1;
      if (batchDepth === 0 && activeBatch !== undefined) {
        const finished = activeBatch;
        activeBatch = undefined;
        if (!Object.is(finished.raw, source.value)) pushUndo(finished.entry);
      }
    }
  };

  const clear = (): void => {
    requireOutsideBatch("clear");
    undoStack.value = [];
    redoStack.value = [];
  };

  const handle = watch(
    source,
    (_next, replaced) => {
      if (restoring || batchDepth > 0) return;
      pushUndo({ value: clone(replaced) });
    },
    { flush: "sync" },
  );

  tryOnScopeDispose(() => {
    handle.stop();
    undoStack.value = [];
    redoStack.value = [];
  });

  return {
    canUndo: computed(() => undoStack.value.length > 0),
    canRedo: computed(() => redoStack.value.length > 0),
    undoCount: computed(() => undoStack.value.length),
    redoCount: computed(() => redoStack.value.length),
    undo,
    redo,
    batch,
    clear,
  };
}

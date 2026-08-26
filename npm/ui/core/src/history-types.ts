import type { ShallowRef } from "vue";

/** One undoable unit of work whose effects are already applied. */
export interface HistoryEntry {
  /** Reverse the applied change. */
  readonly undo: () => void;

  /** Re-apply the change after it was undone. */
  readonly redo: () => void;

  /** Human-readable label for Undo/Redo menu items. */
  readonly label?: string;

  /**
   * Adjacent pushes carrying the same key merge into one step while they
   * arrive within the coalescing window and nothing was undone in between.
   */
  readonly coalesceKey?: string;
}

/** Value-based entry description used by `HistoryController.pushSnapshot`. */
export interface HistorySnapshotOptions<Value> {
  /** State before the change. */
  readonly before: Value;

  /** State after the change. */
  readonly after: Value;

  /** Writer invoked with `before` on undo and `after` on redo. */
  readonly apply: (value: Value) => void;

  /** Human-readable label for Undo/Redo menu items. */
  readonly label?: string;

  /** Coalescing key; see {@link HistoryEntry.coalesceKey}. */
  readonly coalesceKey?: string;

  /**
   * Equality used to drop no-op snapshots.
   *
   * @default Object.is
   */
  readonly isEqual?: (left: Value, right: Value) => boolean;
}

/** Open transaction handle returned by `HistoryController.beginTransaction`. */
export interface HistoryTransaction {
  /** Fold everything staged since `beginTransaction` into one step. */
  readonly commit: () => void;

  /** Undo and discard everything staged since `beginTransaction`. */
  readonly rollback: () => void;
}

/** Options for `createHistory` and `useHistory`. */
export interface HistoryOptions {
  /**
   * Maximum retained undo steps; the oldest step is dropped beyond it.
   *
   * @default 1000
   */
  readonly limit?: number;

  /**
   * Milliseconds within which same-key pushes coalesce into one step.
   *
   * @default 1000
   */
  readonly coalesceWindow?: number;

  /**
   * Monotonic clock used for coalescing decisions, injectable for tests.
   *
   * @default Date.now
   */
  readonly now?: () => number;
}

/** Undo/redo timeline with coalescing, transactions, and snapshots. */
export interface HistoryController {
  /** Whether at least one step can be undone. */
  readonly canUndo: Readonly<ShallowRef<boolean>>;

  /** Whether at least one step can be redone. */
  readonly canRedo: Readonly<ShallowRef<boolean>>;

  /** Number of undoable steps. */
  readonly undoDepth: Readonly<ShallowRef<number>>;

  /** Number of redoable steps. */
  readonly redoDepth: Readonly<ShallowRef<number>>;

  /** Label of the next undo step, or `null`. */
  readonly undoLabel: Readonly<ShallowRef<string | null>>;

  /** Label of the next redo step, or `null`. */
  readonly redoLabel: Readonly<ShallowRef<string | null>>;

  /** True while an undo or redo is re-applying state. */
  readonly isRestoring: Readonly<ShallowRef<boolean>>;

  /** Record one already-applied entry, coalescing when eligible. */
  readonly push: (entry: HistoryEntry) => void;

  /** Record one already-applied value change, dropping no-op snapshots. */
  readonly pushSnapshot: <Value>(options: HistorySnapshotOptions<Value>) => void;

  /** Undo one step and report whether one existed. */
  readonly undo: () => boolean;

  /** Redo one step and report whether one existed. */
  readonly redo: () => boolean;

  /** Open a transaction that folds staged pushes into one step. */
  readonly beginTransaction: (label?: string) => HistoryTransaction;

  /** Run `work` inside a transaction, rolling back when it throws. */
  readonly transaction: <Result>(label: string | undefined, work: () => Result) => Result;

  /** Drop both timelines without touching application state. */
  readonly clear: () => void;

  /** Clear the timeline and make imperative calls terminal. */
  readonly dispose: () => void;
}

/** Options for `createEditableTransaction`. */
export interface EditableTransactionOptions<Value> {
  /** History timeline that receives one step per committed edit. */
  readonly history: HistoryController;

  /** Read the current value from the editor. */
  readonly read: () => Value;

  /** Write a value back into the editor. */
  readonly write: (value: Value) => void;

  /** Label recorded on committed steps. */
  readonly label?: string;

  /** Coalescing key recorded on committed steps. */
  readonly coalesceKey?: string;

  /**
   * Equality used to drop unchanged commits.
   *
   * @default Object.is
   */
  readonly isEqual?: (left: Value, right: Value) => boolean;
}

/** In-place editing session that commits one history step per edit. */
export interface EditableTransaction<Value> {
  /** True between `begin` and the next `commit` or `cancel`. */
  readonly isEditing: Readonly<ShallowRef<boolean>>;

  /** Capture the pre-edit value. Idempotent while editing. */
  readonly begin: () => void;

  /** Write a live value, beginning the session when necessary. */
  readonly update: (value: Value) => void;

  /** Push one step for the whole session; report whether one was pushed. */
  readonly commit: () => boolean;

  /** Restore the pre-edit value; report whether a session was open. */
  readonly cancel: () => boolean;
}

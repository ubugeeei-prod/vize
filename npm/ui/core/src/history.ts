import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef } from "vue";

import {
  foldRecords,
  resolveHistoryOptions,
  toRecord,
  toSnapshotEntry,
  tryCoalesce,
  type HistoryRecord,
} from "./history-stack.ts";
import type {
  HistoryController,
  HistoryEntry,
  HistoryOptions,
  HistorySnapshotOptions,
  HistoryTransaction,
} from "./history-types.ts";

const disposedDiagnostic = "VIZE_UI_HISTORY_DISPOSED";
const setupDiagnostic = "VIZE_UI_HISTORY_SETUP";
const transactionDiagnostic = "VIZE_UI_HISTORY_TRANSACTION";

/**
 * Create an undo/redo timeline with coalescing, transactions, and snapshots.
 *
 * Entries describe changes that are already applied; the controller only
 * replays them. It owns no DOM or timers, so it is safe to create during
 * server rendering. Pushes that happen while the controller is restoring
 * state are discarded so reactive mirrors cannot corrupt the timeline. Call
 * {@link HistoryController.dispose} when using this factory outside a Vue
 * effect scope.
 */
export function createHistory(options: HistoryOptions = {}): HistoryController {
  const resolved = resolveHistoryOptions(options);
  const undoStack: HistoryRecord[] = [];
  const redoStack: HistoryRecord[] = [];
  const staged: HistoryRecord[] = [];
  const transactions: { readonly label: string | null; readonly startIndex: number }[] = [];
  const canUndo = shallowRef(false);
  const canRedo = shallowRef(false);
  const undoDepth = shallowRef(0);
  const redoDepth = shallowRef(0);
  const undoLabel = shallowRef<string | null>(null);
  const redoLabel = shallowRef<string | null>(null);
  const isRestoring = shallowRef(false);
  let disposed = false;

  const assertActive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the history has been disposed`);
  };
  const assertNoTransaction = (operation: string) => {
    if (transactions.length > 0) {
      throw new Error(`${transactionDiagnostic}: cannot ${operation} while a transaction is open`);
    }
  };
  const sync = () => {
    canUndo.value = undoStack.length > 0;
    canRedo.value = redoStack.length > 0;
    undoDepth.value = undoStack.length;
    redoDepth.value = redoStack.length;
    undoLabel.value = undoStack.at(-1)?.label ?? null;
    redoLabel.value = redoStack.at(-1)?.label ?? null;
  };
  const commitRecord = (record: HistoryRecord) => {
    redoStack.length = 0;
    if (!tryCoalesce(undoStack.at(-1), record, resolved.coalesceWindow)) {
      undoStack.push(record);
      if (undoStack.length > resolved.limit) undoStack.shift();
    }
    sync();
  };
  const push = (entry: HistoryEntry) => {
    assertActive();
    const record = toRecord(entry, resolved.now());
    if (isRestoring.value) return;
    if (transactions.length > 0) {
      staged.push(record);
      return;
    }
    commitRecord(record);
  };
  const replay = (from: HistoryRecord[], to: HistoryRecord[], action: "undo" | "redo") => {
    assertActive();
    assertNoTransaction(action);
    const record = from.pop();
    if (record === undefined) return false;
    isRestoring.value = true;
    try {
      record[action]();
    } finally {
      isRestoring.value = false;
      sync();
    }
    to.push(record);
    sync();
    return true;
  };

  const beginTransaction = (label?: string): HistoryTransaction => {
    assertActive();
    if (label !== undefined && typeof label !== "string") {
      throw new TypeError(`${transactionDiagnostic}: label must be a string`);
    }
    const frame = { label: label ?? null, startIndex: staged.length };
    transactions.push(frame);
    let settled = false;
    const assertOpen = () => {
      assertActive();
      if (settled || transactions.at(-1) !== frame) {
        throw new Error(`${transactionDiagnostic}: transactions settle once, innermost first`);
      }
      settled = true;
      transactions.pop();
    };
    return Object.freeze({
      commit: () => {
        assertOpen();
        if (transactions.length > 0) return;
        const records = staged.splice(0);
        if (records.length === 0) return;
        if (records.length === 1 && frame.label === null) {
          commitRecord(records[0]!);
          return;
        }
        commitRecord(foldRecords(records, frame.label, resolved.now()));
      },
      rollback: () => {
        assertOpen();
        const records = staged.splice(frame.startIndex);
        isRestoring.value = true;
        try {
          for (let index = records.length - 1; index >= 0; index -= 1) records[index]!.undo();
        } finally {
          isRestoring.value = false;
        }
      },
    });
  };

  return Object.freeze({
    canUndo: shallowReadonly(canUndo),
    canRedo: shallowReadonly(canRedo),
    undoDepth: shallowReadonly(undoDepth),
    redoDepth: shallowReadonly(redoDepth),
    undoLabel: shallowReadonly(undoLabel),
    redoLabel: shallowReadonly(redoLabel),
    isRestoring: shallowReadonly(isRestoring),
    push,
    pushSnapshot: <Value>(snapshot: HistorySnapshotOptions<Value>) => {
      assertActive();
      const entry = toSnapshotEntry(snapshot);
      if (entry !== null) push(entry);
    },
    undo: () => replay(undoStack, redoStack, "undo"),
    redo: () => replay(redoStack, undoStack, "redo"),
    beginTransaction,
    transaction: <Result>(label: string | undefined, work: () => Result): Result => {
      if (typeof work !== "function") {
        throw new TypeError(`${transactionDiagnostic}: work must be a function`);
      }
      const frame = beginTransaction(label);
      let result: Result;
      try {
        result = work();
      } catch (error) {
        frame.rollback();
        throw error;
      }
      frame.commit();
      return result;
    },
    clear: () => {
      assertActive();
      assertNoTransaction("clear");
      undoStack.length = 0;
      redoStack.length = 0;
      sync();
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      undoStack.length = 0;
      redoStack.length = 0;
      staged.length = 0;
      transactions.length = 0;
      sync();
    },
  });
}

/** Create a history controller disposed with the current Vue effect scope. */
export function useHistory(options: HistoryOptions = {}): HistoryController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const history = createHistory(options);
  onScopeDispose(history.dispose);
  return history;
}

export { createEditableTransaction } from "./history-editable.ts";
export type {
  EditableTransaction,
  EditableTransactionOptions,
  HistoryController,
  HistoryEntry,
  HistoryOptions,
  HistorySnapshotOptions,
  HistoryTransaction,
} from "./history-types.ts";

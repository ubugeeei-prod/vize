import type { HistoryEntry, HistoryOptions, HistorySnapshotOptions } from "./history-types.ts";

const optionDiagnostic = "VIZE_UI_HISTORY_OPTION";

/** Mutable internal record; merging rewrites `redo`, `label`, and `at`. */
export interface HistoryRecord {
  undo: () => void;
  redo: () => void;
  label: string | null;
  coalesceKey: string | null;
  at: number;
}

export interface ResolvedHistoryOptions {
  readonly limit: number;
  readonly coalesceWindow: number;
  readonly now: () => number;
}

export function resolveHistoryOptions(options: HistoryOptions): ResolvedHistoryOptions {
  const limit = options.limit ?? 1000;
  if (!Number.isInteger(limit) || limit < 1) {
    throw new TypeError(`${optionDiagnostic}: limit must be an integer >= 1`);
  }
  const coalesceWindow = options.coalesceWindow ?? 1000;
  if (
    typeof coalesceWindow !== "number" ||
    !Number.isFinite(coalesceWindow) ||
    coalesceWindow < 0
  ) {
    throw new TypeError(`${optionDiagnostic}: coalesceWindow must be a finite number >= 0`);
  }
  const now = options.now ?? Date.now;
  if (typeof now !== "function") {
    throw new TypeError(`${optionDiagnostic}: now must be a function`);
  }
  return Object.freeze({ limit, coalesceWindow, now });
}

function readOptionalString(value: string | undefined, name: string): string | null {
  if (value === undefined) return null;
  if (typeof value !== "string") {
    throw new TypeError(`${optionDiagnostic}: ${name} must be a string`);
  }
  return value;
}

export function toRecord(entry: HistoryEntry, at: number): HistoryRecord {
  if (typeof entry.undo !== "function" || typeof entry.redo !== "function") {
    throw new TypeError(`${optionDiagnostic}: entries need undo and redo functions`);
  }
  return {
    undo: entry.undo,
    redo: entry.redo,
    label: readOptionalString(entry.label, "label"),
    coalesceKey: readOptionalString(entry.coalesceKey, "coalesceKey"),
    at,
  };
}

export function toSnapshotEntry<Value>(
  options: HistorySnapshotOptions<Value>,
): HistoryEntry | null {
  if (typeof options.apply !== "function") {
    throw new TypeError(`${optionDiagnostic}: apply must be a function`);
  }
  const isEqual = options.isEqual ?? Object.is;
  if (typeof isEqual !== "function") {
    throw new TypeError(`${optionDiagnostic}: isEqual must be a function`);
  }
  readOptionalString(options.label, "label");
  readOptionalString(options.coalesceKey, "coalesceKey");
  if (isEqual(options.before, options.after)) return null;
  const { before, after, apply } = options;
  const entry: HistoryEntry = {
    undo: () => apply(before),
    redo: () => apply(after),
  };
  return {
    ...entry,
    ...(options.label !== undefined ? { label: options.label } : {}),
    ...(options.coalesceKey !== undefined ? { coalesceKey: options.coalesceKey } : {}),
  };
}

/**
 * Merge `record` into `top` when both carry the same coalescing key and the
 * push arrived inside the coalescing window. The merged step keeps the first
 * undo, adopts the latest redo, and restarts the window.
 */
export function tryCoalesce(
  top: HistoryRecord | undefined,
  record: HistoryRecord,
  coalesceWindow: number,
): boolean {
  if (
    top === undefined ||
    top.coalesceKey === null ||
    top.coalesceKey !== record.coalesceKey ||
    record.at - top.at > coalesceWindow
  ) {
    return false;
  }
  top.redo = record.redo;
  top.label = record.label ?? top.label;
  top.at = record.at;
  return true;
}

/** Fold staged transaction records into one composite step. */
export function foldRecords(
  records: readonly HistoryRecord[],
  label: string | null,
  at: number,
): HistoryRecord {
  const steps = [...records];
  return {
    undo: () => {
      for (let index = steps.length - 1; index >= 0; index -= 1) steps[index]!.undo();
    },
    redo: () => {
      for (const step of steps) step.redo();
    },
    label,
    coalesceKey: null,
    at,
  };
}

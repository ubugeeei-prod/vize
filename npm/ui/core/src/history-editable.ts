import { shallowReadonly, shallowRef } from "vue";

import type { EditableTransaction, EditableTransactionOptions } from "./history-types.ts";

const optionDiagnostic = "VIZE_UI_HISTORY_EDITABLE";

/**
 * Create an in-place editing session that records one history step per edit.
 *
 * `begin` captures the pre-edit value, `update` writes live values without
 * touching history, and `commit` pushes a single snapshot step covering the
 * whole session; `cancel` restores the captured value instead. This keeps
 * per-keystroke writes out of the timeline so field edits undo as one unit.
 */
export function createEditableTransaction<Value>(
  options: EditableTransactionOptions<Value>,
): EditableTransaction<Value> {
  if (typeof options.history?.pushSnapshot !== "function") {
    throw new TypeError(`${optionDiagnostic}: history must be a HistoryController`);
  }
  if (typeof options.read !== "function" || typeof options.write !== "function") {
    throw new TypeError(`${optionDiagnostic}: read and write must be functions`);
  }
  for (const name of ["label", "coalesceKey"] as const) {
    if (options[name] !== undefined && typeof options[name] !== "string") {
      throw new TypeError(`${optionDiagnostic}: ${name} must be a string`);
    }
  }
  const isEqual = options.isEqual ?? Object.is;
  if (typeof isEqual !== "function") {
    throw new TypeError(`${optionDiagnostic}: isEqual must be a function`);
  }

  const isEditing = shallowRef(false);
  let before: Value | undefined;

  const begin = () => {
    if (isEditing.value) return;
    before = options.read();
    isEditing.value = true;
  };

  return Object.freeze({
    isEditing: shallowReadonly(isEditing),
    begin,
    update: (value: Value) => {
      begin();
      options.write(value);
    },
    commit: () => {
      if (!isEditing.value) return false;
      isEditing.value = false;
      const after = options.read();
      const captured = before as Value;
      before = undefined;
      if (isEqual(captured, after)) return false;
      options.history.pushSnapshot({
        before: captured,
        after,
        apply: options.write,
        isEqual,
        ...(options.label !== undefined ? { label: options.label } : {}),
        ...(options.coalesceKey !== undefined ? { coalesceKey: options.coalesceKey } : {}),
      });
      return true;
    },
    cancel: () => {
      if (!isEditing.value) return false;
      isEditing.value = false;
      const captured = before as Value;
      before = undefined;
      options.write(captured);
      return true;
    },
  });
}

import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope } from "vue";

import { createHistory, useHistory } from "./history.ts";
import type { HistoryController } from "./history.ts";

interface Clocked {
  readonly history: HistoryController;
  tick: (milliseconds: number) => void;
}

function clockedHistory(options: Parameters<typeof createHistory>[0] = {}): Clocked {
  let now = 0;
  const history = createHistory({ ...options, now: () => now });
  return {
    history,
    tick: (milliseconds) => {
      now += milliseconds;
    },
  };
}

function trackedValue(history: HistoryController) {
  const state = { value: 0 };
  const set = (next: number, coalesceKey?: string) => {
    const before = state.value;
    state.value = next;
    history.pushSnapshot({
      before,
      after: next,
      apply: (value) => {
        state.value = value;
      },
      label: `Set ${next}`,
      ...(coalesceKey === undefined ? {} : { coalesceKey }),
    });
  };
  return { state, set };
}

test("replays pushed entries in reverse and forward order with reactive depths", () => {
  const history = createHistory();
  const { state, set } = trackedValue(history);
  set(1);
  set(2);

  assert.equal(history.canUndo.value, true);
  assert.equal(history.undoDepth.value, 2);
  assert.equal(history.undoLabel.value, "Set 2");

  assert.equal(history.undo(), true);
  assert.equal(state.value, 1);
  assert.equal(history.redoLabel.value, "Set 2");
  assert.equal(history.undo(), true);
  assert.equal(state.value, 0);
  assert.equal(history.undo(), false);

  assert.equal(history.redo(), true);
  assert.equal(history.redo(), true);
  assert.equal(state.value, 2);
  assert.equal(history.redo(), false);

  history.undo();
  set(9);
  assert.equal(history.canRedo.value, false);
  assert.equal(history.redoDepth.value, 0);
  history.dispose();
});

test("same-key pushes inside the window coalesce into one step", () => {
  const { history, tick } = clockedHistory({ coalesceWindow: 100 });
  const { state, set } = trackedValue(history);
  set(1, "typing");
  tick(50);
  set(12, "typing");
  tick(50);
  set(123, "typing");

  assert.equal(history.undoDepth.value, 1);
  assert.equal(history.undoLabel.value, "Set 123");
  assert.equal(history.undo(), true);
  assert.equal(state.value, 0);
  assert.equal(history.redo(), true);
  assert.equal(state.value, 123);
  history.dispose();
});

test("pushes beyond the window, with other keys, or after undo stay separate", () => {
  const { history, tick } = clockedHistory({ coalesceWindow: 100 });
  const { state, set } = trackedValue(history);
  set(1, "typing");
  tick(101);
  set(2, "typing");
  assert.equal(history.undoDepth.value, 2);

  set(3, "styling");
  assert.equal(history.undoDepth.value, 3);
  set(4);
  set(5);
  assert.equal(history.undoDepth.value, 5);

  history.undo();
  history.redo();
  set(6, "styling");
  assert.equal(history.undoDepth.value, 6);
  assert.equal(state.value, 6);
  history.dispose();
});

test("equal snapshots are dropped", () => {
  const history = createHistory();
  history.pushSnapshot({ before: 5, after: 5, apply: () => undefined });
  history.pushSnapshot({
    before: { id: 1 },
    after: { id: 1 },
    apply: () => undefined,
    isEqual: (left, right) => left.id === right.id,
  });
  assert.equal(history.canUndo.value, false);
  history.dispose();
});

test("transactions fold staged pushes into one labeled atomic step", () => {
  const history = createHistory();
  const { state, set } = trackedValue(history);
  const result = history.transaction("Insert Row", () => {
    set(1);
    set(2);
    set(3);
    return "done";
  });

  assert.equal(result, "done");
  assert.equal(history.undoDepth.value, 1);
  assert.equal(history.undoLabel.value, "Insert Row");
  assert.equal(history.undo(), true);
  assert.equal(state.value, 0);
  assert.equal(history.redo(), true);
  assert.equal(state.value, 3);
  history.dispose();
});

test("rollback and throwing callbacks undo staged entries in reverse", () => {
  const history = createHistory();
  const { state, set } = trackedValue(history);
  set(1);
  assert.throws(
    () =>
      history.transaction(undefined, () => {
        set(2);
        set(3);
        throw new Error("boom");
      }),
    /boom/,
  );
  assert.equal(state.value, 1);
  assert.equal(history.undoDepth.value, 1);

  const frame = history.beginTransaction("Manual");
  set(4);
  frame.rollback();
  assert.equal(state.value, 1);
  assert.equal(history.undoDepth.value, 1);
  history.dispose();
});

test("nested transactions fold into the outer step and enforce LIFO settlement", () => {
  const history = createHistory();
  const { state, set } = trackedValue(history);
  const outer = history.beginTransaction("Outer");
  set(1);
  const inner = history.beginTransaction("Inner");
  set(2);

  assert.throws(() => outer.commit(), /VIZE_UI_HISTORY_TRANSACTION/);
  inner.commit();
  assert.throws(() => inner.commit(), /VIZE_UI_HISTORY_TRANSACTION/);
  assert.equal(history.undoDepth.value, 0);
  outer.commit();
  assert.equal(history.undoDepth.value, 1);
  assert.equal(history.undoLabel.value, "Outer");
  history.undo();
  assert.equal(state.value, 0);

  const rolled = history.beginTransaction();
  const innerRolled = history.beginTransaction();
  set(7);
  innerRolled.rollback();
  assert.equal(state.value, 0);
  rolled.commit();
  assert.equal(history.undoDepth.value, 0);
  history.dispose();
});

test("undo, redo, and clear refuse to run while a transaction is open", () => {
  const history = createHistory();
  const frame = history.beginTransaction();
  assert.throws(() => history.undo(), /VIZE_UI_HISTORY_TRANSACTION/);
  assert.throws(() => history.redo(), /VIZE_UI_HISTORY_TRANSACTION/);
  assert.throws(() => history.clear(), /VIZE_UI_HISTORY_TRANSACTION/);
  frame.commit();
  history.clear();
  history.dispose();
});

test("pushes from reactive mirrors are discarded while restoring", () => {
  const history = createHistory();
  const state = { value: 0 };
  const set = (next: number) => {
    const before = state.value;
    state.value = next;
    history.pushSnapshot({
      before,
      after: next,
      apply: (value) => {
        // A state watcher that records every write back into history.
        assert.equal(history.isRestoring.value, true);
        state.value = value;
        history.push({ undo: () => undefined, redo: () => undefined });
      },
    });
  };

  set(1);
  assert.equal(history.undoDepth.value, 1);
  assert.equal(history.undo(), true);
  assert.equal(state.value, 0);
  assert.equal(history.undoDepth.value, 0);
  assert.equal(history.redoDepth.value, 1);
  assert.equal(history.isRestoring.value, false);
  history.dispose();
});

test("the oldest step drops beyond the limit", () => {
  const history = createHistory({ limit: 2 });
  const { state, set } = trackedValue(history);
  set(1);
  set(2);
  set(3);
  assert.equal(history.undoDepth.value, 2);
  history.undo();
  history.undo();
  assert.equal(history.undo(), false);
  assert.equal(state.value, 1);
  history.dispose();
});

test("a throwing entry surfaces its failure, drops, and keeps the timeline usable", () => {
  const history = createHistory();
  const { state, set } = trackedValue(history);
  set(1);
  history.push({
    undo: () => {
      throw new Error("cannot revert");
    },
    redo: () => undefined,
  });

  assert.throws(() => history.undo(), /cannot revert/);
  assert.equal(history.undoDepth.value, 1);
  assert.equal(history.redoDepth.value, 0);
  assert.equal(history.undo(), true);
  assert.equal(state.value, 0);
  history.dispose();
});

test("stable diagnostics reject malformed entries and options", () => {
  assert.throws(() => createHistory({ limit: 0 }), /VIZE_UI_HISTORY_OPTION/);
  assert.throws(() => createHistory({ coalesceWindow: -1 }), /VIZE_UI_HISTORY_OPTION/);
  const history = createHistory();
  assert.throws(
    () => history.push({ undo: null as never, redo: () => undefined }),
    /VIZE_UI_HISTORY_OPTION/,
  );
  assert.throws(
    () => history.push({ undo: () => undefined, redo: () => undefined, label: 1 as never }),
    /VIZE_UI_HISTORY_OPTION/,
  );
  assert.throws(
    () => history.pushSnapshot({ before: 0, after: 1, apply: null as never }),
    /VIZE_UI_HISTORY_OPTION/,
  );
  assert.throws(() => history.transaction("x", null as never), /VIZE_UI_HISTORY_TRANSACTION/);
  assert.throws(() => history.beginTransaction(1 as never), /VIZE_UI_HISTORY_TRANSACTION/);
  history.dispose();
});

test("dispose and Vue scope teardown clear both timelines and become terminal", () => {
  const scope = effectScope();
  const history = scope.run(() => useHistory())!;
  const { set } = trackedValue(history);
  set(1);
  assert.equal(history.canUndo.value, true);

  scope.stop();
  assert.equal(history.canUndo.value, false);
  assert.equal(history.undoDepth.value, 0);
  assert.throws(() => history.undo(), /VIZE_UI_HISTORY_DISPOSED/);
  assert.throws(
    () => history.push({ undo: () => undefined, redo: () => undefined }),
    /VIZE_UI_HISTORY_DISPOSED/,
  );
  history.dispose();
  assert.throws(() => useHistory(), /VIZE_UI_HISTORY_SETUP/);
});

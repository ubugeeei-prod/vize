import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, shallowRef } from "vue";

import { useHistory } from "./use-history.ts";

void test("records writes and restores them through undo and redo", () => {
  const text = shallowRef("");
  const { canUndo, canRedo, undoCount, redoCount, undo, redo } = useHistory(text);

  assert.equal(canUndo.value, false);
  assert.equal(redo(), false);
  assert.equal(undo(), false);

  text.value = "a";
  text.value = "ab";
  assert.equal(undoCount.value, 2);
  assert.equal(canUndo.value, true);
  assert.equal(canRedo.value, false);

  assert.equal(undo(), true);
  assert.equal(text.value, "a");
  assert.equal(undo(), true);
  assert.equal(text.value, "");
  assert.equal(canUndo.value, false);
  assert.equal(redoCount.value, 2);

  assert.equal(redo(), true);
  assert.equal(text.value, "a");
  assert.equal(redo(), true);
  assert.equal(text.value, "ab");
  assert.equal(canRedo.value, false);
  assert.equal(undoCount.value, 2);
});

void test("keeps counts stable across undo and redo cycles", () => {
  const value = shallowRef(0);
  const { undoCount, redoCount, undo, redo } = useHistory(value);

  value.value = 1;
  value.value = 2;
  for (let cycle = 0; cycle < 3; cycle += 1) {
    undo();
    redo();
  }
  assert.equal(undoCount.value, 2);
  assert.equal(redoCount.value, 0);
  assert.equal(value.value, 2);
});

void test("clears the redo stack when a new change is recorded", () => {
  const value = shallowRef("one");
  const { canRedo, undo } = useHistory(value);

  value.value = "two";
  undo();
  assert.equal(canRedo.value, true);

  value.value = "three";
  assert.equal(canRedo.value, false);
});

void test("ignores writes that do not change the value", () => {
  const value = shallowRef("same");
  const { undoCount } = useHistory(value);

  value.value = "same";
  assert.equal(undoCount.value, 0);
});

void test("drops the oldest entries beyond the capacity", () => {
  const value = shallowRef(0);
  const { canUndo, undoCount, undo } = useHistory(value, { capacity: 2 });

  value.value = 1;
  value.value = 2;
  value.value = 3;
  assert.equal(undoCount.value, 2);

  assert.equal(undo(), true);
  assert.equal(value.value, 2);
  assert.equal(undo(), true);
  assert.equal(value.value, 1);
  assert.equal(canUndo.value, false);
});

void test("rejects capacities that cannot bound a stack", () => {
  const isCapacityError = (error: unknown): boolean =>
    error instanceof RangeError &&
    error.message.startsWith("[VIZE_COMPOSE_HISTORY_INVALID_CAPACITY]");

  for (const capacity of [0, -1, 1.5, Number.NaN]) {
    assert.throws(() => useHistory(shallowRef(0), { capacity }), isCapacityError);
  }
});

void test("groups batched writes into one undo entry and passes the result through", () => {
  const text = shallowRef("");
  const { undoCount, undo, redo, batch } = useHistory(text);

  text.value = "a";
  const result = batch(() => {
    text.value = "ab";
    text.value = "abc";
    return 42;
  });
  assert.equal(result, 42);
  assert.equal(undoCount.value, 2);

  assert.equal(undo(), true);
  assert.equal(text.value, "a");
  assert.equal(redo(), true);
  assert.equal(text.value, "abc");
});

void test("collapses nested batches into the outermost group", () => {
  const value = shallowRef(0);
  const { undoCount, undo, batch } = useHistory(value);

  batch(() => {
    value.value = 1;
    batch(() => {
      value.value = 2;
    });
    value.value = 3;
  });
  assert.equal(undoCount.value, 1);
  undo();
  assert.equal(value.value, 0);
});

void test("commits no entry for a batch that restores the starting value", () => {
  const value = shallowRef("x");
  const { undoCount, batch } = useHistory(value);

  batch(() => {
    value.value = "y";
    value.value = "x";
  });
  assert.equal(undoCount.value, 0);
});

void test("commits the group as one step even when the batch throws", () => {
  const value = shallowRef("before");
  const { undoCount, undo, batch } = useHistory(value);

  assert.throws(
    () =>
      batch(() => {
        value.value = "partial";
        throw new Error("boom");
      }),
    /boom/,
  );
  assert.equal(value.value, "partial");
  assert.equal(undoCount.value, 1);
  undo();
  assert.equal(value.value, "before");
});

void test("forbids stack movement inside a batch", () => {
  const value = shallowRef(0);
  const { undo, redo, clear, batch } = useHistory(value);

  const isBatchError = (error: unknown): boolean =>
    error instanceof Error && error.message.startsWith("[VIZE_COMPOSE_HISTORY_IN_BATCH]");

  assert.throws(() => batch(() => undo()), isBatchError);
  assert.throws(() => batch(() => redo()), isBatchError);
  assert.throws(() => batch(() => clear()), isBatchError);
});

void test("clear drops every entry while keeping the current value", () => {
  const value = shallowRef(0);
  const { canUndo, canRedo, undo, clear } = useHistory(value);

  value.value = 1;
  value.value = 2;
  undo();
  clear();
  assert.equal(canUndo.value, false);
  assert.equal(canRedo.value, false);
  assert.equal(value.value, 1);
});

void test("isolates snapshots from in-place mutation through the clone option", () => {
  interface Draft {
    n: number;
  }
  const draft = shallowRef<Draft>({ n: 1 });
  const { undo, redo } = useHistory(draft, {
    clone: (value) => structuredClone(value),
  });

  const original = draft.value;
  draft.value = { n: 2 };
  original.n = 99;

  assert.equal(undo(), true);
  assert.equal(draft.value.n, 1);
  assert.notEqual(draft.value, original);

  const restored = draft.value;
  assert.equal(redo(), true);
  restored.n = 77;
  assert.equal(undo(), true);
  assert.equal(draft.value.n, 1);
});

void test("stops recording and releases snapshots when the owning scope stops", () => {
  const value = shallowRef(0);
  const scope = effectScope();
  const controls = scope.run(() => useHistory(value));
  assert.ok(controls);

  value.value = 1;
  assert.equal(controls.canUndo.value, true);

  scope.stop();
  assert.equal(controls.canUndo.value, false);
  assert.equal(controls.canRedo.value, false);

  value.value = 2;
  assert.equal(controls.undoCount.value, 0);
  assert.equal(controls.undo(), false);
  assert.equal(value.value, 2);
});

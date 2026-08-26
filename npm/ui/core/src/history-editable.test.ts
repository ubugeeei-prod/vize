import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createEditableTransaction, createHistory } from "./history.ts";

function editor() {
  const history = createHistory();
  const state = { value: "draft" };
  const session = createEditableTransaction({
    history,
    read: () => state.value,
    write: (value) => {
      state.value = value;
    },
    label: "Edit Title",
  });
  return { history, state, session };
}

test("live updates bypass history and commit pushes one step for the session", () => {
  const { history, state, session } = editor();
  session.begin();
  session.begin();
  assert.equal(session.isEditing.value, true);
  session.update("d");
  session.update("draft two");
  assert.equal(state.value, "draft two");
  assert.equal(history.undoDepth.value, 0);

  assert.equal(session.commit(), true);
  assert.equal(session.isEditing.value, false);
  assert.equal(history.undoDepth.value, 1);
  assert.equal(history.undoLabel.value, "Edit Title");

  history.undo();
  assert.equal(state.value, "draft");
  history.redo();
  assert.equal(state.value, "draft two");
  history.dispose();
});

test("update begins a session automatically", () => {
  const { history, state, session } = editor();
  session.update("auto");
  assert.equal(session.isEditing.value, true);
  session.commit();
  history.undo();
  assert.equal(state.value, "draft");
  history.dispose();
});

test("cancel restores the pre-edit value and unchanged commits push nothing", () => {
  const { history, state, session } = editor();
  session.update("scratch");
  assert.equal(session.cancel(), true);
  assert.equal(state.value, "draft");
  assert.equal(session.isEditing.value, false);
  assert.equal(session.cancel(), false);

  session.begin();
  session.update("changed");
  session.update("draft");
  assert.equal(session.commit(), false);
  assert.equal(session.commit(), false);
  assert.equal(history.undoDepth.value, 0);
  history.dispose();
});

test("committed sessions coalesce through a shared key", () => {
  const history = createHistory({ coalesceWindow: 1000 });
  const state = { value: "" };
  const session = createEditableTransaction({
    history,
    read: () => state.value,
    write: (value) => {
      state.value = value;
    },
    coalesceKey: "title",
  });
  session.update("a");
  session.commit();
  session.update("ab");
  session.commit();

  assert.equal(history.undoDepth.value, 1);
  history.undo();
  assert.equal(state.value, "");
  history.redo();
  assert.equal(state.value, "ab");
  history.dispose();
});

test("stable diagnostics reject malformed sessions", () => {
  const history = createHistory();
  assert.throws(
    () =>
      createEditableTransaction({ history: null as never, read: () => 0, write: () => undefined }),
    /VIZE_UI_HISTORY_EDITABLE/,
  );
  assert.throws(
    () => createEditableTransaction({ history, read: null as never, write: () => undefined }),
    /VIZE_UI_HISTORY_EDITABLE/,
  );
  assert.throws(
    () =>
      createEditableTransaction({
        history,
        read: () => 0,
        write: () => undefined,
        label: 1 as never,
      }),
    /VIZE_UI_HISTORY_EDITABLE/,
  );
  history.dispose();
});

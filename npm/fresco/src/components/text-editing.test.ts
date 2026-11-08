import assert from "node:assert/strict";
import { test } from "vitest";

import {
  createTextEditingState,
  deleteTextBackward,
  deleteTextForward,
  nextWordBoundary,
  previousWordBoundary,
  redoTextEdit,
  replaceTextSelection,
  selectAllText,
  selectionBounds,
  setTextSelection,
  undoTextEdit,
} from "./text-editing.js";

test("selection replacement preserves grapheme boundaries and undo history", () => {
  let state = createTextEditingState("a👋🏽c", 1);
  state = setTextSelection(state, 2, true);
  state = replaceTextSelection(state, "b");
  assert.equal(state.value, "abc");
  assert.deepEqual(selectionBounds(state.selection), [2, 2]);

  state = undoTextEdit(state);
  assert.equal(state.value, "a👋🏽c");
  assert.deepEqual(selectionBounds(state.selection), [1, 2]);
  state = redoTextEdit(state);
  assert.equal(state.value, "abc");
});

test("word boundaries treat punctuation and whitespace as independent runs", () => {
  const value = "alpha,  beta";
  assert.equal(previousWordBoundary(value, 12), 8);
  assert.equal(previousWordBoundary(value, 8), 5);
  assert.equal(nextWordBoundary(value, 0), 5);
  assert.equal(nextWordBoundary(value, 5), 8);
  assert.equal(nextWordBoundary(value, 6), 8);
});

test("word deletion and selected deletion remain reversible", () => {
  let state = createTextEditingState("one two", 7);
  state = deleteTextBackward(state, true);
  assert.equal(state.value, "one ");
  state = undoTextEdit(state);
  state = setTextSelection(state, 0);
  state = deleteTextForward(state, true);
  assert.equal(state.value, "two");
  state = selectAllText(state);
  state = deleteTextBackward(state);
  assert.equal(state.value, "");
});

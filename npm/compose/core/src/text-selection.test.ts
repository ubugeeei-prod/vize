import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useTextSelection } from "./text-selection.ts";
import { asDocument, FakeDocument, FakeSelection } from "./testing/fake-dom.ts";

void test("re-reads the in-place mutated selection on selectionchange", () => {
  const document = new FakeDocument();
  const selection = new FakeSelection();
  document.selection = selection;
  const scope = effectScope();
  const state = scope.run(() => useTextSelection({ host: asDocument(document) }));
  assert.ok(state);
  assert.equal(state.text.value, "");
  assert.deepEqual(state.ranges.value, []);

  selection.text = "hello";
  selection.ranges = [{ x: 1, y: 2, width: 3, height: 4 }];
  document.dispatchEvent(new Event("selectionchange"));
  assert.equal(state.text.value, "hello");
  assert.equal(state.ranges.value.length, 1);
  assert.equal(state.rects.value[0]?.right, 4);

  scope.stop();
  selection.text = "ignored";
  document.dispatchEvent(new Event("selectionchange"));
  assert.equal(state.text.value, "hello");
});

void test("server renders expose an empty selection", () => {
  const state = useTextSelection({ host: () => undefined });
  assert.equal(state.selection.value, null);
  assert.equal(state.text.value, "");
  assert.deepEqual(state.rects.value, []);
});

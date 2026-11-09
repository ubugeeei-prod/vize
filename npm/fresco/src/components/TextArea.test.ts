import assert from "node:assert/strict";
import { h, ref } from "vue";
import { test } from "vitest";

import { findNodes, renderTui } from "../testing/index.js";
import { TextArea } from "./TextArea.js";
import {
  lineBoundary,
  moveTextAreaCursorVertically,
  positionToTextAreaPoint,
  textAreaPointToPosition,
} from "./textarea-editing.js";

test("multiline positions remain grapheme-safe at line boundaries", () => {
  const value = "a🙂\nxy";

  assert.deepEqual(positionToTextAreaPoint(value, 4), { line: 1, column: 1 });
  assert.equal(textAreaPointToPosition(value, 0, 99), 2);
  assert.equal(textAreaPointToPosition(value, 1, 1), 4);
  assert.equal(lineBoundary(value, 4, "start"), 3);
  assert.equal(lineBoundary(value, 4, "end"), 5);
  assert.equal(moveTextAreaCursorVertically(value, 2, 1, 2), 5);
});

test("TextArea owns multiline editing, history, selection, validation, and submission", async () => {
  const value = ref("one\nx\nthree");
  const copied: string[] = [];
  const cut: string[] = [];
  const submitted: string[] = [];
  const validation: Array<[boolean, string | undefined]> = [];
  const rendered = renderTui(() =>
    h(TextArea, {
      modelValue: value.value,
      cursorLine: 0,
      cursorColumn: 3,
      rows: 2,
      focused: true,
      lineNumbers: true,
      validate: (candidate: string) => candidate.includes("three") || "missing final line",
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
      onCopy: (selected: string) => copied.push(selected),
      onCut: (selected: string) => cut.push(selected),
      onSubmit: (submittedValue: string) => submitted.push(submittedValue),
      onValidationChange: (valid: boolean, message?: string) => validation.push([valid, message]),
    }),
  );

  await rendered.input.text("🙂");
  assert.equal(value.value, "one🙂\nx\nthree");
  await rendered.input.key({ key: "enter" });
  assert.equal(value.value, "one🙂\n\nx\nthree");
  await rendered.input.key({ key: "enter", ctrl: true });
  assert.deepEqual(submitted, ["one🙂\n\nx\nthree"]);

  await rendered.input.key({ key: "a", ctrl: true });
  await rendered.input.key({ key: "c", ctrl: true });
  await rendered.input.key({ key: "x", ctrl: true });
  assert.deepEqual(copied, ["one🙂\n\nx\nthree"]);
  assert.deepEqual(cut, ["one🙂\n\nx\nthree"]);
  assert.equal(value.value, "");
  assert.deepEqual(validation.at(-1), [false, "missing final line"]);

  await rendered.input.key({ key: "z", ctrl: true });
  assert.equal(value.value, "one🙂\n\nx\nthree");
  assert.deepEqual(validation.at(-1), [true, undefined]);

  const renderedText = findNodes(rendered.root, (node) => node.type === "text")
    .map((node) => node.props.text)
    .filter((text): text is string => text !== undefined);
  assert.ok(renderedText.some((text) => text.includes("three")));
  rendered.unmount();
});

test("TextArea commits IME text and ignores input while disabled", async () => {
  const value = ref("");
  const disabled = ref(false);
  const rendered = renderTui(() =>
    h(TextArea, {
      modelValue: value.value,
      focused: true,
      disabled: disabled.value,
      "onUpdate:modelValue": (next: string) => {
        value.value = next;
      },
    }),
  );

  await rendered.input.compositionStart();
  await rendered.input.compositionUpdate("日本", 2);
  await rendered.input.compositionEnd("日本", 2);
  assert.equal(value.value, "日本");

  disabled.value = true;
  await rendered.input.text("語");
  assert.equal(value.value, "日本");
  rendered.unmount();
});

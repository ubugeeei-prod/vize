import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { isTypingKeystroke, onStartTyping } from "./on-start-typing.ts";
import { asDocument, eventWith, FakeDocument } from "./testing/fake-dom.ts";

const key = (value: string, extra: Record<string, unknown> = {}): Event =>
  eventWith("keydown", {
    key: value,
    code: "",
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    ...extra,
  });

void test("classifies printable keystrokes", () => {
  const typing = (event: Event): boolean => isTypingKeystroke(event as KeyboardEvent);
  assert.equal(typing(key("a")), true);
  assert.equal(typing(key("7")), true);
  assert.equal(typing(key("Enter")), false);
  assert.equal(typing(key(" ")), false);
  assert.equal(typing(key("s", { metaKey: true })), false);
});

void test("fires only when no editable element is focused", () => {
  const document = new FakeDocument();
  const input = document.createElement();
  input.tagName = "INPUT";
  const editor = document.createElement();
  editor.setAttribute("role", "textbox");
  let calls = 0;
  const scope = effectScope();
  scope.run(() => onStartTyping(() => (calls += 1), { host: asDocument(document) }));

  document.dispatchEvent(key("a"));
  assert.equal(calls, 1);
  document.activeElement = input;
  document.dispatchEvent(key("b"));
  document.activeElement = editor;
  document.dispatchEvent(key("c"));
  assert.equal(calls, 1);

  document.activeElement = null;
  scope.stop();
  document.dispatchEvent(key("d"));
  assert.equal(calls, 1);
});

import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { useFocus, useFocusWithin } from "./focus.ts";
import { appendChild, asElement, FakeDocument } from "./testing/fake-dom.ts";

void test("tracks focus events and focuses or blurs on assignment", () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const other = document.createElement();
  const scope = effectScope();
  const focus = scope.run(() => useFocus(asElement(element)));
  assert.ok(focus);
  assert.equal(focus.focused.value, false);

  focus.focused.value = true;
  assert.equal(document.activeElement, element);
  assert.equal(focus.focused.value, true);
  other.focus();
  assert.equal(focus.focused.value, false);
  focus.focused.value = true;
  focus.focused.value = false;
  assert.equal(document.activeElement, null);
  scope.stop();
  element.focus();
  assert.equal(focus.focused.value, false);
});

void test("initialValue focuses once resolved and focusVisible filters", async () => {
  const document = new FakeDocument();
  const element = document.createElement();
  const target = ref<Element | null>(null);
  const scope = effectScope();
  const focus = scope.run(() => useFocus(target, { initialValue: true, focusVisible: true }));
  assert.equal(document.activeElement, null);

  target.value = asElement(element);
  await nextTick();
  assert.equal(document.activeElement, element);
  assert.equal(focus?.focused.value, false);
  element.blur();
  element.matching.add(":focus-visible");
  element.focus();
  assert.equal(focus?.focused.value, true);
  scope.stop();
});

void test("focus-within stays true while focus moves between descendants", () => {
  const document = new FakeDocument();
  const container = document.createElement();
  const first = document.createElement();
  const second = document.createElement();
  const outside = document.createElement();
  appendChild(container, first);
  appendChild(container, second);
  const scope = effectScope();
  const within = scope.run(() => useFocusWithin(asElement(container)));
  assert.ok(within);

  first.focus();
  assert.equal(within.focused.value, true);
  second.focus();
  assert.equal(within.focused.value, true);
  outside.focus();
  assert.equal(within.focused.value, false);
  scope.stop();
});

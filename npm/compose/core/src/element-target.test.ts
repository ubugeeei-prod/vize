import assert from "node:assert/strict";
import { test } from "node:test";
import { ref } from "vue";

import { isElementNode, resolveElement, resolveElements } from "./element-target.ts";
import type { ElementTargetValue } from "./element-target.ts";
import { asElement, FakeDocument } from "./testing/fake-dom.ts";

void test("resolves elements, refs, getters, and component roots", () => {
  const document = new FakeDocument();
  const element = asElement(document.createElement());
  const component = { $el: element };

  assert.equal(resolveElement(element), element);
  assert.equal(resolveElement(ref(element)), element);
  assert.equal(
    resolveElement(() => element),
    element,
  );
  assert.equal(
    resolveElement(() => component as unknown as ElementTargetValue),
    element,
  );
});

void test("unresolved, text-root, and server values resolve to null", () => {
  assert.equal(resolveElement(null), null);
  assert.equal(resolveElement(undefined), null);
  assert.equal(resolveElement(ref(null)), null);
  const fragmentRoot = { $el: { nodeType: 3 } };
  assert.equal(
    resolveElement(() => fragmentRoot as unknown as ElementTargetValue),
    null,
  );
  assert.equal(isElementNode({ nodeType: 1 }), false);
  assert.equal(isElementNode("div"), false);
});

void test("lists drop unresolved entries and duplicates in first-seen order", () => {
  const document = new FakeDocument();
  const first = asElement(document.createElement());
  const second = asElement(document.createElement());

  assert.deepEqual(resolveElements([first, null, second, first, undefined]), [first, second]);
  assert.deepEqual(resolveElements(first), [first]);
  assert.deepEqual(resolveElements(null), []);
});

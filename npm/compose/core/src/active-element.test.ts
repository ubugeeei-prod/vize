import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope } from "vue";

import { useActiveElement } from "./active-element.ts";
import { asDocument, FakeDocument } from "./testing/fake-dom.ts";

void test("follows document focus changes and stops with the scope", () => {
  const document = new FakeDocument();
  const first = document.createElement();
  const second = document.createElement();
  const scope = effectScope();
  const active = scope.run(() => useActiveElement({ host: asDocument(document) }));
  assert.ok(active);
  assert.equal(active.value, null);

  first.focus();
  assert.equal(active.value, first);
  second.focus();
  assert.equal(active.value, second);
  second.blur();
  assert.equal(active.value, null);

  scope.stop();
  first.focus();
  assert.equal(active.value, null);
});

void test("descends into open shadow roots unless disabled", () => {
  const document = new FakeDocument();
  const inner = document.createElement();
  const outer = document.createElement();
  outer.shadowRoot = { activeElement: inner };
  document.activeElement = outer;
  assert.equal(useActiveElement({ host: asDocument(document) }).value, inner);
  assert.equal(useActiveElement({ host: asDocument(document), deep: false }).value, outer);
  assert.equal(useActiveElement({ host: () => null }).value, null);
});

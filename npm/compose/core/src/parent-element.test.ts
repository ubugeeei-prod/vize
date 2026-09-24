import assert from "node:assert/strict";
import { test } from "node:test";
import { nextTick, ref } from "vue";

import { useParentElement } from "./parent-element.ts";
import { appendChild, asElement, FakeDocument } from "./testing/fake-dom.ts";

void test("tracks the parent of the resolved target", async () => {
  const document = new FakeDocument();
  const parent = document.createElement();
  const child = document.createElement();
  const orphan = document.createElement();
  appendChild(parent, child);
  const target = ref<Element | null>(asElement(child));
  const result = useParentElement(target);
  assert.equal(result.value, asElement(parent));

  target.value = asElement(orphan);
  await nextTick();
  assert.equal(result.value, null);
  target.value = null;
  await nextTick();
  assert.equal(result.value, null);
});

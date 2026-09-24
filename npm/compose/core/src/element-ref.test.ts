import assert from "node:assert/strict";
import { test } from "node:test";

import { useElementRef } from "./element-ref.ts";
import { asElement, FakeDocument } from "./testing/fake-dom.ts";

void test("captures elements and component roots without an instance", () => {
  const element = asElement(new FakeDocument().createElement());
  const { element: current, setRef } = useElementRef();
  assert.equal(current.value, null);
  setRef(element);
  assert.equal(current.value, element);
  setRef(null);
  assert.equal(current.value, null);
});

void test("narrows with a guard", () => {
  const fake = new FakeDocument().createElement();
  const isDiv = (value: Element): value is HTMLDivElement => value.tagName === "DIV";
  const { element, setRef } = useElementRef(isDiv);
  setRef(asElement(fake));
  assert.equal(element.value, asElement(fake));
  fake.tagName = "SPAN";
  setRef(asElement(fake));
  assert.equal(element.value, null);
});

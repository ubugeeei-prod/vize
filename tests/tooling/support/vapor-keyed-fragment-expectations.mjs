// Preserve the original full oracle; only input value attributes are refined.
import assert from "node:assert/strict";

export const keyedCases = [
  "component",
  "element",
  "stable",
  "nested-attrs",
  "root-attrs",
  "if-attrs",
  "nested-if-attrs",
];

export function keyedExpectations(originals) {
  const expected = Object.fromEntries(
    keyedCases.map((name) => [name, JSON.parse(originals.get(`${name}.expected.json`))]),
  );
  const current = JSON.parse(originals.get("element.expected-current.json"));
  const original = structuredClone(current);
  for (const [index, label] of ["first", "first", "first", "second"].entries()) {
    const input = original[index].tree[0].children[2];
    assert.equal(input.tag, "input");
    assert.deepEqual(input.attributes, { value: label });
    delete input.attributes.value;
  }
  assert.deepEqual(original, expected.element);
  expected.element = current;
  return expected;
}

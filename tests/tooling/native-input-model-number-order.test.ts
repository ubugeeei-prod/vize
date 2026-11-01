import assert from "node:assert/strict";
import { test } from "node:test";
import { assertWholeNumberArray } from "./support/native-input-model-number-order.ts";

const expected = [
  { id: "first", state: "observed-unqualified", unknown: { left: 1, right: ["raw"] } },
  { id: "second", state: "observed-unqualified", unknown: { left: 2, right: ["whole"] } },
  { id: "third", state: "observed-unqualified", unknown: { left: 3, right: ["packet"] } },
];

test("complete ordered rows retain every unknown field", () => {
  assertWholeNumberArray(structuredClone(expected), expected, "whole replay");
  const changed = structuredClone(expected);
  const first = changed.at(0);
  assert(first);
  first.unknown.right.push("extra");
  assert.throws(() => assertWholeNumberArray(changed, expected, "unknown field change"));
});

test("same-id per-row matches and counts cannot admit reordered current or expected arrays", () => {
  const reordered = [...expected].reverse();
  for (const row of reordered)
    assert.deepEqual(
      row,
      expected.find((value) => value.id === row.id),
    );
  assert.equal(reordered.length, expected.length);
  assert.throws(() => assertWholeNumberArray(reordered, expected, "current order"));
  assert.throws(() => assertWholeNumberArray(expected, reordered, "expected order"));
});

test("nested object field order is retained alongside array order", () => {
  const first = expected.at(0);
  assert(first);
  const reordered = [{ ...first, unknown: { right: ["raw"], left: 1 } }, ...expected.slice(1)];
  assert.deepEqual(reordered, expected);
  assert.throws(() => assertWholeNumberArray(reordered, expected, "nested field order"));
});

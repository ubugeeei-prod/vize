import assert from "node:assert/strict";
import { test } from "vitest";

import {
  adjacentEnabledIndex,
  createTypeaheadBuffer,
  firstEnabledIndex,
  lastEnabledIndex,
  typeaheadIndex,
  visibleCollectionOffset,
} from "./collection-navigation.js";

const items = [
  { label: "Alpha" },
  { label: "Beta", disabled: true },
  { label: "Bravo" },
  { label: "Charlie" },
];

test("collection boundaries and movement skip disabled items", () => {
  assert.equal(firstEnabledIndex(items), 0);
  assert.equal(lastEnabledIndex(items), 3);
  assert.equal(adjacentEnabledIndex(items, 0, 1), 2);
  assert.equal(adjacentEnabledIndex(items, 2, -1), 0);
  assert.equal(adjacentEnabledIndex(items, 3, 1), 0);
  assert.equal(adjacentEnabledIndex([{ disabled: true }], -1, 1), -1);
});

test("typeahead starts after the current item and wraps", () => {
  assert.equal(
    typeaheadIndex(items, 0, "br", (item) => item.label),
    2,
  );
  assert.equal(
    typeaheadIndex(items, 2, "br", (item) => item.label),
    2,
  );
  assert.equal(
    typeaheadIndex(items, 2, "a", (item) => item.label),
    0,
  );
  assert.equal(
    typeaheadIndex(items, 0, "beta", (item) => item.label),
    0,
  );
});

test("typeahead buffers refinements, cycles repeats, and expires after the timeout", () => {
  let now = 0;
  const buffer = createTypeaheadBuffer(700, () => now);

  assert.equal(buffer.push("B"), "b");
  now = 100;
  assert.equal(buffer.push("r"), "br");
  now = 200;
  buffer.reset();
  assert.equal(buffer.push("b"), "b");
  now = 300;
  assert.equal(buffer.push("b"), "b", "repeated characters cycle rather than refine");
  now = 1_100;
  assert.equal(buffer.push("c"), "c");
});

test("bounded collection windows follow the highlighted item", () => {
  assert.equal(visibleCollectionOffset(0, 0, 3, 8), 0);
  assert.equal(visibleCollectionOffset(3, 0, 3, 8), 1);
  assert.equal(visibleCollectionOffset(7, 1, 3, 8), 5);
  assert.equal(visibleCollectionOffset(2, 5, 3, 8), 2);
  assert.equal(visibleCollectionOffset(7, 5, undefined, 8), 0);
});

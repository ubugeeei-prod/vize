import assert from "node:assert/strict";
import { test } from "node:test";

import { dirtyFieldsOf, valuesKeepingDirty } from "./form-dirty.ts";

void test("dirty flags omit clean branches and align array indexes", () => {
  const previous = {
    name: "Ada",
    age: 36,
    address: { city: "Tokyo", zip: "100" },
    tags: ["a", "b"],
  };
  assert.deepEqual(dirtyFieldsOf(previous, previous), {});
  assert.deepEqual(
    dirtyFieldsOf(
      { ...previous, name: "Grace", address: { city: "Osaka", zip: "100" }, tags: ["a", "c", "d"] },
      previous,
    ),
    {
      name: true,
      address: { city: true },
      tags: [undefined, true, true],
    },
  );
  assert.deepEqual(dirtyFieldsOf({ ...previous, nickname: "A" }, previous), { nickname: true });
});

void test("keeping dirty values takes new defaults for clean fields only", () => {
  const previous = { name: "Ada", age: 36, address: { city: "Tokyo", zip: "100" }, tags: ["a"] };
  const current = {
    name: "Grace",
    age: 36,
    address: { city: "Osaka", zip: "100" },
    tags: ["a", "b"],
  };
  const base = { name: "New", age: 1, address: { city: "Kyoto", zip: "200" }, tags: ["z"] };
  assert.deepEqual(valuesKeepingDirty(base, current, previous), {
    name: "Grace",
    age: 1,
    address: { city: "Osaka", zip: "200" },
    tags: ["z", "b"],
  });

  const removed = { age: 36, address: { city: "Tokyo" } };
  assert.deepEqual(valuesKeepingDirty(base, removed, previous), {
    age: 1,
    address: { city: "Kyoto" },
  });
});

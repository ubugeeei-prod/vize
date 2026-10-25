import assert from "node:assert/strict";
import { test } from "node:test";
import { read, remember } from "./hmr-runtime.ts";

void test("shared exported objects have independent tokens for separate component modules", () => {
  const shared = {};
  remember(shared, "first");
  const first = read(shared, "first");
  remember(shared, "second");
  const second = read(shared, "second");
  assert.ok(first);
  assert.ok(second);
  assert.notEqual(first, second);
  assert.equal(read(shared, "first"), first);
  remember(shared, "second");
  assert.notEqual(read(shared, "second"), second);
  assert.equal(read(shared, "first"), first);
});

void test("primitive exports cannot become component state records", () => {
  for (const value of [null, undefined, true, 1, "component", Symbol("component")]) {
    remember(value, "module");
    assert.equal(read(value, "module"), undefined);
  }
});

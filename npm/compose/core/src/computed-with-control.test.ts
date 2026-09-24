import assert from "node:assert/strict";
import { test } from "node:test";
import { effect, reactive, shallowRef } from "vue";

import { computedWithControl } from "./computed-with-control.ts";

void test("recomputes only when the explicit source changes", () => {
  const version = shallowRef(0);
  const ignored = shallowRef(1);
  let runs = 0;
  const value = computedWithControl(version, (previous: number | undefined) => {
    runs += 1;
    return (previous ?? 0) + ignored.value;
  });

  assert.equal(value.value, 1);
  ignored.value = 10;
  assert.equal(value.value, 1);
  assert.equal(runs, 1);
  version.value = 1;
  assert.equal(value.value, 11);
  assert.equal(runs, 2);
});

void test("trigger invalidates manually and notifies readers", () => {
  const external = { count: 0 };
  const value = computedWithControl([], () => external.count);
  const seen: number[] = [];
  effect(() => {
    seen.push(value.value);
  });
  external.count = 5;
  value.trigger();
  assert.deepEqual(seen, [0, 5]);
});

void test("deep sources invalidate on nested changes", () => {
  const state = reactive({ nested: { count: 1 } });
  const value = computedWithControl(
    () => state.nested,
    () => state.nested.count * 2,
    {
      deep: true,
    },
  );
  assert.equal(value.value, 2);
  state.nested.count = 3;
  assert.equal(value.value, 6);
});

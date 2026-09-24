import assert from "node:assert/strict";
import { test } from "node:test";
import { computed, shallowRef } from "vue";

import { useMemoize } from "./use-memoize.ts";

void test("caches per argument list and reloads on demand", () => {
  let calls = 0;
  const square = useMemoize((value: number) => {
    calls += 1;
    return value * value;
  });
  assert.equal(square(3), 9);
  assert.equal(square(3), 9);
  assert.equal(calls, 1);
  assert.equal(square.generateKey(3), "[3]");
  assert.equal(square.load(3), 9);
  assert.equal(calls, 2);
  square.delete(3);
  square(3);
  assert.equal(calls, 3);
  square.clear();
  square(3);
  assert.equal(calls, 4);
});

void test("caches undefined results and custom keys", () => {
  let calls = 0;
  const lookup = useMemoize(
    (user: { id: string }) => {
      calls += 1;
      return user.id === "none" ? undefined : user.id;
    },
    { getKey: (user) => user.id },
  );
  lookup({ id: "none" });
  lookup({ id: "none" });
  assert.equal(calls, 1);
  assert.equal(lookup.cache.has("none"), true);
});

void test("the default cache is reactive", () => {
  let version = 0;
  const read = useMemoize(() => ++version);
  const seen = shallowRef(0);
  const value = computed(() => {
    seen.value += 0;
    return read();
  });
  assert.equal(value.value, 1);
  read.load();
  assert.equal(value.value, 2);
});

void test("accepts a custom cache", () => {
  const cache = new Map<string, number>();
  const memoized = useMemoize((value: number) => value + 1, { cache });
  memoized(1);
  assert.deepEqual([...cache.entries()], [["[1]", 2]]);
});

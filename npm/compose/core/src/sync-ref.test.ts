import assert from "node:assert/strict";
import { test } from "node:test";
import { effectScope, nextTick, ref } from "vue";

import { syncRef } from "./sync-ref.ts";

void test("syncs both ways and starts from the left value", () => {
  const left = ref("a");
  const right = ref("b");
  syncRef(left, right);
  assert.equal(right.value, "a");
  right.value = "c";
  assert.equal(left.value, "c");
  left.value = "d";
  assert.equal(right.value, "d");
});

void test("converts between different types", () => {
  const celsius = ref(20);
  const label = ref("");
  syncRef(celsius, label, {
    transform: { ltr: (value) => `${value}°C`, rtl: (text) => Number.parseFloat(text) },
  });
  assert.equal(label.value, "20°C");
  label.value = "25.5°C";
  assert.equal(celsius.value, 25.5);
  assert.equal(label.value, "25.5°C");
});

void test("one-way directions and immediate: false", () => {
  const left = ref(1);
  const right = ref(2);
  syncRef(left, right, { direction: "rtl" });
  assert.equal(left.value, 2);
  left.value = 5;
  assert.equal(right.value, 2);

  const source = ref(1);
  const target = ref(0);
  syncRef(source, target, { direction: "ltr", immediate: false });
  assert.equal(target.value, 0);
  source.value = 3;
  assert.equal(target.value, 3);
  target.value = 9;
  assert.equal(source.value, 3);
});

void test("the stop handle and the owning scope end syncing", async () => {
  const left = ref(0);
  const right = ref(0);
  const stop = syncRef(left, right, { flush: "pre" });
  left.value = 1;
  await nextTick();
  assert.equal(right.value, 1);
  stop();
  left.value = 2;
  await nextTick();
  assert.equal(right.value, 1);

  const scope = effectScope();
  scope.run(() => syncRef(left, right));
  scope.stop();
  left.value = 3;
  assert.equal(right.value, 2);
});

void test("non-inverse converters cannot ping-pong", () => {
  const left = ref(1);
  const right = ref(0);
  syncRef(left, right, { transform: { ltr: (value) => value + 1, rtl: (value) => value + 1 } });
  assert.deepEqual([left.value, right.value], [1, 2]);
  left.value = 10;
  assert.deepEqual([left.value, right.value], [10, 11]);
});

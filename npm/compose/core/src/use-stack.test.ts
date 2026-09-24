import assert from "node:assert/strict";
import { test } from "node:test";
import { computed, nextTick, watch } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useStack } from "./use-stack.ts";

void test("pops in last-in, first-out order", () => {
  const stack = useStack(["a"]);
  assert.equal(stack.push("b", "c"), 2);
  assert.deepEqual(stack.items.value, ["a", "b", "c"]);
  assert.equal(stack.peek(), "c");
  assert.equal(stack.pop(), "c");
  assert.equal(stack.pop(), "b");
  assert.equal(stack.size.value, 1);
  stack.clear();
  assert.equal(stack.isEmpty.value, true);
  assert.equal(stack.pop(), undefined);
  assert.equal(stack.peek(), undefined);
});

void test("drops the bottom item at capacity by default", () => {
  const stack = useStack([1, 2, 3], { capacity: 2 });
  assert.deepEqual(stack.items.value, [2, 3]);
  stack.push(4, 5);
  assert.deepEqual(stack.items.value, [4, 5]);
});

void test("rejects pushes at capacity with the reject policy", () => {
  const stack = useStack<number>([], { capacity: 2, overflow: "reject" });
  assert.equal(stack.push(1, 2, 3), 2);
  assert.deepEqual(stack.items.value, [1, 2]);
});

void test("drives computed values and watchers", async () => {
  const stack = useStack<string>();
  const top = computed(() => stack.peek());
  const sizes: number[] = [];
  watch(stack.size, (size) => sizes.push(size));

  stack.push("a");
  assert.equal(top.value, "a");
  await nextTick();
  stack.push("b");
  assert.equal(top.value, "b");
  await nextTick();
  stack.pop();
  stack.pop();
  await nextTick();
  assert.deepEqual(sizes, [1, 2, 0]);
});

void test("snapshots are not mutated by later operations", () => {
  const stack = useStack([1]);
  const before = stack.items.value;
  stack.push(2);
  assert.deepEqual(before, [1]);
});

void test("validates the capacity", () => {
  assert.throws(() => useStack([], { capacity: -1 }), /VIZE_COMPOSE_STACK_INVALID_CAPACITY/);
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => {
    const stack = useStack([1, 2]);
    stack.push(3);
    return { items: stack.items, top: stack.peek() };
  });
  assert.equal(state, '{"items":[1,2,3],"top":3}');
});

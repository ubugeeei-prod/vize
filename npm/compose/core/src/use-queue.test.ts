import assert from "node:assert/strict";
import { test } from "node:test";
import { computed, nextTick, watch } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useQueue } from "./use-queue.ts";

void test("dequeues in first-in, first-out order", () => {
  const queue = useQueue([1, 2]);
  assert.equal(queue.enqueue(3, 4), 2);
  assert.deepEqual(queue.items.value, [1, 2, 3, 4]);
  assert.equal(queue.peek(), 1);
  assert.equal(queue.dequeue(), 1);
  assert.equal(queue.dequeue(), 2);
  assert.equal(queue.size.value, 2);
  assert.deepEqual(queue.drain(), [3, 4]);
  assert.equal(queue.isEmpty.value, true);
  assert.equal(queue.dequeue(), undefined);
  assert.equal(queue.peek(), undefined);
});

void test("drops the oldest items at capacity by default", () => {
  const queue = useQueue([1, 2, 3], { capacity: 2 });
  assert.deepEqual(queue.items.value, [2, 3]);
  assert.equal(queue.enqueue(4), 1);
  assert.deepEqual(queue.items.value, [3, 4]);
});

void test("rejects new items at capacity with the reject policy", () => {
  const queue = useQueue<string>([], { capacity: 2, overflow: "reject" });
  assert.equal(queue.enqueue("a", "b", "c"), 2);
  assert.deepEqual(queue.items.value, ["a", "b"]);
  queue.dequeue();
  assert.equal(queue.enqueue("d"), 1);
  assert.deepEqual(queue.items.value, ["b", "d"]);
});

void test("keeps order across buffer compaction", () => {
  const queue = useQueue<number>();
  for (let index = 0; index < 200; index += 1) queue.enqueue(index);
  for (let index = 0; index < 150; index += 1) assert.equal(queue.dequeue(), index);
  queue.enqueue(200);
  assert.equal(queue.size.value, 51);
  assert.equal(queue.peek(), 150);
  assert.deepEqual(queue.items.value.slice(-2), [199, 200]);
});

void test("drives computed values and watchers", async () => {
  const queue = useQueue<string>();
  const head = computed(() => queue.peek());
  const sizes: number[] = [];
  watch(queue.size, (size) => sizes.push(size));

  queue.enqueue("a");
  assert.equal(head.value, "a");
  await nextTick();
  queue.enqueue("b");
  queue.dequeue();
  assert.equal(head.value, "b");
  await nextTick();
  queue.clear();
  await nextTick();
  assert.deepEqual(sizes, [1, 0]);
});

void test("rejecting everything does not notify", async () => {
  const queue = useQueue([1], { capacity: 1, overflow: "reject" });
  let runs = 0;
  watch(queue.items, () => {
    runs += 1;
  });
  assert.equal(queue.enqueue(2), 0);
  queue.clear();
  queue.clear();
  await nextTick();
  assert.equal(runs, 1);
});

void test("validates the capacity", () => {
  assert.throws(() => useQueue([], { capacity: 0 }), /VIZE_COMPOSE_QUEUE_INVALID_CAPACITY/);
  assert.throws(() => useQueue([], { capacity: 1.5 }), /VIZE_COMPOSE_QUEUE_INVALID_CAPACITY/);
  assert.doesNotThrow(() => useQueue([], { capacity: Number.POSITIVE_INFINITY }));
});

void test("server rendering is deterministic", async () => {
  const state = await renderComposableOnServer(() => {
    const queue = useQueue(["a", "b"]);
    queue.enqueue("c");
    queue.dequeue();
    return { items: queue.items, size: queue.size };
  });
  assert.equal(state, '{"items":["b","c"],"size":2}');
});

import assert from "node:assert/strict";
import { test } from "node:test";

import { useAsyncQueue } from "./use-async-queue.ts";

void test("runs tasks in order, chaining results", async () => {
  let finished = 0;
  const queue = useAsyncQueue(
    [async () => 2, async (previous) => previous * 10, (previous) => `${previous}!`],
    { immediate: false, onFinished: () => (finished += 1) },
  );
  assert.equal(queue.activeIndex.value, -1);
  const records = await queue.run();
  assert.deepEqual(
    records.map((record) => [record.state, record.data]),
    [
      ["fulfilled", 2],
      ["fulfilled", 20],
      ["fulfilled", "20!"],
    ],
  );
  assert.equal(queue.isFinished.value, true);
  assert.equal(finished, 1);
});

void test("interrupts on failure unless told otherwise", async () => {
  const errors: number[] = [];
  const interrupted = useAsyncQueue(
    [
      async () => {
        throw new Error("boom");
      },
      async () => "never",
    ],
    { immediate: false, onError: (_error, index) => errors.push(index) },
  );
  const records = await interrupted.run();
  assert.deepEqual(
    records.map((record) => record.state),
    ["rejected", "aborted"],
  );
  assert.deepEqual(errors, [0]);

  const tolerant = useAsyncQueue(
    [
      async () => {
        throw new Error("boom");
      },
      async (previous) => previous ?? "recovered",
    ],
    { immediate: false, interrupt: false },
  );
  const tolerantRecords = await tolerant.run();
  assert.deepEqual(
    tolerantRecords.map((record) => record.state),
    ["rejected", "fulfilled"],
  );
  assert.equal(tolerantRecords[1].data, "recovered");
});

void test("abort cancels the running task and marks the rest aborted", async () => {
  let signal: AbortSignal | undefined;
  const queue = useAsyncQueue(
    [
      (_previous, context) => {
        signal = context.signal;
        return new Promise<number>(() => undefined);
      },
      async () => 1,
    ],
    { immediate: false },
  );
  void queue.run();
  await Promise.resolve();
  queue.abort("stop");
  assert.equal(signal?.aborted, true);
  assert.deepEqual(
    queue.results.value.map((record) => record.state),
    ["aborted", "aborted"],
  );
  assert.equal(queue.isFinished.value, true);
});

void test("an external signal aborts the queue", async () => {
  const controller = new AbortController();
  controller.abort();
  const queue = useAsyncQueue([async () => 1], { signal: controller.signal, immediate: false });
  const records = await queue.run();
  assert.deepEqual(
    records.map((record) => record.state),
    ["aborted"],
  );
});

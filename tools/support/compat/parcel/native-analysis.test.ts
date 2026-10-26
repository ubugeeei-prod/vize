import assert from "node:assert/strict";
import { test } from "node:test";
import { summarizeNativeEvents } from "./native-analysis.mjs";

const event = (callId: string, duration = 1, tid = 1) => ({
  type: "trace",
  name: "napi:compileSfc",
  pid: 7,
  tid,
  duration,
  ts: 12,
  args: { name: "组件.vue", callId },
});

void test("duplicate reporter delivery retains raw events and counts a producer call once", () => {
  const first = event("first");
  const input = [first, structuredClone(first), event("second", 2)];
  const original = JSON.stringify(input);
  const result = summarizeNativeEvents(input);
  assert.equal(JSON.stringify(input), original);
  assert.equal(result.rawEvents, 3);
  assert.equal(result.calls, 2);
  assert.equal(result.deliveryCopies, 1);
  assert.equal(result.operations[0].summedInclusiveMs, 3);
});

void test("different native worker threads retain separate operation costs", () => {
  const result = summarizeNativeEvents([event("first", 2, 1), event("second", 3, 2)]);
  assert.deepEqual(
    result.operations.map(({ tid, calls, summedInclusiveMs }) => ({
      tid,
      calls,
      summedInclusiveMs,
    })),
    [
      { tid: 1, calls: 1, summedInclusiveMs: 2 },
      { tid: 2, calls: 1, summedInclusiveMs: 3 },
    ],
  );
});

void test("changed duplicate events and missing producer identity refuse derived counts", () => {
  assert.throws(
    () => summarizeNativeEvents([event("same", 1), event("same", 2)]),
    /delivery copies/,
  );
  assert.throws(() => summarizeNativeEvents([{ ...event("first"), args: {} }]));
  assert.throws(() => summarizeNativeEvents([event("negative", -1)]));
});

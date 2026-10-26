import assert from "node:assert/strict";

/** Keep raw delivery copies, derive operation counts from producer call identity. */
export function summarizeNativeEvents(events) {
  const calls = new Map();
  for (const event of events) {
    assert.equal(event.type, "trace");
    assert.ok(event.name.startsWith("napi:"));
    assert.equal(typeof event.args?.callId, "string");
    assert.ok(event.args.callId.length);
    assert.ok(Number.isFinite(event.duration) && event.duration >= 0);
    const bytes = JSON.stringify(event);
    const previous = calls.get(event.args.callId);
    if (previous)
      assert.equal(previous.bytes, bytes, "one native call cannot change between delivery copies");
    else calls.set(event.args.callId, { bytes, event });
  }
  const operations = new Map();
  for (const { event } of calls.values()) {
    const key = JSON.stringify([event.name, event.pid, event.tid]);
    const row = operations.get(key) ?? {
      name: event.name,
      pid: event.pid,
      tid: event.tid,
      calls: 0,
      summedInclusiveMs: 0,
    };
    row.calls++;
    row.summedInclusiveMs += event.duration;
    operations.set(key, row);
  }
  return {
    rawEvents: events.length,
    calls: calls.size,
    deliveryCopies: events.length - calls.size,
    note: "Per-call inclusive NAPI wall durations grouped by process/thread. Sums describe call time, not host elapsed time or internal Rust phases. All original delivery events are retained.",
    operations: [...operations.values()],
  };
}

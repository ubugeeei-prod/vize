import assert from "node:assert/strict";

interface NativeTraceEvent {
  type: string;
  name: string;
  pid: number;
  tid: number;
  duration: number;
  args?: { callId?: string };
  [field: string]: unknown;
}

interface NativeOperation {
  name: string;
  pid: number;
  tid: number;
  calls: number;
  summedInclusiveMs: number;
}

/** Keep raw delivery copies, derive operation counts from producer call identity. */
export function summarizeNativeEvents(events: NativeTraceEvent[]) {
  const calls = new Map<string, { bytes: string; event: NativeTraceEvent }>();
  for (const event of events) {
    assert.equal(event.type, "trace");
    assert.ok(event.name.startsWith("napi:"));
    const callId = event.args?.callId;
    assert.equal(typeof callId, "string");
    if (typeof callId !== "string") assert.fail("native calls require a string identity");
    assert.ok(callId.length);
    assert.ok(Number.isFinite(event.duration) && event.duration >= 0);
    const bytes = JSON.stringify(event);
    const previous = calls.get(callId);
    if (previous)
      assert.equal(previous.bytes, bytes, "one native call cannot change between delivery copies");
    else calls.set(callId, { bytes, event });
  }
  const operations = new Map<string, NativeOperation>();
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

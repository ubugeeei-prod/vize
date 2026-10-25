import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assertSnapshotDrainBeforeOwnerShutdown,
  timedFrames,
} from "./support/native-runtime-selection/lsp-api-snapshot-drain.mjs";
import { decodeNativeFrames } from "./support/native-runtime-selection/lsp-strace.mjs";
const row = (packet, start, end = start + 1) => ({
  packet,
  interval: { startNanoseconds: String(start), endNanoseconds: String(end) },
});
const input = [
  row({ jsonrpc: "2.0", id: 1, method: "createSnapshot", params: {} }, 1),
  row({ jsonrpc: "2.0", id: 2, method: "release", params: { snapshot: 3, handle: "3" } }, 5),
];
const output = [
  row({ jsonrpc: "2.0", id: 1, result: { snapshot: 3, projects: [] } }, 3),
  row({ jsonrpc: "2.0", id: 2, result: true }, 7),
];
const shutdown = row({ jsonrpc: "2.0", id: 4, method: "shutdown" }, 10);
void test("actual head release requires the whole same-ID ACK before native shutdown", () => {
  assert.equal(assertSnapshotDrainBeforeOwnerShutdown(input, output, shutdown).length, 1);
  assert.throws(
    () => assertSnapshotDrainBeforeOwnerShutdown(input.slice(0, 1), output, shutdown),
    /missing or extra release/u,
  );
  for (const mutate of [
    (rows) => {
      rows[1].interval.endNanoseconds = "10";
    },
    (rows) => {
      rows[1].packet.result = {};
    },
    (rows) => {
      rows[1].packet.result = null;
    },
    (rows) => {
      rows[1].packet.result = false;
    },
    (rows) => {
      rows[1].packet.id = 100;
    },
    (rows) => {
      rows[1].packet.error = { code: -1 };
    },
  ]) {
    const bad = structuredClone(output);
    mutate(bad);
    assert.throws(() => assertSnapshotDrainBeforeOwnerShutdown(input, bad, shutdown));
  }
  assert.throws(() =>
    assertSnapshotDrainBeforeOwnerShutdown([...input, input[1]], output, shutdown),
  );
  for (const handle of [0, -1, "3", 3.5, null]) {
    const malformed = structuredClone(output);
    malformed[0].packet.result.snapshot = handle;
    assert.throws(
      () => assertSnapshotDrainBeforeOwnerShutdown(input, malformed, shutdown),
      /exact known beta numeric snapshot handle/u,
    );
  }
  const wrongType = structuredClone(input);
  wrongType[1].packet.params.snapshot = "3";
  assert.throws(
    () => assertSnapshotDrainBeforeOwnerShutdown(wrongType, output, shutdown),
    /whole exact beta release/u,
  );
});
void test("framing retains conservative complete syscall interval across partial bytes", () => {
  const body = Buffer.from(JSON.stringify({ jsonrpc: "2.0", id: 1, result: null }));
  const bytes = Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
  const events = [
    {
      operation: "write",
      fd: 7,
      bytes: bytes.subarray(0, 8),
      interval: { startNanoseconds: "1", endNanoseconds: "2" },
    },
    {
      operation: "write",
      fd: 7,
      bytes: bytes.subarray(8),
      interval: { startNanoseconds: "3", endNanoseconds: "4" },
    },
  ];
  const frames = timedFrames(events, "write", 7, decodeNativeFrames);
  assert.deepEqual(frames[0].interval, { startNanoseconds: "1", endNanoseconds: "4" });
  assert.deepEqual(frames[0].packet, { jsonrpc: "2.0", id: 1, result: null });
});

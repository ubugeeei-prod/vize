import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { decodeFrames } from "../../../differential/lsp-wire.ts";
import type { JsonRpcMessage } from "../../../differential/lsp-types.ts";

export const MAX_WIRE_BYTES = 16 * 1024 * 1024;
export type Chunk = { start: number; end: number; ns: string };
export type TimedMessage = { message: JsonRpcMessage; start: number; end: number; ns: string };
export const hash = (bytes: string | Uint8Array): string =>
  createHash("sha256").update(bytes).digest("hex");

// A message's clock is the Node data chunk that completes its actual frame.
// It is transport availability, not a server-internal or individual-byte clock.
export function timedMessages(bytes: Buffer, chunks: Chunk[]): TimedMessage[] {
  assert(bytes.length <= MAX_WIRE_BYTES, "raw wire quota exceeded");
  let offset = 0;
  let last = -1n;
  const result: TimedMessage[] = [];
  let pending = Buffer.alloc(0);
  let consumed = 0;
  for (const chunk of chunks) {
    assert.equal(chunk.start, offset, "chunk coverage must be contiguous");
    assert(Number.isSafeInteger(chunk.end) && chunk.end > offset && chunk.end <= bytes.length);
    assert.match(chunk.ns, /^\d+$/);
    assert(BigInt(chunk.ns) >= last, "monotonic chunk clocks required");
    last = BigInt(chunk.ns);
    pending = Buffer.concat([pending, bytes.subarray(offset, chunk.end)]);
    while (pending.length > 0) {
      const separator = pending.indexOf("\r\n\r\n");
      if (separator < 0) break;
      const lengths = [
        ...pending
          .subarray(0, separator)
          .toString("ascii")
          .matchAll(/^Content-Length: (\d+)$/gim),
      ];
      assert.equal(lengths.length, 1);
      const length = Number(lengths[0][1]);
      assert(Number.isSafeInteger(length) && length <= MAX_WIRE_BYTES);
      const end = separator + 4 + length;
      if (end > pending.length) break;
      const { messages } = decodeFrames(pending.subarray(0, end));
      assert.equal(messages.length, 1);
      result.push({ message: messages[0], start: consumed, end: consumed + end, ns: chunk.ns });
      consumed += end;
      pending = pending.subarray(end);
    }
    offset = chunk.end;
  }
  assert.equal(offset, bytes.length, "all actual bytes must have clocks");
  assert.equal(pending.length, 0, "truncated frame is not an accepted sample");
  return result;
}

export function object(value: unknown): Record<string, unknown> {
  assert(value != null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}

export function elapsedMs(start: string, end: string): number {
  const delta = BigInt(end) - BigInt(start);
  assert(delta >= 0n, "a response cannot precede its request");
  return Number(delta) / 1e6;
}

export function statistics(values: number[], failures = 0, startup = false) {
  assert(values.every((value) => Number.isFinite(value) && value >= 0));
  assert(Number.isSafeInteger(failures) && failures >= 0);
  const sorted = [...values].sort((a, b) => a - b);
  const rank = (quantile: number) => sorted[Math.ceil(quantile * sorted.length) - 1] ?? null;
  return {
    n: values.length,
    failures,
    p50Ms: rank(0.5),
    p95Ms: startup ? null : rank(0.95),
    p95Reason: startup
      ? "only three fresh processes; startup tail is insufficiently sampled"
      : null,
    maxMs: sorted.at(-1) ?? null,
  };
}

export function responseFor(request: TimedMessage, server: TimedMessage[]): TimedMessage {
  assert(typeof request.message.id === "number", "numeric actual client request ID required");
  const found = server.filter(
    (row) => row.message.id === request.message.id && row.message.method == null,
  );
  assert.equal(found.length, 1, "each request requires exactly one complete response");
  const response = found[0];
  elapsedMs(request.ns, response.ns);
  assert(
    !Object.hasOwn(response.message, "error"),
    "failed RPC retains failure, not a fast sample",
  );
  assert(Object.hasOwn(response.message, "result"), "omitted result is not success");
  return response;
}

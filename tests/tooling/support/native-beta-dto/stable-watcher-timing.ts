import assert from "node:assert/strict";
import type { NativeTimedObservation } from "./native-io.ts";
type Packet = Record<string, unknown>;
const object = (value: unknown): Packet => {
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    "whole native packet required",
  );
  return value as Packet;
};
export interface WatcherTimedIo extends NativeTimedObservation {
  op: string;
  fd: number;
  data: Uint8Array;
}
export const micros = (value: string) => {
  assert.match(value, /^[0-9]+\.[0-9]{6}$/u, "whole syscall timestamp required");
  return BigInt(value.replace(".", ""));
};
export function timedPackets(io: readonly WatcherTimedIo[], op: string, fd: number) {
  const chunks = io
    .filter((row) => row.op === op && row.fd === fd)
    .sort((a, b) => a.at.localeCompare(b.at) || a.tid - b.tid || a.line - b.line);
  const raw = Buffer.concat(chunks.map((row) => Buffer.from(row.data)));
  let offset = 0;
  const packets: Array<{ packet: Packet; at: string; parts: NativeTimedObservation[] }> = [];
  while (offset < raw.length) {
    const marker = raw.indexOf("\r\n\r\n", offset);
    assert.ok(marker >= 0, "complete timed native header required");
    const header = raw.subarray(offset, marker).toString("ascii");
    assert.match(header, /^Content-Length: [0-9]+$/u);
    const end = marker + 4 + Number(header.slice("Content-Length: ".length));
    assert.ok(end <= raw.length, "complete timed native body required");
    let bytes = 0;
    const parts = chunks.filter((row) => {
      const begin = bytes;
      bytes += row.data.length;
      return begin < end && bytes > offset;
    });
    packets.push({
      packet: object(JSON.parse(raw.subarray(marker + 4, end).toString("utf8"))),
      at: parts.at(-1)!.at,
      parts: parts.map(({ at, tid, line, startedLine, completion }) => ({
        at,
        tid,
        line,
        startedLine,
        completion,
      })),
    });
    offset = end;
  }
  return packets;
}

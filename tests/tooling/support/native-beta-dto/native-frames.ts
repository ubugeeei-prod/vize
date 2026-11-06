import assert from "node:assert/strict";
import type { Packet } from "./audit-wire-types.ts";

/** Decode one whole native LSP byte stream without accepting partial frames. */
export function nativeFrames(data: Buffer): Packet[] {
  let offset = 0;
  const packets: Packet[] = [];
  while (offset < data.length) {
    const marker = data.indexOf("\r\n\r\n", offset);
    assert(marker >= 0, "Unclosed native header");
    const header = data.subarray(offset, marker).toString("ascii");
    const lengths = [...header.matchAll(/^Content-Length: ([0-9]+)$/gim)];
    assert.equal(lengths.length, 1, header);
    const size = Number(lengths[0]![1]);
    assert(size <= 16 * 1024 * 1024);
    const end = marker + 4 + size;
    assert(end <= data.length, "Truncated native body");
    const packet = JSON.parse(data.subarray(marker + 4, end).toString("utf8")) as Packet;
    assert.equal(packet.jsonrpc, "2.0");
    packets.push(packet);
    offset = end;
  }
  assert.equal(offset, data.length);
  return packets;
}

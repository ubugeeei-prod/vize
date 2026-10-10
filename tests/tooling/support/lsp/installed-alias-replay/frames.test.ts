import assert from "node:assert/strict";
import { test } from "node:test";
import { FrameDecoder } from "./frames.ts";

const frame = (packet: unknown) => {
  const body = Buffer.from(JSON.stringify(packet));
  return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]);
};
test("fragmented Unicode frames preserve whole packets and adjacent notifications", () => {
  const packets = [
    { jsonrpc: "2.0", id: 4, result: { text: "😀日本" } },
    {
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: { version: 2, diagnostics: [] },
    },
  ];
  const bytes = Buffer.concat(packets.map(frame));
  for (const stride of [1, 3, 17, bytes.length]) {
    const decoder = new FrameDecoder();
    const observed: unknown[] = [];
    for (let offset = 0; offset < bytes.length; offset += stride)
      decoder.push(bytes.subarray(offset, offset + stride), (packet) => observed.push(packet));
    assert.deepEqual(observed, packets);
    assert.equal(decoder.remaining, 0);
  }
});
test("malformed framing or UTF8 never silently drops a packet", () => {
  for (const bytes of [
    Buffer.from("Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}"),
    Buffer.from("Content-Length: x\r\n\r\n{}"),
    Buffer.from("Content-Length: 9007199254740992\r\n\r\n{}"),
    Buffer.from("Content-Length: 2\r\n\r\n{}"),
    Buffer.concat([Buffer.from("Content-Length: 1\r\n\r\n"), Buffer.from([0xff])]),
  ]) {
    const decoder = new FrameDecoder();
    assert.throws(() => decoder.push(bytes, () => assert.fail("invalid packet reached a request")));
  }
  const decoder = new FrameDecoder();
  const incomplete = frame({ jsonrpc: "2.0", id: 1, result: [] }).subarray(0, -1);
  decoder.push(incomplete, () => assert.fail("incomplete packet was admitted"));
  assert.equal(decoder.remaining, incomplete.length);
});

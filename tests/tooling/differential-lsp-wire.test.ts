import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { decodeFrames, frameMessage, LspWire } from "../differential/lsp-wire.ts";

void test("LSP wire client captures split real subprocess frames and a clean shutdown", async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-wire-law-"));
  const script = path.join(dir, "fake.ts");
  fs.writeFileSync(
    script,
    `
    let buffer = Buffer.alloc(0);
    process.stdin.on('data', chunk => {
      buffer = Buffer.concat([buffer, chunk]);
      while (true) {
        const end = buffer.indexOf('\\r\\n\\r\\n');
        if (end < 0) return;
        const length = Number(buffer.subarray(0, end).toString().match(/Content-Length: (\\d+)/)[1]);
        if (buffer.length < end + 4 + length) return;
        const message = JSON.parse(buffer.subarray(end + 4, end + 4 + length));
        buffer = buffer.subarray(end + 4 + length);
        if (message.method === 'exit') { process.exit(0); }
        if (message.id) {
          const body = JSON.stringify({jsonrpc: '2.0', id: message.id, result: message.method === 'shutdown' ? null : '😀あ'});
          const frame = Buffer.from('Content-Length: ' + Buffer.byteLength(body) + '\\r\\n\\r\\n' + body);
          process.stdout.write(frame.subarray(0, 7));
          process.stdout.write(frame.subarray(7));
        }
      }
    });
  `,
  );
  const wire = new LspWire(process.execPath, [script], dir);
  try {
    assert.deepEqual(await wire.request("unicode", { source: "あ" }), {
      jsonrpc: "2.0",
      id: 1,
      result: "😀あ",
    });
    await wire.finish();
    const observation = wire.observation();
    assert.equal(observation.exitStatus, 0);
    assert.equal(
      decodeFrames(Buffer.from(observation.serverWireBase64, "base64")).messages.length,
      2,
    );
  } finally {
    await wire.stop();
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

void test("LSP framed observations retain Unicode bytes, complete frames and JSON envelopes", () => {
  const message = { jsonrpc: "2.0", id: 1, result: "😀\r\nあ" };
  const frame = frameMessage(message);
  assert.deepEqual(decodeFrames(frame).messages, [message]);
  for (const cut of [1, 10, frame.length - 1]) {
    assert.throws(() => decodeFrames(frame.subarray(0, cut)), /truncated/);
    assert.equal(decodeFrames(frame.subarray(0, cut), false).consumed, 0);
  }
  assert.throws(
    () => decodeFrames(Buffer.from("Content-Length: 1\r\nContent-Length: 1\r\n\r\nx")),
    /one Content-Length/,
  );
  assert.throws(() => decodeFrames(frameMessage({ id: 1, result: null })), /JSON-RPC/);
  const invalidUtf8 = Buffer.concat([
    Buffer.from("Content-Length: 3\r\n\r\n"),
    Buffer.from([34, 255, 34]),
  ]);
  assert.throws(() => decodeFrames(invalidUtf8), /encoded data/);
});

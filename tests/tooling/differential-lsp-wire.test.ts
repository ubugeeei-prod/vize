import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { decodeFrames, frameMessage, LspWire } from "../differential/lsp-wire.ts";

void test("LSP wire client captures split real subprocess frames and a clean shutdown", async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-wire-law-"));
  const script = path.join(dir, "fake.ts");
  fs.writeFileSync(script, readToolingProgram("commands/differential-lsp-wire/respond-to-lsp.js"));
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

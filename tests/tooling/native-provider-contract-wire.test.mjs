import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { completeFrames, takeFrame } from "./support/native-provider-contract/wire.mjs";
import { bindInitialize } from "./support/native-provider-contract/observe.mjs";
const frame = (text) =>
  Buffer.concat([
    Buffer.from(`Content-Length: ${Buffer.byteLength(text)}\r\n\r\n`),
    Buffer.from(text),
  ]);
test("direct SDK framing retains unknown fields, Unicode and every complete frame", () => {
  const packet = { jsonrpc: "2.0", id: 7, result: { unknown: ["🧭日本語", null, { x: true }] } };
  const raw = frame(JSON.stringify(packet));
  assert.deepEqual(
    completeFrames(Buffer.concat([raw, raw])).map((entry) => entry.packet),
    [packet, packet],
  );
  assert.equal(takeFrame(raw.subarray(0, raw.length - 1)), null);
  assert.throws(() => completeFrames(raw.subarray(0, raw.length - 1)), /complete original/u);
});
test("direct SDK framing refuses duplicate lengths, invalid UTF8, oversized and trailing bytes", () => {
  for (const raw of [
    Buffer.from("Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}"),
    Buffer.from(`Content-Length: ${16 * 1024 * 1024 + 1}\r\n\r\n`),
    Buffer.from([...Buffer.from("Content-Length: 1\r\n\r\n"), 255]),
    Buffer.concat([frame("{}"), Buffer.from("extra")]),
  ])
    assert.throws(() => completeFrames(raw));
});
test("production initializer relocation preserves the entire original capability authority", () => {
  const fixture = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "../_fixtures/runtime/native-beta-eof-contract/initialize-request.json",
  );
  const original = JSON.parse(fs.readFileSync(fixture, "utf8"));
  original.params.workspaceFolders[0].opaqueFutureField = { keep: [1, null, "🧭"] };
  original.opaqueFutureTopLevel = { keep: true };
  const bound = bindInitialize(original, "/tmp/owned 🧭 project", 42);
  assert.deepEqual(
    bound.params.workspaceFolders[0].opaqueFutureField,
    original.params.workspaceFolders[0].opaqueFutureField,
  );
  assert.deepEqual(bound.opaqueFutureTopLevel, original.opaqueFutureTopLevel);
  assert.deepEqual(bound.params.capabilities, original.params.capabilities);
  assert.deepEqual(bound.params.initializationOptions, original.params.initializationOptions);
  assert.equal(bound.params.rootUri, "file:///tmp/owned%20%F0%9F%A7%AD%20project");
  assert.equal(bound.params.processId, 42);
  assert.notEqual(bound.params.rootPath, original.params.rootPath);
});

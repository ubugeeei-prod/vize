import assert from "node:assert/strict";
import fs from "node:fs";
import { sha256 } from "./custody.mjs";

export function inputObservation(filename, text) {
  const before = { text, bytes: Buffer.byteLength(text), sha256: sha256(Buffer.from(text)) };
  try {
    const bytes = fs.readFileSync(filename);
    return {
      ...before,
      afterText: bytes.toString("utf8"),
      afterBytes: bytes.length,
      afterBase64: bytes.toString("base64"),
      afterSha256: sha256(bytes),
      readError: null,
    };
  } catch (error) {
    return {
      ...before,
      afterText: null,
      afterBytes: null,
      afterBase64: null,
      afterSha256: null,
      readError: { name: error.name, message: error.message },
    };
  }
}

export function verifyPacket(packet, stdout, stderr) {
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert.equal(packet.status, packet.expectedStatus);
  assert.equal(stderr.length, 0);
  packet.actual = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(stdout));
  assert.deepEqual(packet.actual, packet.expected);
  for (const input of [packet.input, packet.configuration].filter(Boolean)) {
    assert.equal(input.readError, null, "original source/configuration disappeared");
    assert.equal(input.afterText, input.text, "original source/configuration changed");
    assert.equal(input.afterSha256, input.sha256);
  }
}

// Deliberately unavailable test peer: no successful native answers are emitted.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readSync, writeFileSync, writeSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const [captureDirectory, ...argv] = process.argv.slice(2);
assert(captureDirectory);
assert(argv[0] === "--api" || argv[0] === "--lsp");

// Read the complete real request before sending a denial or closing the pipe.
const headerBytes: number[] = [];
const one = Buffer.alloc(1);
while (!Buffer.from(headerBytes).subarray(-4).equals(Buffer.from("\r\n\r\n"))) {
  assert.equal(readSync(0, one, 0, 1, null), 1, "complete bootstrap header required");
  const byte = one.at(0);
  assert(byte !== undefined);
  headerBytes.push(byte);
}
const header = Buffer.from(headerBytes);
const lengths = header
  .toString("utf8")
  .split("\r\n")
  .filter((line) => /^Content-Length: /i.test(line));
assert.equal(lengths.length, 1);
const lengthHeader = lengths.at(0);
assert(lengthHeader);
const length = Number(lengthHeader.slice("Content-Length: ".length));
assert(Number.isSafeInteger(length) && length > 0);
const payload = Buffer.alloc(length);
for (let offset = 0; offset < length;) {
  const received = readSync(0, payload, offset, length - offset, null);
  assert(received > 0, "complete bootstrap payload required");
  offset += received;
}
const request = JSON.parse(payload.toString("utf8"));
assert.equal(request.jsonrpc, "2.0");
assert.equal(request.method, "initialize");
assert(typeof request.id === "number" || typeof request.id === "string");

const message = "native diagnostic transport intentionally unavailable";
const response = Buffer.from(
  JSON.stringify({ jsonrpc: "2.0", id: request.id, error: { code: -32041, message } }),
);
const frame = Buffer.concat([Buffer.from(`Content-Length: ${response.length}\r\n\r\n`), response]);
writeFileSync(
  join(captureDirectory, `${process.pid}.json`),
  JSON.stringify({
    argv,
    runtime: {
      nodeVersion: process.version,
      nodeExecutable: process.execPath,
      nodeSha256: createHash("sha256").update(readFileSync(process.execPath)).digest("hex"),
      wholePeerSource: readFileSync(fileURLToPath(import.meta.url), "utf8"),
    },
    wholeRequestFrame: [...header, ...payload],
    wholeResponseFrame: [...frame],
  }),
);
for (let offset = 0; offset < frame.length;) {
  offset += writeSync(1, frame, offset, frame.length - offset);
}
writeSync(2, `${message}\n`);
process.exit(41);

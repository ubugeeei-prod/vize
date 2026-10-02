import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { gzipSync } from "node:zlib";

const [input] = process.argv.slice(2);
assert(input);
const raw = fs.readFileSync(input);
const payload = JSON.parse(raw.toString());
assert.equal(payload.schema, 1);
assert.equal(payload.label, "linter-original-capture-proof");
assert(payload.files.length > 0 && payload.files.length <= 128);
assert(raw.length <= 32 * 1024 * 1024);
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const gzip = gzipSync(raw, { level: 9 });
assert(gzip.length <= 2 * 1024 * 1024);
const chunks = gzip.toString("base64").match(/.{1,12288}/g)!;
assert(chunks.length > 0 && chunks.length <= 228);
const header = {
  schema: 1,
  label: payload.label,
  files: payload.files.length,
  chunks: chunks.length,
  rawBytes: raw.length,
  gzipBytes: gzip.length,
  rawSha256: hash(raw),
  gzipSha256: hash(gzip),
};
console.log("VIZE_CAPTURE_HEADER_V1 " + JSON.stringify(header));
for (const [index, chunk] of chunks.entries())
  console.log(`VIZE_CAPTURE_CHUNK_V1 ${index + 1}/${chunks.length} ${chunk}`);
console.log(`VIZE_CAPTURE_END_V1 ${header.gzipSha256}`);

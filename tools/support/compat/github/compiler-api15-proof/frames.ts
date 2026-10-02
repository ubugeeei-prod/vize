import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";

const [input] = process.argv.slice(2);
assert(input);
const label = "compiler-api15-artifact-proof";
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const names: string[] = [];
function walk(relative: string) {
  for (const entry of fs.readdirSync(path.join(input, relative), { withFileTypes: true })) {
    const next = relative ? `${relative}/${entry.name}` : entry.name;
    assert(!entry.isSymbolicLink());
    if (entry.isDirectory()) walk(next);
    else if (entry.isFile() && next !== "artifact.zip") names.push(next);
  }
}
walk(""); names.sort();
assert(names.length > 0 && names.length <= 128);
const authority = JSON.parse(fs.readFileSync(path.join(input, "authority/transport.json"), "utf8"));
assert.equal(authority.hostedSourceBoundAuditAcceptedCases, 15);
assert.equal(authority.hostedLiteralZipAndEveryEntryCrcHashVerified, true);
assert.equal(authority.fullWorkflowAccepted, false);
assert.equal(authority.nativeHandled, 0);
const files = names.map(name => {
  const bytes = fs.readFileSync(path.join(input, name));
  assert(!path.isAbsolute(name) && !name.split("/").some(part => ["", ".", ".."].includes(part)));
  return { path: name, bytes: bytes.length, sha256: hash(bytes), base64: bytes.toString("base64") };
});
const capturedBytes = files.reduce((sum, file) => sum + file.bytes, 0);
assert(capturedBytes <= 16 * 1024 * 1024);
const raw = Buffer.from(JSON.stringify({ schema: 1, label, files }));
assert(raw.length <= 32 * 1024 * 1024);
const gzip = gzipSync(raw, { level: 9 });
assert(gzip.length <= 2 * 1024 * 1024);
const base64 = gzip.toString("base64");
const chunks = base64.match(/.{1,12288}/g)!;
assert(chunks.length > 0 && chunks.length <= 228);
const header = { schema: 1, label, files: files.length, chunks: chunks.length, rawBytes: raw.length, gzipBytes: gzip.length, capturedBytes, rawSha256: hash(raw), gzipSha256: hash(gzip) };
console.log("VIZE_CAPTURE_HEADER_V1 " + JSON.stringify(header));
for (const [index, chunk] of chunks.entries()) console.log(`VIZE_CAPTURE_CHUNK_V1 ${index + 1}/${chunks.length} ${chunk}`);
console.log(`VIZE_CAPTURE_END_V1 ${header.gzipSha256}`);

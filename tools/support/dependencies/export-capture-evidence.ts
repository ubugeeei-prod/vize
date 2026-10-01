// Lossless read-only transport for opt-in hosted captures, also kept as a file.
import { createHash } from "node:crypto";
import { lstatSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { gzipSync } from "node:zlib";

const [directory, output, label] = process.argv.slice(2);
if (!directory || !output || !label || !/^[a-z0-9-]{1,80}$/.test(label)) {
  throw new Error(
    "usage: export-capture-evidence.ts INPUT OUTPUT LABEL (lowercase letters/digits/dashes)",
  );
}
const root = path.resolve(directory);
const destination = path.resolve(output);
if (destination === root || destination.startsWith(`${root}${path.sep}`)) {
  throw new Error("Evidence output must be outside its captured input");
}
const maxFiles = 128;
const maxBytes = 16 * 1024 * 1024;
const maxCompressed = 2 * 1024 * 1024;
const chunkBytes = 12 * 1024;
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const files: { path: string; bytes: number; sha256: string; base64: string }[] = [];
let bytes = 0;
function visit(relative: string): void {
  const current = path.join(root, relative);
  const stat = lstatSync(current);
  if (stat.isSymbolicLink()) throw new Error(`Symlink is not captured: ${relative}`);
  if (stat.isDirectory()) {
    for (const child of readdirSync(current).sort()) visit(path.join(relative, child));
    return;
  }
  if (!stat.isFile() || !relative || relative.split(path.sep).some((part) => part === "..")) {
    throw new Error(`Invalid capture path: ${relative}`);
  }
  if (files.length === maxFiles || bytes + stat.size > maxBytes) {
    throw new Error("Capture exceeds the reviewed transport bounds");
  }
  const content = readFileSync(current);
  if (content.length !== stat.size) throw new Error(`Capture changed while reading: ${relative}`);
  bytes += content.length;
  files.push({
    path: relative.split(path.sep).join("/"),
    bytes: content.length,
    sha256: digest(content),
    base64: content.toString("base64"),
  });
}
visit("");
if (!files.length) throw new Error("Capture is empty");
const raw = Buffer.from(JSON.stringify({ schema: 1, label, files }) + "\n");
if (raw.length > maxBytes * 2)
  throw new Error("Capture JSON exceeds the reviewed transport bounds");
const compressed = gzipSync(raw, { level: 9 });
if (compressed.length > maxCompressed)
  throw new Error("Compressed capture exceeds the reviewed transport bounds");
mkdirSync(path.dirname(destination), { recursive: true });
writeFileSync(destination, compressed);
const encoded = compressed.toString("base64");
const chunks = Math.ceil(encoded.length / chunkBytes);
const header = {
  schema: 1,
  label,
  files: files.length,
  capturedBytes: bytes,
  rawBytes: raw.length,
  rawSha256: digest(raw),
  gzipBytes: compressed.length,
  gzipSha256: digest(compressed),
  chunks,
};
console.log(`VIZE_CAPTURE_HEADER_V1 ${JSON.stringify(header)}`);
for (let index = 0; index < chunks; index++) {
  console.log(
    `VIZE_CAPTURE_CHUNK_V1 ${index + 1}/${chunks} ${encoded.slice(index * chunkBytes, (index + 1) * chunkBytes)}`,
  );
}
console.log(`VIZE_CAPTURE_END_V1 ${header.gzipSha256}`);

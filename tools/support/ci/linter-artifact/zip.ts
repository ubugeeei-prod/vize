import assert from "node:assert/strict";
import path from "node:path";
import { inflateRawSync } from "node:zlib";

// Derived from reviewed transport 2aea09a; every entry, including binaries,
// is inflated, size/CRC checked and inventoried. No ZIP bytes are emitted.
const crcTable = Array.from({ length: 256 }, (_, index) => {
  let value = index;
  for (let bit = 0; bit < 8; bit++) value = value & 1 ? 0xedb88320 ^ (value >>> 1) : value >>> 1;
  return value >>> 0;
});
function crc32(bytes: Uint8Array) {
  let value = 0xffffffff;
  for (const byte of bytes) value = crcTable[(value ^ byte) & 255] ^ (value >>> 8);
  return (value ^ 0xffffffff) >>> 0;
}
export function zipEntries(zip: Buffer) {
  let end = -1;
  for (let offset = zip.length - 22; offset >= Math.max(0, zip.length - 65557); offset--) {
    if (
      zip.readUInt32LE(offset) === 0x06054b50 &&
      offset + 22 + zip.readUInt16LE(offset + 20) === zip.length
    ) {
      end = offset;
      break;
    }
  }
  assert(end >= 0, "complete ZIP end record");
  assert.equal(zip.readUInt16LE(end + 4), 0);
  assert.equal(zip.readUInt16LE(end + 6), 0);
  const count = zip.readUInt16LE(end + 10);
  assert.equal(zip.readUInt16LE(end + 8), count);
  assert(count > 0 && count <= 8192 && count !== 0xffff);
  const centralBytes = zip.readUInt32LE(end + 12);
  const start = zip.readUInt32LE(end + 16);
  assert.equal(start + centralBytes, end);
  let cursor = start;
  const entries: { path: string; bytes: Buffer; crc32: string; compressedBytes: number }[] = [];
  let totalBytes = 0;
  for (let index = 0; index < count; index++) {
    assert.equal(zip.readUInt32LE(cursor), 0x02014b50);
    const flags = zip.readUInt16LE(cursor + 8),
      method = zip.readUInt16LE(cursor + 10);
    assert.equal(flags & 1, 0);
    assert([0, 8].includes(method));
    const crc = zip.readUInt32LE(cursor + 16),
      compressed = zip.readUInt32LE(cursor + 20),
      size = zip.readUInt32LE(cursor + 24);
    assert(size <= 256 * 1024 * 1024 && compressed < zip.length);
    const nameLength = zip.readUInt16LE(cursor + 28),
      extra = zip.readUInt16LE(cursor + 30),
      comment = zip.readUInt16LE(cursor + 32);
    const local = zip.readUInt32LE(cursor + 42),
      nameBytes = zip.subarray(cursor + 46, cursor + 46 + nameLength);
    const name = nameBytes.toString("utf8");
    assert(
      nameBytes.equals(Buffer.from(name)) &&
        !path.isAbsolute(name) &&
        !name.includes("\\") &&
        !/[\0*?\[\]:]/.test(name),
    );
    assert(name.split("/").every((part) => part && part !== "." && part !== ".."));
    assert(!entries.some((entry) => entry.path === name));
    assert.equal(zip.readUInt32LE(local), 0x04034b50);
    assert.equal(zip.readUInt16LE(local + 6), flags);
    assert.equal(zip.readUInt16LE(local + 8), method);
    const localName = zip.readUInt16LE(local + 26),
      localExtra = zip.readUInt16LE(local + 28);
    assert(zip.subarray(local + 30, local + 30 + localName).equals(nameBytes));
    const offset = local + 30 + localName + localExtra;
    assert(offset + compressed <= start);
    const packed = zip.subarray(offset, offset + compressed);
    const bytes =
      method === 0 ? packed : inflateRawSync(packed, { maxOutputLength: 256 * 1024 * 1024 });
    assert.equal(bytes.length, size);
    assert.equal(crc32(bytes), crc);
    totalBytes += size;
    assert(totalBytes <= 512 * 1024 * 1024);
    entries.push({
      path: name,
      bytes,
      compressedBytes: compressed,
      crc32: crc.toString(16).padStart(8, "0"),
    });
    cursor += 46 + nameLength + extra + comment;
    assert(cursor <= start + centralBytes);
  }
  assert.equal(cursor, start + centralBytes);
  return entries;
}

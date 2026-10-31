import assert from "node:assert/strict";
import { test } from "node:test";
import { readZipEntry, writeZip, zipEntries } from "../../tools/support/archive/zip.ts";

// Python's standard zipfile, ZIP_DEFLATED, one empty stderr.log at the DOS epoch.
// This independent ZIP contains the two-byte 03 00 empty DEFLATE stream.
const emptyDeflated = Buffer.from(
  "UEsDBBQAAAAIAAAAIQAAAAAAAgAAAAAAAAAKAAAAc3RkZXJyLmxvZwMAUEsBAhQDFAAAAAgAAAAh" +
    "AAAAAAACAAAAAAAAAAoAAAAAAAAAAAAAAIABAAAAAHN0ZGVyci5sb2dQSwUGAAAAAAEAAQA4AAAAKgAAAAAA",
  "base64",
);

function claimEmpty(archive: Buffer): Buffer {
  const forged = Buffer.from(archive);
  const central = forged.readUInt32LE(forged.length - 22 + 16);
  const local = forged.readUInt32LE(central + 42);
  // Both headers consistently claim an empty member, but its compressed payload remains intact.
  for (const offset of [local + 14, local + 22, central + 16, central + 24])
    forged.writeUInt32LE(0, offset);
  return forged;
}

test("reads an independently encoded empty deflated stderr member", () => {
  const entries = zipEntries(emptyDeflated);
  assert.equal(entries.length, 1);
  const entry = entries[0]!;
  assert.equal(entry.name, "stderr.log");
  assert.equal(entry.method, 8);
  assert.equal(entry.compressedSize, 2);
  assert.equal(entry.size, 0);
  assert.equal(entry.crc32, 0);
  assert.deepEqual(readZipEntry(emptyDeflated, entry), Buffer.alloc(0));
});

test("rejects a one-byte payload whose two headers claim zero uncompressed bytes", () => {
  const archive = claimEmpty(writeZip([{ name: "stderr.log", data: "x", deflate: true }]));
  assert.throws(
    () => readZipEntry(archive, zipEntries(archive)[0]!),
    /zip member size mismatch: stderr\.log/u,
  );
});

test("keeps a bounded inflate for a larger payload claiming zero uncompressed bytes", () => {
  const archive = claimEmpty(writeZip([{ name: "stderr.log", data: "xx", deflate: true }]));
  assert.throws(() => readZipEntry(archive, zipEntries(archive)[0]!), {
    code: "ERR_BUFFER_TOO_LARGE",
  });
});

test("checks CRC even when the deflated member is empty", () => {
  const archive = Buffer.from(emptyDeflated);
  const central = archive.readUInt32LE(archive.length - 22 + 16);
  archive.writeUInt32LE(1, archive.readUInt32LE(central + 42) + 14);
  archive.writeUInt32LE(1, central + 16);
  assert.throws(
    () => readZipEntry(archive, zipEntries(archive)[0]!),
    /bad CRC-32 for zip member: stderr\.log/u,
  );
});

test("preserves empty stored and nonempty stored or deflated bytes", () => {
  for (const deflate of [false, true]) {
    for (const data of [Buffer.alloc(0), Buffer.from([0, 255, 127]), Buffer.from("雪🌸")]) {
      const archive = writeZip([{ name: "payload", data, deflate }]);
      assert.deepEqual(readZipEntry(archive, zipEntries(archive)[0]!), data);
    }
  }
});

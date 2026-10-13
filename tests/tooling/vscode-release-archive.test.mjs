import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import zlib from "node:zlib";
import { test } from "node:test";
import { extractReleaseArchive } from "../../editors/vscode/src/release-archive.ts";

const payload = Buffer.concat([
  Buffer.from("native executable\n日本語\0"),
  Buffer.alloc(4097, 0xaf),
]);

function tar(name) {
  const header = Buffer.alloc(512);
  header.write(name, 0);
  header.write(payload.length.toString(8).padStart(11, "0") + "\0", 124);
  header.write("0", 156);
  return zlib.gzipSync(
    Buffer.concat([
      header,
      payload,
      Buffer.alloc(Math.ceil(payload.length / 512) * 512 - payload.length + 1024),
    ]),
  );
}

function zip(name, compression = 0, size = payload.length) {
  const filename = Buffer.from(name);
  const compressed = compression === 8 ? zlib.deflateRawSync(payload) : payload;
  const local = Buffer.alloc(30);
  local.writeUInt32LE(0x04034b50, 0);
  local.writeUInt16LE(compression, 8);
  local.writeUInt32LE(compressed.length, 18);
  local.writeUInt32LE(size, 22);
  local.writeUInt16LE(filename.length, 26);
  const central = Buffer.alloc(46);
  central.writeUInt32LE(0x02014b50, 0);
  central.writeUInt16LE(compression, 10);
  central.writeUInt32LE(compressed.length, 20);
  central.writeUInt32LE(size, 24);
  central.writeUInt16LE(filename.length, 28);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(1, 10);
  end.writeUInt32LE(local.length + filename.length + compressed.length, 16);
  return Buffer.concat([local, filename, compressed, central, filename, end]);
}

await test("moved archive reader preserves full executable bytes and executable mode for tar/stored/deflated zip", async () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-reader-"));
  try {
    for (const [name, bytes] of [
      ["release.tar.gz", tar("release/vize")],
      ["stored.zip", zip("release/vize")],
      ["deflated.zip", zip("release/vize", 8)],
    ]) {
      const source = path.join(temporary, name);
      const output = path.join(temporary, name + "-output");
      fs.writeFileSync(source, bytes);
      fs.mkdirSync(output);
      await extractReleaseArchive(source, output, "vize");
      assert.deepEqual(fs.readFileSync(path.join(output, "vize")), payload);
      if (process.platform !== "win32")
        assert.equal(fs.statSync(path.join(output, "vize")).mode & 0o777, 0o755);
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

await test("moved reader retains original missing/malformed/size/compression refusals without publishing an executable", async () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-refusal-"));
  try {
    for (const [name, bytes, message] of [
      ["missing.tar.gz", tar("release/other"), "archive did not contain vize"],
      ["missing.zip", zip("release/other"), "archive did not contain vize"],
      ["malformed.zip", Buffer.alloc(22), "invalid zip archive"],
      [
        "stored-size.zip",
        zip("vize", 0, payload.length + 1),
        "stored zip entry has an unexpected size",
      ],
      [
        "deflated-size.zip",
        zip("vize", 8, payload.length + 1),
        "deflated zip entry has an unexpected size",
      ],
      ["compression.zip", zip("vize", 99), "unsupported zip compression method 99"],
    ]) {
      const source = path.join(temporary, name);
      const output = path.join(temporary, name + "-output");
      fs.writeFileSync(source, bytes);
      fs.mkdirSync(output);
      await assert.rejects(
        extractReleaseArchive(source, output, "vize"),
        (error) => error.message === message,
      );
      assert.equal(fs.existsSync(path.join(output, "vize")), false);
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

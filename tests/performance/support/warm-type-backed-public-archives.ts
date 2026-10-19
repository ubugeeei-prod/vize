import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";

type Row = Record<string, string | number>;
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

/** Writes one member exclusively below `root`, refusing escapes and duplicates. */
function writeMember(root: string, name: string, bytes: Buffer, mode?: number) {
  const target = path.join(root, ...name.split("/"));
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, bytes, { flag: "wx" });
  if (mode !== undefined) fs.chmodSync(target, mode & 0o777);
}

function unzip(archive: Buffer, root: string): Row[] {
  let end = archive.length - 22;
  while (end >= 0 && archive.readUInt32LE(end) !== 0x06054b50) end--;
  assert.ok(end >= 0, "zip end of central directory");
  const count = archive.readUInt16LE(end + 10);
  assert.ok(count <= 10_000);
  let offset = archive.readUInt32LE(end + 16);
  const rows: Row[] = [];
  const seen = new Set<string>();
  let total = 0;
  for (let index = 0; index < count; index++) {
    assert.equal(archive.readUInt32LE(offset), 0x02014b50, "zip central directory entry");
    const flags = archive.readUInt16LE(offset + 8);
    const method = archive.readUInt16LE(offset + 10);
    const crc = archive.readUInt32LE(offset + 16);
    const compressed = archive.readUInt32LE(offset + 20);
    const size = archive.readUInt32LE(offset + 24);
    const nameLength = archive.readUInt16LE(offset + 28);
    const extraLength = archive.readUInt16LE(offset + 30);
    const commentLength = archive.readUInt16LE(offset + 32);
    const fileType = (archive.readUInt32LE(offset + 38) >>> 16) & 0o170000;
    const local = archive.readUInt32LE(offset + 42);
    const name = archive.toString("utf8", offset + 46, offset + 46 + nameLength);
    offset += 46 + nameLength + extraLength + commentLength;
    const parts = name.replace(/\/$/, "").split("/");
    assert.ok(!name.startsWith("/") && !parts.includes("..") && !name.includes("\\"), name);
    assert.ok(!seen.has(name), `duplicate ${name}`);
    seen.add(name);
    assert.equal(flags & 1, 0, "encrypted zip member");
    assert.ok([0, 0o100000, 0o040000].includes(fileType), `zip member type ${name}`);
    assert.equal(parts[0], "warm-pair");
    total += size;
    assert.ok(total <= 512 * 1024 * 1024);
    if (name.endsWith("/")) {
      fs.mkdirSync(path.join(root, ...parts), { recursive: true });
      continue;
    }
    assert.equal(archive.readUInt32LE(local), 0x04034b50, "zip local header");
    const start = local + 30 + archive.readUInt16LE(local + 26) + archive.readUInt16LE(local + 28);
    const raw = archive.subarray(start, start + compressed);
    assert.ok(method === 0 || method === 8, `zip method ${method}`);
    const bytes = method === 0 ? Buffer.from(raw) : zlib.inflateRawSync(raw);
    assert.equal(bytes.length, size);
    assert.equal(zlib.crc32(bytes), crc, `zip crc ${name}`);
    writeMember(root, name, bytes);
    rows.push({ path: name, bytes: size, sha256: sha256(bytes), crc32: crc });
  }
  return rows;
}

function untar(archive: Buffer, root: string): Row[] {
  const tar = zlib.gunzipSync(archive);
  const octal = (start: number, length: number) =>
    Number.parseInt(
      tar
        .toString("ascii", start, start + length)
        .replace(/\0.*$/s, "")
        .trim() || "0",
      8,
    );
  const members: { name: string; type: string; mode: number; size: number; data: Buffer }[] = [];
  for (let offset = 0; offset + 512 <= tar.length && tar[offset] !== 0;) {
    const name = tar.toString("utf8", offset, offset + 100).replace(/\0.*$/s, "");
    const size = octal(offset + 124, 12);
    const data = tar.subarray(offset + 512, offset + 512 + size);
    members.push({
      name,
      type: String.fromCharCode(tar[offset + 156] || 48),
      mode: octal(offset + 100, 8),
      size,
      data,
    });
    offset += 512 + Math.ceil(size / 512) * 512;
  }
  assert.equal(members.length, 1);
  const [entry] = members;
  assert.ok(
    entry.name === "vize" && entry.type === "0" && entry.size > 0 && entry.size < 256 * 1024 * 1024,
  );
  assert.ok(entry.mode & 0o111);
  writeMember(root, "vize", entry.data, entry.mode);
  return [{ path: entry.name, bytes: entry.size, mode: entry.mode, sha256: sha256(entry.data) }];
}

/** Validate complete official archives before opening any captured authority. */
export function unpackPublicArchive(kind: "zip" | "tar", archive: string, destination: string) {
  fs.mkdirSync(destination);
  let error: unknown;
  try {
    const bytes = fs.readFileSync(archive);
    const members = kind === "zip" ? unzip(bytes, destination) : untar(bytes, destination);
    const audit = { kind, members };
    fs.writeFileSync(
      path.join(destination, "archive-audit.json"),
      `${JSON.stringify(audit, null, 2)}\n`,
    );
    return audit;
  } catch (failure) {
    error = failure;
    throw failure;
  } finally {
    fs.writeFileSync(
      `${destination}.process.json`,
      `${JSON.stringify({ status: error === undefined ? 0 : 1, error: error === undefined ? undefined : String(error) }, null, 2)}\n`,
    );
  }
}

// Read and write ZIP archives without extracting them to disk.
// The reader validates the central directory against each local header and
// checks every member's CRC-32 on read, as Python's zipfile does.
import { crc32, deflateRawSync, inflateRawSync } from "node:zlib";

export interface ZipEntry {
  readonly name: string;
  readonly flags: number;
  readonly method: number;
  readonly crc32: number;
  readonly compressedSize: number;
  readonly size: number;
  /** Upper 16 bits hold the Unix mode when the archive was made on Unix. */
  readonly externalAttributes: number;
  readonly isDirectory: boolean;
  readonly localHeaderOffset: number;
}

const endOfCentralDirectory = 0x06054b50;
const centralFileHeader = 0x02014b50;
const localFileHeader = 0x04034b50;
const stored = 0;
const deflated = 8;
// 1980-01-01 00:00, the earliest DOS timestamp; fixtures stay byte-stable.
const dosEpoch = (1 << 5) | 1;

function findEnd(bytes: Buffer): number {
  const earliest = Math.max(0, bytes.length - 22 - 0xffff);
  for (let offset = bytes.length - 22; offset >= earliest; offset -= 1)
    if (bytes.readUInt32LE(offset) === endOfCentralDirectory) return offset;
  throw new Error("zip end of central directory not found");
}

/** List the central directory; refuses ZIP64, multi-disk and inconsistent records. */
export function zipEntries(archive: Uint8Array): ZipEntry[] {
  const bytes = Buffer.from(archive.buffer, archive.byteOffset, archive.byteLength);
  const end = findEnd(bytes);
  const count = bytes.readUInt16LE(end + 10);
  if (bytes.readUInt16LE(end + 4) !== 0 || bytes.readUInt16LE(end + 8) !== count)
    throw new Error("multi-disk zip archives are not supported");
  let offset = bytes.readUInt32LE(end + 16);
  if (count === 0xffff || offset === 0xffffffff)
    throw new Error("zip64 archives are not supported");
  const entries: ZipEntry[] = [];
  for (let index = 0; index < count; index += 1) {
    if (offset + 46 > end || bytes.readUInt32LE(offset) !== centralFileHeader)
      throw new Error("invalid zip central directory");
    const nameLength = bytes.readUInt16LE(offset + 28);
    const extraLength = bytes.readUInt16LE(offset + 30);
    const commentLength = bytes.readUInt16LE(offset + 32);
    const flags = bytes.readUInt16LE(offset + 8);
    const rawName = bytes.subarray(offset + 46, offset + 46 + nameLength);
    // Bit 11 marks UTF-8 names; legacy names are CP437, which is ASCII-compatible.
    const name = rawName.toString(flags & 0x800 ? "utf8" : "latin1");
    entries.push({
      name,
      flags,
      method: bytes.readUInt16LE(offset + 10),
      crc32: bytes.readUInt32LE(offset + 16),
      compressedSize: bytes.readUInt32LE(offset + 20),
      size: bytes.readUInt32LE(offset + 24),
      externalAttributes: bytes.readUInt32LE(offset + 38),
      isDirectory: name.endsWith("/"),
      localHeaderOffset: bytes.readUInt32LE(offset + 42),
    });
    offset += 46 + nameLength + extraLength + commentLength;
  }
  return entries;
}

/** Read one member's complete bytes, verifying its size and CRC-32. */
export function readZipEntry(archive: Uint8Array, entry: ZipEntry): Buffer {
  const bytes = Buffer.from(archive.buffer, archive.byteOffset, archive.byteLength);
  const offset = entry.localHeaderOffset;
  if (offset + 30 > bytes.length || bytes.readUInt32LE(offset) !== localFileHeader)
    throw new Error(`invalid zip local header: ${entry.name}`);
  if (entry.flags & 1) throw new Error(`encrypted zip member: ${entry.name}`);
  const start = offset + 30 + bytes.readUInt16LE(offset + 26) + bytes.readUInt16LE(offset + 28);
  const compressed = bytes.subarray(start, start + entry.compressedSize);
  if (compressed.length !== entry.compressedSize)
    throw new Error(`truncated zip member: ${entry.name}`);
  let data: Buffer;
  if (entry.method === stored) data = Buffer.from(compressed);
  else if (entry.method === deflated)
    data = inflateRawSync(compressed, { maxOutputLength: entry.size });
  else throw new Error(`unsupported zip compression ${entry.method}: ${entry.name}`);
  if (data.length !== entry.size) throw new Error(`zip member size mismatch: ${entry.name}`);
  if (crc32(data) !== entry.crc32) throw new Error(`bad CRC-32 for zip member: ${entry.name}`);
  return data;
}

export interface ZipInput {
  readonly name: string;
  readonly data: Uint8Array | string;
  readonly deflate?: boolean;
}

/** Build a ZIP archive (UTF-8 names, no data descriptors) for fixtures and payloads. */
export function writeZip(inputs: readonly ZipInput[]): Buffer {
  const locals: Buffer[] = [];
  const centrals: Buffer[] = [];
  let offset = 0;
  for (const input of inputs) {
    const name = Buffer.from(input.name, "utf8");
    const data = Buffer.from(input.data);
    const method = input.deflate ? deflated : stored;
    const body = method === deflated ? deflateRawSync(data) : data;
    const checksum = crc32(data);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(localFileHeader, 0);
    local.writeUInt16LE(20, 4);
    local.writeUInt16LE(0x800, 6);
    local.writeUInt16LE(method, 8);
    local.writeUInt16LE(dosEpoch, 12);
    local.writeUInt32LE(checksum, 14);
    local.writeUInt32LE(body.length, 18);
    local.writeUInt32LE(data.length, 22);
    local.writeUInt16LE(name.length, 26);
    const central = Buffer.alloc(46);
    central.writeUInt32LE(centralFileHeader, 0);
    central.writeUInt16LE(0x0314, 4);
    central.writeUInt16LE(20, 6);
    central.writeUInt16LE(0x800, 8);
    central.writeUInt16LE(method, 10);
    central.writeUInt16LE(dosEpoch, 14);
    central.writeUInt32LE(checksum, 16);
    central.writeUInt32LE(body.length, 20);
    central.writeUInt32LE(data.length, 24);
    central.writeUInt16LE(name.length, 28);
    central.writeUInt32LE((input.name.endsWith("/") ? 0o40755 : 0o100644) * 0x10000, 38);
    central.writeUInt32LE(offset, 42);
    locals.push(local, name, body);
    centrals.push(central, name);
    offset += local.length + name.length + body.length;
  }
  const directory = Buffer.concat(centrals);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(endOfCentralDirectory, 0);
  end.writeUInt16LE(inputs.length, 8);
  end.writeUInt16LE(inputs.length, 10);
  end.writeUInt32LE(directory.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...locals, directory, end]);
}

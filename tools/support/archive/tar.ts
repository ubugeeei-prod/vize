// Read (and, for inert fixtures, write) gzip-compressed tar archives without
// extracting them. Member names, kinds and bytes follow Python's tarfile
// reader: ustar prefixes, PAX `path`/`linkpath`/`size` records, GNU long names,
// V7 directories spelled with a trailing slash, and verified header checksums.
import zlib from "node:zlib";

export type TarKind = "file" | "directory" | "symlink" | "hardlink" | "other";

export interface TarMember {
  readonly name: string;
  readonly kind: TarKind;
  readonly typeflag: string;
  readonly mode: number;
  readonly size: number;
  readonly linkname: string;
  readonly data: Buffer;
}

const block = 512;

function text(header: Buffer, offset: number, length: number): string {
  const field = header.subarray(offset, offset + length);
  const end = field.indexOf(0);
  return field.subarray(0, end === -1 ? field.length : end).toString("utf8");
}

function number(header: Buffer, offset: number, length: number): number {
  const field = header.subarray(offset, offset + length);
  if ((field[0] ?? 0) & 0x80) {
    // GNU base-256 encoding for values that do not fit the octal field.
    let value = (field[0] ?? 0) & 0x7f;
    for (const byte of field.subarray(1)) value = value * 256 + byte;
    return value;
  }
  const digits = text(header, offset, length).trim();
  if (!/^[0-7]*$/u.test(digits)) throw new Error("invalid tar header number");
  return digits ? Number.parseInt(digits, 8) : 0;
}

function checksumMatches(header: Buffer): boolean {
  const expected = number(header, 148, 8);
  let unsigned = 0;
  let signed = 0;
  for (let index = 0; index < block; index += 1) {
    const byte = index >= 148 && index < 156 ? 0x20 : (header[index] ?? 0);
    unsigned += byte;
    signed += byte > 127 ? byte - 256 : byte;
  }
  return expected === unsigned || expected === signed;
}

function kindOf(typeflag: string): TarKind {
  if (typeflag === "0" || typeflag === "" || typeflag === "7") return "file";
  if (typeflag === "5") return "directory";
  if (typeflag === "2") return "symlink";
  if (typeflag === "1") return "hardlink";
  return "other";
}

function paxRecords(data: Buffer): Map<string, string> {
  const records = new Map<string, string>();
  let offset = 0;
  while (offset < data.length) {
    const space = data.indexOf(0x20, offset);
    const length = Number(data.subarray(offset, space).toString("ascii"));
    if (space === -1 || !Number.isSafeInteger(length) || length <= 0)
      throw new Error("invalid PAX extended header");
    const record = data.subarray(space + 1, offset + length - 1).toString("utf8");
    const equals = record.indexOf("=");
    if (equals === -1) throw new Error("invalid PAX extended header");
    records.set(record.slice(0, equals), record.slice(equals + 1));
    offset += length;
  }
  return records;
}

/** Every member of a .tar.gz payload, in archive order. */
export function readTarGz(payload: Uint8Array): TarMember[] {
  return readTar(zlib.gunzipSync(payload));
}

/** Every member of an uncompressed tar payload, in archive order. */
export function readTar(payload: Uint8Array): TarMember[] {
  const tar = Buffer.from(payload.buffer, payload.byteOffset, payload.byteLength);
  const members: TarMember[] = [];
  let longName: string | null = null;
  let longLink: string | null = null;
  let pax = new Map<string, string>();
  const global = new Map<string, string>();
  for (let offset = 0; offset + block <= tar.length;) {
    const header = tar.subarray(offset, offset + block);
    if (header.every((byte) => byte === 0)) break;
    if (!checksumMatches(header)) throw new Error("invalid tar header checksum");
    const typeflag = text(header, 156, 1);
    const records = new Map([...global, ...pax]);
    const sizeOverride = records.get("size");
    const size = sizeOverride === undefined ? number(header, 124, 12) : Number(sizeOverride);
    const start = offset + block;
    const data = tar.subarray(start, start + size);
    if (data.length !== size) throw new Error("truncated tar member");
    offset = start + Math.ceil(size / block) * block;
    if (typeflag === "L" || typeflag === "K") {
      const value = text(data, 0, data.length);
      if (typeflag === "L") longName = value;
      else longLink = value;
      continue;
    }
    if (typeflag === "x" || typeflag === "g") {
      const parsed = paxRecords(data);
      if (typeflag === "g") for (const [key, value] of parsed) global.set(key, value);
      else pax = parsed;
      continue;
    }
    const prefix = text(header, 345, 155);
    const magic = text(header, 257, 6);
    let name = text(header, 0, 100);
    if (prefix && magic.startsWith("ustar")) name = `${prefix}/${name}`;
    name = records.get("path") ?? longName ?? name;
    let kind = kindOf(typeflag);
    // V7 archives spell a directory as a regular member with a trailing slash.
    if (typeflag === "" && name.endsWith("/")) kind = "directory";
    if (kind === "directory") name = name.replace(/\/+$/u, "");
    members.push({
      name,
      kind,
      typeflag: typeflag || "\0",
      mode: number(header, 100, 8),
      size,
      linkname: records.get("linkpath") ?? longLink ?? text(header, 157, 100),
      data,
    });
    longName = null;
    longLink = null;
    pax = new Map();
  }
  return members;
}

export interface TarEntry {
  readonly name: string;
  readonly data?: Uint8Array;
  readonly mode?: number;
  readonly typeflag?: string;
  readonly linkname?: string;
}

function octal(header: Buffer, offset: number, length: number, value: number): void {
  header.write(`${value.toString(8).padStart(length - 1, "0")}\0`, offset, length, "ascii");
}

export function writeTarGz(entries: readonly TarEntry[]): Buffer {
  return zlib.gzipSync(writeTar(entries));
}

/** A minimal ustar writer for inert test archives; names must fit 100 bytes. */
export function writeTar(entries: readonly TarEntry[]): Buffer {
  const blocks: Buffer[] = [];
  for (const entry of entries) {
    const data = Buffer.from(entry.data ?? new Uint8Array());
    if (Buffer.byteLength(entry.name) > 100) throw new Error("tar fixture name too long");
    const header = Buffer.alloc(block);
    header.write(entry.name, 0, 100, "utf8");
    octal(header, 100, 8, entry.mode ?? 0o644);
    octal(header, 108, 8, 0);
    octal(header, 116, 8, 0);
    octal(header, 124, 12, data.length);
    octal(header, 136, 12, 0);
    header.write(entry.typeflag ?? "0", 156, 1, "ascii");
    header.write(entry.linkname ?? "", 157, 100, "utf8");
    header.write("ustar\0" + "00", 257, 8, "ascii");
    header.fill(0x20, 148, 156);
    const sum = header.reduce((total, byte) => total + byte, 0);
    header.write(`${sum.toString(8).padStart(6, "0")}\0 `, 148, 8, "ascii");
    blocks.push(header, data, Buffer.alloc((block - (data.length % block)) % block));
  }
  blocks.push(Buffer.alloc(block * 2));
  return Buffer.concat(blocks);
}

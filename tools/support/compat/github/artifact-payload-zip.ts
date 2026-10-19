// Minimal ZIP reader/writer for the opt-in artifact payload trial: stored and
// deflate entries, ZIP64, UTF-8 names and complete external attributes.
import {
  closeSync,
  createReadStream,
  fstatSync,
  openSync,
  readSync,
  statSync,
  writeSync,
} from "node:fs";
import { Readable } from "node:stream";
import { constants, crc32, createDeflateRaw, createInflateRaw, deflateRawSync } from "node:zlib";

const LOCAL = 0x04034b50;
const CENTRAL = 0x02014b50;
const END = 0x06054b50;
const END64 = 0x06064b50;
const LOCATOR64 = 0x07064b50;
const MAX32 = 0xffffffff;
const MAX16 = 0xffff;
// Code page 437 high half, the ZIP default for names without the UTF-8 flag.
const CP437 =
  "ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ";

export interface ZipEntry {
  readonly name: string;
  readonly flags: number;
  readonly method: number;
  readonly crc: number;
  readonly compressedSize: number;
  readonly size: number;
  readonly externalAttr: number;
  readonly createSystem: number;
  readonly offset: number;
}

export interface ZipAddOptions {
  readonly externalAttr?: number;
  readonly createSystem?: number;
  readonly method?: number;
  readonly level?: number;
}

interface WrittenEntry {
  readonly name: Buffer;
  readonly flags: number;
  readonly method: number;
  readonly crc: number;
  readonly compressedSize: number;
  readonly size: number;
  readonly externalAttr: number;
  readonly createSystem: number;
  readonly offset: number;
  readonly time: number;
  readonly date: number;
}

function fail(message: string): never {
  throw new Error(`Bad ZIP file: ${message}`);
}

function readAt(fd: number, position: number, length: number): Buffer {
  const buffer = Buffer.alloc(length);
  let done = 0;
  while (done < length) {
    const read = readSync(fd, buffer, done, length - done, position + done);
    if (read === 0) fail("truncated archive");
    done += read;
  }
  return buffer;
}

function writeAt(fd: number, buffer: Uint8Array, position: number): void {
  let done = 0;
  while (done < buffer.length) {
    done += writeSync(fd, buffer, done, buffer.length - done, position + done);
  }
}

function u64(buffer: Buffer, offset: number): number {
  const value = buffer.readBigUInt64LE(offset);
  if (value > BigInt(Number.MAX_SAFE_INTEGER)) fail("ZIP64 value exceeds the safe range");
  return Number(value);
}

function decodeName(raw: Buffer, flags: number): string {
  const name = flags & 0x800 ? raw.toString("utf8") : [...raw].map(cp437).join("");
  // Python's zipfile truncates names at the first NUL.
  const nul = name.indexOf("\0");
  return nul === -1 ? name : name.slice(0, nul);
}

function cp437(byte: number): string {
  return byte < 0x80 ? String.fromCharCode(byte) : CP437.charAt(byte - 0x80);
}

function zip64Fields(extra: Buffer, wanted: readonly boolean[]): number[] {
  let offset = 0;
  while (offset + 4 <= extra.length) {
    const id = extra.readUInt16LE(offset);
    const length = extra.readUInt16LE(offset + 2);
    if (id === 0x0001) {
      const values: number[] = [];
      let cursor = offset + 4;
      for (const want of wanted) {
        if (!want) continue;
        if (cursor + 8 > offset + 4 + length) fail("corrupt ZIP64 extra field");
        values.push(u64(extra, cursor));
        cursor += 8;
      }
      return values;
    }
    offset += 4 + length;
  }
  if (wanted.some(Boolean)) fail("missing ZIP64 extra field");
  return [];
}

export function readZip(path: string): ZipEntry[] {
  const fd = openSync(path, "r");
  try {
    const size = fstatSync(fd).size;
    const tailLength = Math.min(size, 22 + MAX16);
    const tail = readAt(fd, size - tailLength, tailLength);
    let end = -1;
    for (let index = tail.length - 22; index >= 0; index--) {
      if (
        tail.readUInt32LE(index) === END &&
        index + 22 + tail.readUInt16LE(index + 20) <= tail.length
      ) {
        end = index;
        break;
      }
    }
    if (end === -1) fail("end of central directory not found");
    let count = tail.readUInt16LE(end + 10);
    let directorySize = tail.readUInt32LE(end + 12);
    let directoryOffset = tail.readUInt32LE(end + 16);
    const endPosition = size - tailLength + end;
    if (endPosition >= 20) {
      const locator = readAt(fd, endPosition - 20, 20);
      if (locator.readUInt32LE(0) === LOCATOR64) {
        const record = readAt(fd, u64(locator, 8), 56);
        if (record.readUInt32LE(0) !== END64) fail("corrupt ZIP64 end of central directory");
        count = u64(record, 32);
        directorySize = u64(record, 40);
        directoryOffset = u64(record, 48);
      }
    }
    const directory = readAt(fd, directoryOffset, directorySize);
    const entries: ZipEntry[] = [];
    let cursor = 0;
    for (let index = 0; index < count; index++) {
      if (directory.readUInt32LE(cursor) !== CENTRAL) fail("corrupt central directory");
      const flags = directory.readUInt16LE(cursor + 8);
      const nameLength = directory.readUInt16LE(cursor + 28);
      const extraLength = directory.readUInt16LE(cursor + 30);
      const commentLength = directory.readUInt16LE(cursor + 32);
      const nameStart = cursor + 46;
      const extra = directory.subarray(
        nameStart + nameLength,
        nameStart + nameLength + extraLength,
      );
      let compressedSize = directory.readUInt32LE(cursor + 20);
      let entrySize = directory.readUInt32LE(cursor + 24);
      let offset = directory.readUInt32LE(cursor + 42);
      const wide = zip64Fields(extra, [
        entrySize === MAX32,
        compressedSize === MAX32,
        offset === MAX32,
      ]);
      if (entrySize === MAX32) entrySize = wide.shift() ?? fail("corrupt ZIP64 size");
      if (compressedSize === MAX32) compressedSize = wide.shift() ?? fail("corrupt ZIP64 size");
      if (offset === MAX32) offset = wide.shift() ?? fail("corrupt ZIP64 offset");
      entries.push({
        name: decodeName(directory.subarray(nameStart, nameStart + nameLength), flags),
        flags,
        method: directory.readUInt16LE(cursor + 10),
        crc: directory.readUInt32LE(cursor + 16),
        compressedSize,
        size: entrySize,
        externalAttr: directory.readUInt32LE(cursor + 38),
        createSystem: directory.readUInt8(cursor + 5),
        offset,
      });
      cursor = nameStart + nameLength + extraLength + commentLength;
    }
    return entries;
  } finally {
    closeSync(fd);
  }
}

function dataStart(path: string, entry: ZipEntry): number {
  const fd = openSync(path, "r");
  try {
    const header = readAt(fd, entry.offset, 30);
    if (header.readUInt32LE(0) !== LOCAL) fail("bad magic number for file header");
    const nameLength = header.readUInt16LE(26);
    const extraLength = header.readUInt16LE(28);
    const raw = readAt(fd, entry.offset + 30, nameLength + extraLength);
    const name = decodeName(raw.subarray(0, nameLength), header.readUInt16LE(6));
    if (name !== entry.name) fail("file name in directory and header differ");
    return entry.offset + 30 + nameLength + extraLength;
  } finally {
    closeSync(fd);
  }
}

/** Stream one entry's uncompressed body; its CRC-32 and size are verified at the end. */
export async function* zipEntryChunks(path: string, entry: ZipEntry): AsyncGenerator<Buffer> {
  if (entry.flags & 0x1) fail(`encrypted entry ${entry.name}`);
  if (entry.method !== 0 && entry.method !== 8) fail(`unsupported compression ${entry.method}`);
  const start = dataStart(path, entry);
  let raw: Readable =
    entry.compressedSize === 0
      ? Readable.from([])
      : createReadStream(path, { start, end: start + entry.compressedSize - 1 });
  if (entry.method === 8) {
    const inflate = createInflateRaw();
    raw.on("error", (error) => inflate.destroy(error));
    raw = raw.pipe(inflate);
  }
  let crc = 0;
  let size = 0;
  for await (const chunk of raw) {
    const bytes = chunk as Buffer;
    crc = crc32(bytes, crc);
    size += bytes.length;
    yield bytes;
  }
  if (size !== entry.size) fail(`size differs for file ${entry.name}`);
  if (crc >>> 0 !== entry.crc) fail(`Bad CRC-32 for file ${entry.name}`);
}

export async function readZipEntry(path: string, entry: ZipEntry): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for await (const chunk of zipEntryChunks(path, entry)) chunks.push(chunk);
  return Buffer.concat(chunks);
}

function dosStamp(date: Date): { time: number; date: number } {
  if (date.getFullYear() < 1980) return { time: 0, date: (0 << 9) | (1 << 5) | 1 };
  return {
    time: (date.getHours() << 11) | (date.getMinutes() << 5) | Math.floor(date.getSeconds() / 2),
    date: ((date.getFullYear() - 1980) << 9) | ((date.getMonth() + 1) << 5) | date.getDate(),
  };
}

export class ZipWriter {
  readonly #fd: number;
  #position = 0;
  readonly #entries: WrittenEntry[] = [];

  /** `exclusive` refuses an existing path, like Python's mode "x". */
  constructor(path: string, exclusive = false) {
    this.#fd = openSync(path, exclusive ? "wx" : "w");
  }

  /** Add a body from memory, or stream it from a file path. */
  async add(name: string, body: Uint8Array | string, options: ZipAddOptions = {}): Promise<void> {
    const method = options.method ?? 8;
    if (method !== 0 && method !== 8) fail(`unsupported compression ${method}`);
    const level = options.level ?? constants.Z_DEFAULT_COMPRESSION;
    const encoded = Buffer.from(name, "utf8");
    const flags = Buffer.byteLength(name) === name.length ? 0 : 0x800;
    const fileStat = typeof body === "string" ? statSync(body) : undefined;
    const size = fileStat?.size ?? (body as Uint8Array).length;
    const stamp = dosStamp(fileStat?.mtime ?? new Date(1980, 0, 1));
    const externalAttr =
      options.externalAttr ?? (fileStat ? ((fileStat.mode & 0xffff) << 16) >>> 0 : 0);
    const zip64 = size * 1.05 > MAX32;
    const offset = this.#position;
    const extra = zip64 ? Buffer.alloc(20) : Buffer.alloc(0);
    const header = Buffer.alloc(30);
    this.#position += 30 + encoded.length + extra.length;
    let crc = 0;
    let compressedSize = 0;
    const emit = (chunk: Uint8Array): void => {
      writeAt(this.#fd, chunk, this.#position);
      this.#position += chunk.length;
      compressedSize += chunk.length;
    };
    if (typeof body === "string") {
      const source = createReadStream(body, { highWaterMark: 1024 * 1024 });
      let output: Readable = source;
      if (method === 8) {
        const deflate = createDeflateRaw({ level });
        source.on("error", (error) => deflate.destroy(error));
        source.on("data", (chunk) => {
          crc = crc32(chunk as Buffer, crc);
        });
        output = source.pipe(deflate);
      }
      for await (const chunk of output) {
        if (method === 0) crc = crc32(chunk as Buffer, crc);
        emit(chunk as Buffer);
      }
    } else {
      crc = crc32(body);
      emit(method === 8 ? deflateRawSync(body, { level }) : body);
    }
    const entry: WrittenEntry = {
      name: encoded,
      flags,
      method,
      crc: crc >>> 0,
      compressedSize,
      size,
      externalAttr,
      createSystem: options.createSystem ?? 3,
      offset,
      time: stamp.time,
      date: stamp.date,
    };
    header.writeUInt32LE(LOCAL, 0);
    header.writeUInt16LE(zip64 ? 45 : 20, 4);
    header.writeUInt16LE(flags, 6);
    header.writeUInt16LE(method, 8);
    header.writeUInt16LE(entry.time, 10);
    header.writeUInt16LE(entry.date, 12);
    header.writeUInt32LE(entry.crc, 14);
    header.writeUInt32LE(zip64 ? MAX32 : compressedSize, 18);
    header.writeUInt32LE(zip64 ? MAX32 : size, 22);
    header.writeUInt16LE(encoded.length, 26);
    header.writeUInt16LE(extra.length, 28);
    if (zip64) {
      extra.writeUInt16LE(0x0001, 0);
      extra.writeUInt16LE(16, 2);
      extra.writeBigUInt64LE(BigInt(size), 4);
      extra.writeBigUInt64LE(BigInt(compressedSize), 12);
    } else if (compressedSize > MAX32) {
      fail("compressed size exceeds the ZIP64 estimate");
    }
    writeAt(this.#fd, Buffer.concat([header, encoded, extra]), offset);
    this.#entries.push(entry);
  }

  close(): void {
    try {
      const start = this.#position;
      const records: Buffer[] = [];
      for (const entry of this.#entries) {
        const wide = [entry.size, entry.compressedSize, entry.offset].map(
          (value) => value >= MAX32,
        );
        const values = [entry.size, entry.compressedSize, entry.offset].filter(
          (_, index) => wide[index],
        );
        const extra = Buffer.alloc(values.length === 0 ? 0 : 4 + 8 * values.length);
        if (values.length > 0) {
          extra.writeUInt16LE(0x0001, 0);
          extra.writeUInt16LE(8 * values.length, 2);
          values.forEach((value, index) => extra.writeBigUInt64LE(BigInt(value), 4 + 8 * index));
        }
        const version = values.length > 0 ? 45 : 20;
        const header = Buffer.alloc(46);
        header.writeUInt32LE(CENTRAL, 0);
        header.writeUInt8(version, 4);
        header.writeUInt8(entry.createSystem, 5);
        header.writeUInt16LE(version, 6);
        header.writeUInt16LE(entry.flags, 8);
        header.writeUInt16LE(entry.method, 10);
        header.writeUInt16LE(entry.time, 12);
        header.writeUInt16LE(entry.date, 14);
        header.writeUInt32LE(entry.crc, 16);
        header.writeUInt32LE(wide[1] ? MAX32 : entry.compressedSize, 20);
        header.writeUInt32LE(wide[0] ? MAX32 : entry.size, 24);
        header.writeUInt16LE(entry.name.length, 28);
        header.writeUInt16LE(extra.length, 30);
        header.writeUInt32LE(entry.externalAttr >>> 0, 38);
        header.writeUInt32LE(wide[2] ? MAX32 : entry.offset, 42);
        records.push(header, entry.name, extra);
      }
      const directory = Buffer.concat(records);
      writeAt(this.#fd, directory, start);
      let position = start + directory.length;
      const count = this.#entries.length;
      const wide = count >= MAX16 || start >= MAX32 || directory.length >= MAX32;
      if (wide) {
        const record = Buffer.alloc(56);
        record.writeUInt32LE(END64, 0);
        record.writeBigUInt64LE(44n, 4);
        record.writeUInt16LE(45, 12);
        record.writeUInt16LE(45, 14);
        record.writeBigUInt64LE(BigInt(count), 24);
        record.writeBigUInt64LE(BigInt(count), 32);
        record.writeBigUInt64LE(BigInt(directory.length), 40);
        record.writeBigUInt64LE(BigInt(start), 48);
        const locator = Buffer.alloc(20);
        locator.writeUInt32LE(LOCATOR64, 0);
        locator.writeBigUInt64LE(BigInt(position), 8);
        locator.writeUInt32LE(1, 16);
        writeAt(this.#fd, Buffer.concat([record, locator]), position);
        position += 76;
      }
      const end = Buffer.alloc(22);
      end.writeUInt32LE(END, 0);
      end.writeUInt16LE(Math.min(count, MAX16), 8);
      end.writeUInt16LE(Math.min(count, MAX16), 10);
      end.writeUInt32LE(Math.min(directory.length, MAX32), 12);
      end.writeUInt32LE(Math.min(start, MAX32), 16);
      writeAt(this.#fd, end, position);
    } finally {
      closeSync(this.#fd);
    }
  }
}

/** Streaming inert tar reader; retained payloads alone consume bounded memory. */
import assert from "node:assert/strict";
import { createReadStream } from "node:fs";
import { createGunzip } from "node:zlib";
import type { Readable } from "node:stream";
export type EvidenceTarMember = {
  name: string;
  kind: "file" | "directory" | "link" | "other";
  size: bigint;
  data: Buffer | null;
};
class Bytes {
  iterator: AsyncIterator<unknown>;
  chunk: Buffer = Buffer.alloc(0);
  offset = 0;
  constructor(iterator: AsyncIterator<unknown>) {
    this.iterator = iterator;
  }
  async consume(size: bigint, retain: boolean, optional = false): Promise<Buffer | null> {
    assert.ok(size >= 0n);
    const output = retain ? Buffer.alloc(Number(size)) : null;
    let left = size,
      written = 0;
    while (left > 0n) {
      if (this.offset === this.chunk.length) {
        const next = await this.iterator.next();
        if (next.done) {
          if (optional && left === size) return null;
          throw new Error("truncated tar stream");
        }
        assert.ok(Buffer.isBuffer(next.value));
        this.chunk = next.value;
        this.offset = 0;
      }
      const available = this.chunk.length - this.offset;
      const count = left < BigInt(available) ? Number(left) : available;
      if (output) this.chunk.copy(output, written, this.offset, this.offset + count);
      this.offset += count;
      written += count;
      left -= BigInt(count);
    }
    return output;
  }
}
const utf8 = new TextDecoder("utf-8", { fatal: true });
const text = (body: Buffer, offset: number, length: number) => {
  const field = body.subarray(offset, offset + length),
    end = field.indexOf(0);
  return utf8.decode(end < 0 ? field : field.subarray(0, end));
};
function integer(body: Buffer, offset: number, length: number): bigint {
  const field = body.subarray(offset, offset + length),
    first = field[0] ?? 0;
  if (first & 0x80) {
    assert.equal(first & 0x40, 0, "negative GNU tar integer");
    let value = BigInt(first & 0x7f);
    for (const byte of field.subarray(1)) value = value * 256n + BigInt(byte);
    return value;
  }
  const digits = text(body, offset, length).trim();
  assert.match(digits, /^[0-7]*$/u);
  return digits ? BigInt("0o" + digits) : 0n;
}
function checksum(body: Buffer) {
  let signed = 0n,
    unsigned = 0n;
  for (let i = 0; i < 512; i++) {
    const byte = i >= 148 && i < 156 ? 32 : (body[i] ?? 0);
    unsigned += BigInt(byte);
    signed += BigInt(byte > 127 ? byte - 256 : byte);
  }
  const expected = integer(body, 148, 8);
  assert.ok(expected === unsigned || expected === signed, "tar header checksum mismatch");
}
function pax(body: Buffer) {
  const out = new Map<string, string>();
  let offset = 0;
  while (offset < body.length) {
    const space = body.indexOf(32, offset);
    assert.ok(space >= offset);
    const count = Number(body.subarray(offset, space).toString("ascii"));
    assert.ok(Number.isSafeInteger(count) && count > 0 && offset + count <= body.length);
    assert.equal(body[offset + count - 1], 10);
    const value = utf8.decode(body.subarray(space + 1, offset + count - 1)),
      equal = value.indexOf("=");
    assert.ok(equal >= 0);
    out.set(value.slice(0, equal), value.slice(equal + 1));
    offset += count;
  }
  return out;
}
export async function* evidenceTar(
  archive: string,
  retain: (member: Omit<EvidenceTarMember, "data">) => boolean,
  maxRetainedBytes = 8 * 1024 * 1024,
): AsyncGenerator<EvidenceTarMember> {
  yield* evidenceTarStream(createReadStream(archive), retain, maxRetainedBytes);
}
export async function* evidenceTarStream(
  source: Readable,
  retain: (member: Omit<EvidenceTarMember, "data">) => boolean,
  maxRetainedBytes = 8 * 1024 * 1024,
): AsyncGenerator<EvidenceTarMember> {
  assert.ok(Number.isSafeInteger(maxRetainedBytes) && maxRetainedBytes <= 32 * 1024 * 1024);
  const gunzip = createGunzip();
  source.on("error", (error) => gunzip.destroy(error));
  source.pipe(gunzip);
  const reader = new Bytes(gunzip[Symbol.asyncIterator]());
  let longName: string | null = null,
    local = new Map<string, string>();
  const global = new Map<string, string>();
  try {
    for (;;) {
      const header = await reader.consume(512n, true, true);
      if (header === null || header.every((byte) => byte === 0)) break;
      checksum(header);
      const type = text(header, 156, 1),
        records = new Map([...global, ...local]);
      const extension = ["x", "g", "L", "K"].includes(type);
      const sizeText = extension ? undefined : records.get("size");
      if (sizeText !== undefined) assert.match(sizeText, /^\d+$/u);
      const size = sizeText === undefined ? integer(header, 124, 12) : BigInt(sizeText);
      let name = text(header, 0, 100);
      const prefix = text(header, 345, 155);
      if (prefix && text(header, 257, 6).startsWith("ustar")) name = prefix + "/" + name;
      name = records.get("path") ?? longName ?? name;
      const kind: EvidenceTarMember["kind"] =
        type === "5" || (type === "" && name.endsWith("/"))
          ? "directory"
          : ["0", "", "7"].includes(type)
            ? "file"
            : ["1", "2"].includes(type)
              ? "link"
              : "other";
      const member = { name, kind, size };
      const keep = extension || retain(member);
      assert.ok(
        !keep || size <= BigInt(extension ? 1024 * 1024 : maxRetainedBytes),
        "retained tar payload bound",
      );
      const data = await reader.consume(size, keep);
      await reader.consume((512n - (size % 512n)) % 512n, false);
      if (extension) {
        assert.ok(data);
        if (type === "L") longName = text(data, 0, data.length);
        else if (type === "x") local = pax(data);
        else if (type === "g") for (const [key, value] of pax(data)) global.set(key, value);
        continue;
      }
      yield { ...member, data };
      longName = null;
      local = new Map();
    }
  } finally {
    source.destroy();
    gunzip.destroy();
  }
}

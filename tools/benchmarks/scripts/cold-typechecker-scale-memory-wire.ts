import assert from "node:assert/strict";
type Field = { tag: number; wire: number; value: bigint | Buffer };
function varint(bytes: Buffer, start: number): [bigint, number] {
  let value = 0n,
    shift = 0n,
    cursor = start;
  for (; cursor < bytes.length && shift < 70n; cursor++, shift += 7n) {
    const byte = bytes[cursor];
    value |= BigInt(byte & 127) << shift;
    if (!(byte & 128)) {
      assert(value < 1n << 64n, "protobuf uint64 overflow");
      return [value, cursor + 1];
    }
  }
  throw new Error("invalid/truncated protobuf varint");
}
function fields(bytes: Buffer): Field[] {
  const result: Field[] = [];
  let cursor = 0;
  while (cursor < bytes.length) {
    const [key, next] = varint(bytes, cursor);
    cursor = next;
    const tag = Number(key >> 3n),
      wire = Number(key & 7n);
    assert(tag > 0 && tag <= 0x1fffffff);
    if (wire === 0) {
      const [value, end] = varint(bytes, cursor);
      cursor = end;
      result.push({ tag, wire, value });
    } else {
      let size: number;
      if (wire === 2) {
        const [length, end] = varint(bytes, cursor);
        cursor = end;
        size = Number(length);
      } else {
        assert(wire === 1 || wire === 5, "unsupported protobuf wire");
        size = wire === 1 ? 8 : 4;
      }
      assert(Number.isSafeInteger(size) && size >= 0 && cursor + size <= bytes.length);
      result.push({ tag, wire, value: bytes.subarray(cursor, cursor + size) });
      cursor += size;
    }
  }
  return result;
}
const numeric = (all: Field[], tag: number) => {
  const found = all.filter((f) => f.tag === tag);
  assert(found.length <= 1);
  if (!found.length) return 0n;
  assert(typeof found[0].value === "bigint");
  return found[0].value;
};
const messages = (all: Field[], tag: number) =>
  all
    .filter((f) => f.tag === tag)
    .map((f) => {
      assert(f.wire === 2 && Buffer.isBuffer(f.value));
      return fields(f.value);
    });
const packed = (all: Field[], tag: number) =>
  all
    .filter((f) => f.tag === tag)
    .flatMap((f) => {
      if (typeof f.value === "bigint") return [f.value];
      assert(f.wire === 2);
      const values: bigint[] = [];
      let cursor = 0;
      while (cursor < f.value.length) {
        const [v, end] = varint(f.value, cursor);
        values.push(v);
        cursor = end;
      }
      return values;
    });
const signed = (value: bigint) => (value >= 1n << 63n ? value - (1n << 64n) : value);
const unknown = (all: Field[], known: number[]) =>
  all
    .filter((f) => !known.includes(f.tag))
    .map((f) => ({
      tag: f.tag,
      wire: f.wire,
      value: typeof f.value === "bigint" ? f.value.toString() : f.value.toString("hex"),
    }));

export { fields, numeric, messages, packed, signed, unknown };
export type { Field };

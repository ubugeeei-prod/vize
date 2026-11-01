/** Bounded CPU pprof schema reader. Go -raw remains the independent semantic reader. */
import assert from "node:assert/strict";
import { gunzipSync } from "node:zlib";
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

export function decodeCpuProfile(compressed: Buffer) {
  assert(compressed.length > 0 && compressed.length <= 16 * 1024 * 1024);
  const bytes = gunzipSync(compressed, { maxOutputLength: 64 * 1024 * 1024 }),
    all = fields(bytes);
  const strings = all
    .filter((f) => f.tag === 6)
    .map((f) => {
      assert(f.wire === 2 && Buffer.isBuffer(f.value));
      return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(f.value);
    });
  assert.equal(strings[0], "");
  const text = (value: bigint) => {
    const i = Number(value);
    assert(Number.isSafeInteger(i) && i >= 0 && i < strings.length);
    return strings[i];
  };
  const valueType = (value: Field[]) => ({
    type: text(numeric(value, 1)),
    unit: text(numeric(value, 2)),
    unknown: unknown(value, [1, 2]),
  });
  const types = messages(all, 1).map(valueType);
  assert.deepEqual(
    types.map((t) => [t.type, t.unit]),
    [
      ["samples", "count"],
      ["cpu", "nanoseconds"],
    ],
  );
  const period = signed(numeric(all, 12)),
    periodTypes = messages(all, 11).map(valueType);
  assert(period > 0n);
  assert.deepEqual(
    periodTypes.map((t) => [t.type, t.unit]),
    [["cpu", "nanoseconds"]],
  );
  const samples = messages(all, 2).map((value) => {
    const locations = packed(value, 1).map(String),
      values = packed(value, 2).map(signed);
    assert.equal(values.length, types.length);
    assert(values.every((v) => v >= 0n));
    assert.equal(
      values[1],
      values[0] * period,
      "CPU sample weight differs from count times period",
    );
    const labels = messages(value, 3).map((label) => ({
      key: text(numeric(label, 1)),
      str: text(numeric(label, 2)),
      num: signed(numeric(label, 3)).toString(),
      numUnit: text(numeric(label, 4)),
      unknown: unknown(label, [1, 2, 3, 4]),
    }));
    return { locations, values: values.map(String), labels, unknown: unknown(value, [1, 2, 3]) };
  });
  const functions = messages(all, 5).map((f) => ({
    id: numeric(f, 1).toString(),
    name: text(numeric(f, 2)),
    systemName: text(numeric(f, 3)),
    filename: text(numeric(f, 4)),
    startLine: signed(numeric(f, 5)).toString(),
    unknown: unknown(f, [1, 2, 3, 4, 5]),
  }));
  const locations = messages(all, 4).map((l) => ({
    id: numeric(l, 1).toString(),
    mappingId: numeric(l, 2).toString(),
    address: numeric(l, 3).toString(),
    folded: numeric(l, 5) !== 0n,
    lines: messages(l, 4).map((line) => ({
      functionId: numeric(line, 1).toString(),
      line: signed(numeric(line, 2)).toString(),
      column: signed(numeric(line, 3)).toString(),
      unknown: unknown(line, [1, 2, 3]),
    })),
    unknown: unknown(l, [1, 2, 3, 4, 5]),
  }));
  const mappings = messages(all, 3).map((m) => ({
    id: numeric(m, 1).toString(),
    start: numeric(m, 2).toString(),
    limit: numeric(m, 3).toString(),
    offset: numeric(m, 4).toString(),
    file: text(numeric(m, 5)),
    buildId: text(numeric(m, 6)),
    hasFunctions: numeric(m, 7) !== 0n,
    hasFilenames: numeric(m, 8) !== 0n,
    hasLines: numeric(m, 9) !== 0n,
    hasInlineFrames: numeric(m, 10) !== 0n,
    unknown: unknown(m, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
  }));
  for (const entries of [functions, locations, mappings])
    assert.equal(new Set(entries.map((v) => v.id)).size, entries.length);
  const totals = types.map((_, index) =>
    samples.reduce((sum, sample) => sum + BigInt(sample.values[index]), 0n),
  );
  assert(totals[0] > 0n && totals[1] > 0n, "CPU attribution requires positive measured samples");
  const byFunction = new Map(functions.map((f) => [f.id, f])),
    byLocation = new Map(locations.map((l) => [l.id, l]));
  let unknownLeafCpu = 0n,
    anyUnknownCpu = 0n,
    lostCpu = 0n;
  for (const sample of samples) {
    const frames = sample.locations.flatMap((id) => {
      const location = byLocation.get(id);
      return location?.lines.length
        ? location.lines.map((line) => byFunction.get(line.functionId))
        : [undefined];
    });
    const cpu = BigInt(sample.values[1]);
    if (!frames.length || !frames[0]?.name) unknownLeafCpu += cpu;
    if (!frames.length || frames.some((f) => !f?.name)) anyUnknownCpu += cpu;
    if (frames.some((f) => f?.name === "runtime/pprof.lostProfileEvent")) lostCpu += cpu;
  }
  return {
    compressedBytes: compressed.length,
    uncompressedBytes: bytes.length,
    sampleTypes: types,
    sampleTotals: totals.map(String),
    samples,
    functions,
    locations,
    mappings,
    stringTable: strings,
    timeNanos: signed(numeric(all, 9)).toString(),
    durationNanos: signed(numeric(all, 10)).toString(),
    period: period.toString(),
    periodType: periodTypes,
    dropFrames: text(numeric(all, 7)),
    keepFrames: text(numeric(all, 8)),
    comments: packed(all, 13).map(text),
    defaultSampleType: text(numeric(all, 14)),
    unknownFields: unknown(all, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]),
    unknownLeafCpuNs: unknownLeafCpu.toString(),
    samplesWithAnyUnknownFrameCpuNs: anyUnknownCpu.toString(),
    knownLostProfileEventCpuNs: lostCpu.toString(),
    lossScope:
      "Only the exact runtime/pprof.lostProfileEvent marker is classified; no overall zero-loss claim",
    sourceLineScope:
      "running-image hints only; vcs.modified=true; no exact upstream source ownership",
    cpuScope: "process CPU samples, not wall time/instructions/call counts",
  };
}

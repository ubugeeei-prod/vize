/** Private memory-only pprof view; the existing CPU reader remains unchanged. */
import assert from "node:assert/strict";
import { gunzipSync } from "node:zlib";
import {
  fields,
  messages,
  numeric,
  packed,
  signed,
  unknown,
  type Field,
} from "./cold-typechecker-scale-memory-wire.ts";

const expectedTypes = new Map([
  ["alloc_objects", "count"],
  ["alloc_space", "bytes"],
  ["inuse_objects", "count"],
  ["inuse_space", "bytes"],
]);
function known(value: Field[], tags: number[]): void {
  assert.deepEqual(unknown(value, tags), [], "unknown protobuf field refused");
}
function boolean(value: Field[], tag: number): boolean {
  const number = numeric(value, tag);
  assert(number === 0n || number === 1n, "invalid protobuf boolean");
  return number !== 0n;
}

export const memoryProfileLimits = {
  compressedBytes: 16 * 1024 * 1024,
  decodedBytes: 64 * 1024 * 1024,
} as const;

export function decodeMemoryProfile(compressed: Buffer) {
  assert(
    compressed.length > 0 && compressed.length <= memoryProfileLimits.compressedBytes,
    "compressed cap",
  );
  const bytes = gunzipSync(compressed, { maxOutputLength: memoryProfileLimits.decodedBytes });
  const all = fields(bytes);
  known(all, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
  const strings = all
    .filter((f) => f.tag === 6)
    .map((f) => {
      assert(f.wire === 2 && Buffer.isBuffer(f.value));
      return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(f.value);
    });
  assert.equal(strings[0], "");
  const text = (value: bigint): string => {
    const index = Number(value);
    assert(
      Number.isSafeInteger(index) && index >= 0 && index < strings.length,
      "dangling string index",
    );
    return strings[index]!;
  };
  const valueType = (value: Field[]) => {
    known(value, [1, 2]);
    return { type: text(numeric(value, 1)), unit: text(numeric(value, 2)) };
  };
  const sampleTypes = messages(all, 1).map(valueType);
  assert.equal(sampleTypes.length, 4, "exact four memory indices required");
  assert.equal(new Set(sampleTypes.map((value) => value.type)).size, 4);
  for (const value of sampleTypes) {
    assert.equal(expectedTypes.get(value.type), value.unit, "memory type/unit mismatch");
  }
  const periodType = messages(all, 11).map(valueType);
  assert.deepEqual(periodType, [{ type: "space", unit: "bytes" }]);
  const period = signed(numeric(all, 12));
  assert(period > 0n, "positive recorded memory sampling rate required");
  const samples = messages(all, 2).map((value) => {
    known(value, [1, 2, 3]);
    const locations = packed(value, 1).map(String);
    const values = packed(value, 2).map(signed);
    assert.equal(values.length, sampleTypes.length, "sample value arity");
    assert(
      values.every((value) => value >= 0n),
      "negative memory weight refused",
    );
    assert(locations.length > 0, "sample without allocation stack");
    const labels = messages(value, 3).map((label) => {
      known(label, [1, 2, 3, 4]);
      return {
        key: text(numeric(label, 1)),
        str: text(numeric(label, 2)),
        num: signed(numeric(label, 3)).toString(),
        numUnit: text(numeric(label, 4)),
      };
    });
    return { locations, values: values.map(String), labels };
  });
  assert(samples.length > 0);
  const functions = messages(all, 5).map((value) => {
    known(value, [1, 2, 3, 4, 5]);
    return {
      id: numeric(value, 1).toString(),
      name: text(numeric(value, 2)),
      systemName: text(numeric(value, 3)),
      filename: text(numeric(value, 4)),
      startLine: signed(numeric(value, 5)).toString(),
    };
  });
  const locations = messages(all, 4).map((value) => {
    known(value, [1, 2, 3, 4, 5]);
    return {
      id: numeric(value, 1).toString(),
      mappingId: numeric(value, 2).toString(),
      address: numeric(value, 3).toString(),
      folded: boolean(value, 5),
      lines: messages(value, 4).map((line) => {
        known(line, [1, 2, 3]);
        return {
          functionId: numeric(line, 1).toString(),
          line: signed(numeric(line, 2)).toString(),
          column: signed(numeric(line, 3)).toString(),
        };
      }),
    };
  });
  const mappings = messages(all, 3).map((value) => {
    known(value, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    return {
      id: numeric(value, 1).toString(),
      start: numeric(value, 2).toString(),
      limit: numeric(value, 3).toString(),
      offset: numeric(value, 4).toString(),
      file: text(numeric(value, 5)),
      buildId: text(numeric(value, 6)),
      hasFunctions: boolean(value, 7),
      hasFilenames: boolean(value, 8),
      hasLines: boolean(value, 9),
      hasInlineFrames: boolean(value, 10),
    };
  });
  for (const entries of [functions, locations, mappings]) {
    assert(
      entries.every((value) => BigInt(value.id) > 0n),
      "nonpositive identity",
    );
    assert.equal(
      new Set(entries.map((value) => value.id)).size,
      entries.length,
      "duplicate protobuf identity",
    );
  }
  const byFunction = new Map(functions.map((value) => [value.id, value]));
  const byLocation = new Map(locations.map((value) => [value.id, value]));
  const byMapping = new Map(mappings.map((value) => [value.id, value]));
  for (const location of locations) {
    assert(
      location.mappingId === "0" || byMapping.has(location.mappingId),
      "dangling mapping reference",
    );
    assert(location.lines.length > 0, "unresolved location refused");
    for (const line of location.lines) {
      assert(byFunction.has(line.functionId), "dangling function reference");
    }
  }
  for (const sample of samples) {
    for (const id of sample.locations) {
      assert(byLocation.has(id), "dangling sample location");
    }
  }
  const totals = sampleTypes.map((_, index) =>
    samples.reduce((sum, sample) => sum + BigInt(sample.values[index]!), 0n),
  );
  const flat = new Map<string, bigint[]>();
  const cumulative = new Map<string, bigint[]>();
  const add = (table: Map<string, bigint[]>, name: string, values: string[]) => {
    const row = table.get(name) ?? sampleTypes.map(() => 0n);
    values.forEach((value, index) => {
      row[index] = row[index]! + BigInt(value);
    });
    table.set(name, row);
  };
  const sampleFrames = samples.map((sample) =>
    sample.locations.flatMap((id) =>
      byLocation.get(id)!.lines.map((line) => byFunction.get(line.functionId)!.name),
    ),
  );
  samples.forEach((sample, index) => {
    const names = sampleFrames[index]!;
    add(flat, names[0]!, sample.values);
    for (const name of new Set(names)) add(cumulative, name, sample.values);
  });
  const table = (value: Map<string, bigint[]>) =>
    [...value]
      .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
      .map(([name, values]) => ({ name, values: values.map(String) }));
  const defaultSampleType = text(numeric(all, 14));
  assert(defaultSampleType === "" || expectedTypes.has(defaultSampleType));
  return {
    compressedBytes: compressed.length,
    uncompressedBytes: bytes.length,
    sampleTypes,
    sampleTotals: totals.map(String),
    samples,
    functions,
    locations,
    mappings,
    stringTable: strings,
    period: period.toString(),
    periodType,
    timeNanos: signed(numeric(all, 9)).toString(),
    durationNanos: signed(numeric(all, 10)).toString(),
    dropFrames: text(numeric(all, 7)),
    keepFrames: text(numeric(all, 8)),
    comments: packed(all, 13).map(text),
    defaultSampleType,
    flat: table(flat),
    cumulative: table(cumulative),
    weightScope:
      "sampled/scaled profile weights; each sample counted once in totals; inclusive rows overlap",
    inuseScope:
      "recorded sample window/last completed GC, not peak-live heap, dominators or forced-GC proof",
    sourceScope: "actual image function hints; exact modified native source remains unjoined",
  };
}

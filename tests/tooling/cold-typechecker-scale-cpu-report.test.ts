import assert from "node:assert/strict";
import { test } from "node:test";
import { gzipSync } from "node:zlib";
import { decodeCpuProfile } from "../../tools/benchmarks/scripts/cold-typechecker-scale-cpu-proto.ts";
import { parseRawCpuSamples } from "../../tools/benchmarks/scripts/cold-typechecker-scale-cpu-report.ts";

function uint(value: number | bigint): Buffer {
  let remaining = BigInt(value);
  const bytes: number[] = [];
  do {
    const byte = Number(remaining & 127n);
    remaining >>= 7n;
    bytes.push(byte | (remaining ? 128 : 0));
  } while (remaining);
  return Buffer.from(bytes);
}
const scalar = (tag: number, value: number | bigint) => Buffer.concat([uint(tag * 8), uint(value)]);
const message = (tag: number, bytes: Buffer) =>
  Buffer.concat([uint(tag * 8 + 2), uint(bytes.length), bytes]);
const cat = (...bytes: Buffer[]) => Buffer.concat(bytes);
const valueType = (type: number, unit: number) => cat(scalar(1, type), scalar(2, unit));
function fixture(
  options: { weight?: number; period?: number; periodType?: number; zero?: boolean } = {},
) {
  const period = options.period ?? 10_000_000;
  const strings = [
    "",
    "samples",
    "count",
    "cpu",
    "nanoseconds",
    "main.work",
    "runtime/pprof.lostProfileEvent",
    "source.go",
    "image",
    "build",
    "root",
    "label",
    "value",
  ];
  const sample = (location: number, count: number) =>
    message(
      2,
      cat(
        message(1, uint(location)),
        message(
          2,
          cat(
            uint(options.zero ? 0 : count),
            uint(options.zero ? 0 : (options.weight ?? count * period)),
          ),
        ),
        message(3, cat(scalar(1, 11), scalar(2, 12))),
      ),
    );
  const func = (id: number, name: number) =>
    message(5, cat(scalar(1, id), scalar(2, name), scalar(3, name), scalar(4, 7), scalar(5, 42)));
  const line = (id: number) => message(4, cat(scalar(1, id), scalar(2, 43)));
  const location = (id: number, ...functions: number[]) =>
    message(4, cat(scalar(1, id), scalar(2, 1), scalar(3, 4096 + id), ...functions.map(line)));
  return cat(
    message(1, valueType(1, 2)),
    message(1, valueType(3, 4)),
    sample(1, 2),
    sample(2, 1),
    sample(3, 1),
    sample(99, 1),
    message(
      3,
      cat(scalar(1, 1), scalar(2, 4096), scalar(3, 8192), scalar(5, 8), scalar(6, 9), scalar(7, 1)),
    ),
    location(1, 1, 4),
    location(2, 2),
    location(3, 3),
    func(1, 5),
    func(2, 6),
    func(3, 0),
    func(4, 10),
    ...strings.map((s) => message(6, Buffer.from(s))),
    scalar(9, 123),
    scalar(10, 100_000_000),
    message(11, valueType(options.periodType ?? 3, 4)),
    scalar(12, period),
    scalar(14, 3),
    scalar(15, 77),
  );
}
test("full CPU denominator preserves inline frames, labels, unresolved weights and exact known loss marker", () => {
  const profile = decodeCpuProfile(gzipSync(fixture()));
  assert.deepEqual(profile.sampleTotals, ["5", "50000000"]);
  assert.equal(profile.samples.length, 4);
  assert.deepEqual(profile.samples[0], {
    locations: ["1"],
    values: ["2", "20000000"],
    labels: [{ key: "label", str: "value", num: "0", numUnit: "", unknown: [] }],
    unknown: [],
  });
  assert.deepEqual(
    profile.locations[0].lines.map((line) => line.functionId),
    ["1", "4"],
  );
  assert.equal(profile.functions.length, 4);
  assert.equal(profile.mappings.length, 1);
  assert.equal(profile.period, "10000000");
  assert.equal(profile.durationNanos, "100000000");
  assert.equal(profile.unknownLeafCpuNs, "20000000");
  assert.equal(profile.samplesWithAnyUnknownFrameCpuNs, "20000000");
  assert.equal(profile.knownLostProfileEventCpuNs, "10000000");
  assert.deepEqual(profile.unknownFields, [{ tag: 15, wire: 0, value: "77" }]);
});
test("CPU reader refuses overflow, truncated fields, invalid string wires and unmeasured or inconsistent samples", () => {
  for (const bytes of [
    cat(fixture(), Buffer.from([128])),
    cat(fixture(), Buffer.from([128, 128, 128, 128, 128, 128, 128, 128, 128, 2])),
    cat(fixture(), scalar(6, 0)),
    cat(fixture(), Buffer.from([0])),
    fixture({ period: 0 }),
    fixture({ periodType: 1 }),
    fixture({ weight: 1 }),
    fixture({ zero: true }),
  ])
    assert.throws(() => decodeCpuProfile(gzipSync(bytes)));
});
const raw =
  "PeriodType: cpu nanoseconds\nSamples:\nsamples/count cpu/nanoseconds[dflt]\n          2   20000000: 1 2\n                label:[value]\n          1   10000000: 3\nLocations\n1: address function\nMappings\n";
test("Go full raw reader retains ordered weights/location IDs and label text", () => {
  assert.deepEqual(parseRawCpuSamples(raw), {
    sampleTypes: ["samples/count", "cpu/nanoseconds"],
    totals: ["3", "30000000"],
    samples: [
      {
        values: ["2", "20000000"],
        locations: ["1", "2"],
        rawLabels: ["                label:[value]"],
      },
      { values: ["1", "10000000"], locations: ["3"], rawLabels: [] },
    ],
  });
});
test("unknown raw formats, changed sample types and zero CPU do not qualify", () => {
  for (const value of [
    raw.replace("2   20000000", "unknown"),
    raw.replace("cpu/nanoseconds", "wall/nanoseconds"),
    "Samples:\nsamples/count cpu/nanoseconds\n 0 0: 1\nLocations\n",
    "Samples:\nsamples/count cpu/nanoseconds\nLocations\n",
  ])
    assert.throws(() => parseRawCpuSamples(value));
});

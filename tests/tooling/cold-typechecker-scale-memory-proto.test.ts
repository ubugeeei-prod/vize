import assert from "node:assert/strict";
import test from "node:test";
import { gzipSync } from "node:zlib";
import {
  decodeMemoryProfile,
  memoryProfileLimits,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-memory-proto.ts";

const uint = (value: bigint | number): Buffer => {
  let n = BigInt.asUintN(64, BigInt(value));
  const bytes: number[] = [];
  do {
    bytes.push(Number(n & 127n) | (n > 127n ? 128 : 0));
    n >>= 7n;
  } while (n);
  return Buffer.from(bytes);
};
const scalar = (tag: number, value: bigint | number): Buffer =>
  Buffer.concat([uint(tag * 8), uint(value)]);
const message = (tag: number, body: Buffer): Buffer =>
  Buffer.concat([uint(tag * 8 + 2), uint(body.length), body]);
const list = (tag: number, values: number[]): Buffer =>
  message(tag, Buffer.concat(values.map(uint)));
type Options = {
  types?: [number, number][];
  values?: number[];
  sampleLocation?: number;
  lineFunction?: number;
  mapping?: number;
  functionName?: number;
  extraRoot?: Buffer;
  extraFunction?: Buffer;
  extraSample?: Buffer;
  duplicateLocation?: boolean;
  noLines?: boolean;
  stringBytes?: Buffer;
  periodType?: [number, number];
  period?: number;
};
function fixture(options: Options = {}): Buffer {
  const strings = [
    "",
    "alloc_objects",
    "count",
    "alloc_space",
    "bytes",
    "inuse_objects",
    "inuse_space",
    "space",
    "fnA",
    "fnB",
    "source.go",
    "label",
    "雪🌸",
  ];
  const type = (tag: number, [name, unit]: [number, number]) =>
    message(tag, Buffer.concat([scalar(1, name), scalar(2, unit)]));
  const location = (id: number, fn: number) =>
    message(
      4,
      Buffer.concat([
        scalar(1, id),
        scalar(2, options.mapping ?? 1),
        scalar(3, 4096 + id),
        ...(options.noLines
          ? []
          : [message(4, Buffer.concat([scalar(1, fn), scalar(2, 10), scalar(3, 2)]))]),
      ]),
    );
  const raw = Buffer.concat([
    ...(
      options.types ?? [
        [1, 2],
        [3, 4],
        [5, 2],
        [6, 4],
      ]
    ).map((value) => type(1, value)),
    message(
      2,
      Buffer.concat([
        list(1, [options.sampleLocation ?? 1, 2, 1]),
        list(2, options.values ?? [2, 200, 1, 100]),
        message(3, Buffer.concat([scalar(1, 11), scalar(2, 12), scalar(3, 2), scalar(4, 4)])),
        options.extraSample ?? Buffer.alloc(0),
      ]),
    ),
    message(2, Buffer.concat([list(1, [2]), list(2, [3, 300, 1, 100])])),
    message(
      3,
      Buffer.concat([
        scalar(1, 1),
        scalar(2, 4096),
        scalar(3, 8192),
        scalar(5, 10),
        scalar(7, 1),
        scalar(8, 1),
        scalar(9, 1),
        scalar(10, 1),
      ]),
    ),
    location(1, options.lineFunction ?? 1),
    location(2, 2),
    ...(options.duplicateLocation ? [location(1, 1)] : []),
    message(
      5,
      Buffer.concat([
        scalar(1, 1),
        scalar(2, options.functionName ?? 8),
        scalar(3, 8),
        scalar(4, 10),
        scalar(5, 1),
        options.extraFunction ?? Buffer.alloc(0),
      ]),
    ),
    message(5, Buffer.concat([scalar(1, 2), scalar(2, 9), scalar(3, 9), scalar(4, 10)])),
    ...strings.map((value, index) =>
      message(6, index === 12 && options.stringBytes ? options.stringBytes : Buffer.from(value)),
    ),
    scalar(9, 123456789),
    scalar(10, 0),
    type(11, options.periodType ?? [7, 4]),
    scalar(12, options.period ?? 524288),
    list(13, [12]),
    scalar(14, 3),
    options.extraRoot ?? Buffer.alloc(0),
  ]);
  return gzipSync(raw);
}

let serial = 0;
const yes = (run: () => void) => test("whole memory positive " + ++serial, run);
const no = (run: () => void) => test("whole memory refusal " + ++serial, () => assert.throws(run));
yes(() => {
  const result = decodeMemoryProfile(fixture());
  assert.deepEqual(result.sampleTotals, ["5", "500", "2", "200"]);
  assert.equal(result.period, "524288");
  assert.deepEqual(result.periodType, [{ type: "space", unit: "bytes" }]);
  assert.equal(result.timeNanos, "123456789");
  assert.equal(result.durationNanos, "0");
  assert.equal(result.defaultSampleType, "alloc_space");
  assert.deepEqual(result.comments, ["雪🌸"]);
  assert.deepEqual(result.samples[0]!.labels, [
    { key: "label", str: "雪🌸", num: "2", numUnit: "bytes" },
  ]);
  assert.deepEqual(
    result.cumulative.find((value) => value.name === "fnA")!.values,
    ["2", "200", "1", "100"],
    "recursive frame must not multiply sample weights",
  );
  assert.deepEqual(result.cumulative.find((value) => value.name === "fnB")!.values, [
    "5",
    "500",
    "2",
    "200",
  ]);
  assert.deepEqual(result.flat.find((value) => value.name === "fnB")!.values, [
    "3",
    "300",
    "1",
    "100",
  ]);
});
yes(() => {
  const result = decodeMemoryProfile(
    fixture({
      types: [
        [3, 4],
        [1, 2],
        [6, 4],
        [5, 2],
      ],
    }),
  );
  assert.deepEqual(
    result.sampleTypes.map((value) => value.type),
    ["alloc_space", "alloc_objects", "inuse_space", "inuse_objects"],
  );
  assert.deepEqual(result.sampleTotals, ["5", "500", "2", "200"]);
});
for (const options of [
  {
    types: [
      [1, 4],
      [3, 4],
      [5, 2],
      [6, 4],
    ],
  },
  {
    types: [
      [1, 2],
      [3, 4],
      [5, 2],
      [5, 2],
    ],
  },
  {
    types: [
      [1, 2],
      [3, 4],
      [5, 2],
    ],
  },
  {
    types: [
      [8, 2],
      [3, 4],
      [5, 2],
      [6, 4],
    ],
  },
  { values: [2, 200, 1] },
  { values: [2, 200, 1, 100, 9] },
  { values: [-1, 200, 1, 100] },
  { sampleLocation: 99 },
  { lineFunction: 99 },
  { mapping: 99 },
  { functionName: 99 },
  { duplicateLocation: true },
  { noLines: true },
  { stringBytes: Buffer.from([0xff]) },
  { periodType: [7, 2] },
  { period: 0 },
  { extraRoot: scalar(99, 1) },
  { extraFunction: scalar(99, 1) },
  { extraSample: scalar(99, 1) },
] as Options[])
  no(() => decodeMemoryProfile(fixture(options)));
no(() => decodeMemoryProfile(fixture().subarray(0, -5)));
no(() => decodeMemoryProfile(gzipSync(Buffer.from([0x80]))));
no(() => decodeMemoryProfile(gzipSync(Buffer.from([0x0a, 0xff]))));
no(() => decodeMemoryProfile(Buffer.alloc(memoryProfileLimits.compressedBytes + 1)));

no(() => decodeMemoryProfile(gzipSync(Buffer.alloc(memoryProfileLimits.decodedBytes + 1))));

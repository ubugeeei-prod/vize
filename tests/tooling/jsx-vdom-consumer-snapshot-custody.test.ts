/** Finite complete hosted outputs retain all original inputs and expected bytes. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { nativeVdomConsumers } from "../../npm/builder/rspack/src/test/jsx-vdom-child-values.ts";
const root = new URL("../../", import.meta.url);
const hash = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const read = (path: string) => fs.readFileSync(new URL(path, root));
const path = "tests/_fixtures/differential/jsx/child-values-6890-vdom-consumer-successors.json";
const bytes = read(path);
const data = JSON.parse(bytes.toString()) as {
  schema: number;
  head: string;
  originalInputsChanged: boolean;
  runtimeCredit: boolean;
  performanceCredit: boolean;
  wholeCorpusQualified: boolean;
  archives: Record<string, { source: string; run: number; artifact: number; sha256: string }>;
  historicalFiles: { path: string; sha256: string }[];
  historicalLiterals: {
    path: string;
    function: string;
    binding: string;
    literalSha256: string;
  }[];
  rows: {
    kind: string;
    original?: string;
    originalSha256?: string;
    successor: string;
    successorSha256: string;
    archive: string;
    member: string;
  }[];
};
function verify(reader: (path: string) => Buffer) {
  assert.equal(hash(bytes), "03cc702afa07dd83be08e8cebd1eaf08d226db65aa35608d5422e0e3293a9b3a");
  assert.equal(data.schema, 1);
  assert.equal(data.head, "11004c61eff840a54965a569da47cec245ae75b6");
  for (const key of [
    "originalInputsChanged",
    "runtimeCredit",
    "performanceCredit",
    "wholeCorpusQualified",
  ] as const)
    assert.equal(data[key], false);
  assert.deepEqual(
    Object.entries(data.archives).map(([key, value]) => [
      key,
      value.run,
      value.artifact,
      value.sha256,
      value.source,
    ]),
    [
      [
        "native",
        41222652846,
        13050648010,
        "193525ebfb74d0ec36de52b3aa1586106f656b065f0d9ad87092b0eeee521c26",
        data.head,
      ],
      [
        "rspack",
        41222653156,
        13049894272,
        "efaca10197f4df1b531f4a1b21be10148d3a9197c45bbb4feeed5ba6a6b6df5f",
        "b66d02c20f259cd77dd495d3c30ceb2c13d9f13b",
      ],
    ],
  );
  assert.equal(data.rows.length, 12);
  assert.deepEqual(
    ["native-snapshot", "rust-adapter", "scoped-slot", "rspack"].map(
      (kind) => data.rows.filter((row) => row.kind === kind).length,
    ),
    [6, 2, 1, 3],
  );
  assert.equal(new Set(data.rows.map((row) => row.successor)).size, 12);
  for (const row of data.rows) {
    assert.equal(hash(reader(row.successor)), row.successorSha256);
    assert.ok(data.archives[row.archive]);
    assert.ok(row.member.endsWith(row.kind === "native-snapshot" ? ".snap.new" : ".json"));
    if (row.original) {
      assert.equal(hash(reader(row.original)), row.originalSha256);
      assert.equal(row.successor, row.original.replace(/\.snap$/, "_vdom_child_values_6890.snap"));
      assert.notEqual(row.originalSha256, row.successorSha256);
    }
  }
  for (const row of data.historicalFiles) assert.equal(hash(reader(row.path)), row.sha256);
  for (const row of data.historicalLiterals) {
    const source = reader(row.path)
      .toString()
      .split("fn " + row.function + "()")[1]!;
    const literal = source.match(
      new RegExp(row.binding + '\\s*=\\s*("(?:\\\\[\\s\\S]|[^"\\\\])*")\\s*;'),
    )?.[1];
    assert.ok(literal);
    assert.equal(hash(literal), row.literalSha256);
  }
}
test("twelve actual whole consumer outputs preserve every historical expected value", () =>
  verify(read));
test("custody rejects changed expected outputs, predecessor snapshots, original sources and old literals", () => {
  const paths = new Set([
    ...data.rows.flatMap((row) => [row.successor, ...(row.original ? [row.original] : [])]),
    ...data.historicalFiles.map((row) => row.path),
    ...data.historicalLiterals.map((row) => row.path),
  ]);
  for (const path of paths)
    assert.throws(() =>
      verify((file) =>
        file === path
          ? Buffer.from(
              data.historicalLiterals.some((row) => row.path === file)
                ? read(file).toString().replaceAll("toDisplayString", "changedDisplayString")
                : read(file).toString() + "changed",
            )
          : read(file),
      ),
    );
});
test("the Rspack admission retains all three complete captured values and the actual Node DTO boundary", () => {
  const folder = "tests/_fixtures/differential/jsx/child-values-6890-vdom-consumers/";
  for (const [key, packet] of Object.entries(nativeVdomConsumers))
    assert.deepEqual(packet, JSON.parse(read(folder + "native-vdom-" + key + ".json").toString()));
  const bundle = nativeVdomConsumers.bundle;
  assert.equal(bundle.mapOwnProperty, false);
  assert.equal(bundle.mapIsUndefined, true);
  assert.equal(Object.hasOwn(bundle.nativeDto, "map"), false, "the actual Node DTO omitted map");
  assert.equal(bundle.nativeDto.code, bundle.loadedModule);
  assert.deepEqual(bundle.nativeDto.errors, []);
  assert.deepEqual(bundle.nativeDto.warnings, []);
  assert.deepEqual(bundle.nativeDto.scopedStyles, []);
  assert.equal(
    nativeVdomConsumers.module.observed.module,
    nativeVdomConsumers.module.nativeDto.code,
  );
  assert.equal(nativeVdomConsumers.map.observed, nativeVdomConsumers.map.nativeDto.map);
});

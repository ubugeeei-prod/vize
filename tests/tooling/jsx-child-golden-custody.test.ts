import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";

type Json = null | string | number | boolean | Json[] | { [key: string]: Json };
type RecordRow = {
  kind: string;
  previous: string;
  previousSha256: string;
  successor: string;
  successorSha256: string;
  changedPointers?: string[];
  source?: string;
  sourceSha256?: string;
  runtime: boolean;
  nativeHandled?: number;
};
const root = new URL("../../", import.meta.url);
type Manifest = {
  schema: number;
  issue: number;
  source: string;
  capture: unknown;
  qualification: unknown;
  history?: { path: string; sha256: string }[];
  records: RecordRow[];
};
const manifest = JSON.parse(
  fs.readFileSync(
    new URL("tests/_fixtures/differential/jsx/child-values-6890-golden-successors.json", root),
    "utf8",
  ),
) as Manifest;
const read = (file: string) => fs.readFileSync(new URL(file, root));
const hash = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const parse = (file: string) => JSON.parse(read(file).toString("utf8")) as Json;

function differences(before: Json, after: Json, prefix = ""): string[] {
  if (JSON.stringify(before) === JSON.stringify(after)) return [];
  if (Array.isArray(before) && Array.isArray(after)) {
    assert.equal(before.length, after.length);
    return before.flatMap((value, index) => differences(value, after[index], `${prefix}/${index}`));
  }
  if (
    before &&
    after &&
    typeof before === "object" &&
    typeof after === "object" &&
    !Array.isArray(before) &&
    !Array.isArray(after)
  ) {
    assert.deepEqual(Object.keys(before).sort(), Object.keys(after).sort());
    return Object.keys(before).flatMap((key) =>
      differences(before[key], after[key], `${prefix}/${key}`),
    );
  }
  return [prefix];
}

function check(row: RecordRow, previous: Json, successor: Json): void {
  assert.equal(row.runtime, false, "generated packet custody never becomes runtime qualification");
  const changed = differences(previous, successor);
  assert.deepEqual(changed, row.changedPointers);
  if (row.kind === "mapped-roots") {
    assert.deepEqual(changed, [
      "/0/code",
      "/0/map/mappings",
      "/2/code",
      "/2/map/mappings",
      "/4/code",
      "/4/map/mappings",
    ]);
  } else if (row.kind === "admission") {
    assert.deepEqual(changed, [
      "/4/components/0/code",
      "/4/module",
      "/4/moduleMap/mappings",
      "/5/components/0/code",
      "/5/components/1/code",
      "/5/module",
      "/5/moduleMap/mappings",
      "/7/components/0/code",
      "/10/components/0/code",
      "/11/components/0/code",
      "/12/components/0/code",
    ]);
    assert.ok(Array.isArray(previous) && Array.isArray(successor));
    assert.equal(previous.length, 28, "all original admission and refusal cases remain present");
  } else {
    assert.equal(row.kind, "refusal");
    const expected =
      row.source === "AfsStepperDialog.stories.tsx.txt"
        ? ["/components/3/code", "/components/4/code"]
        : ["/components/0/code"];
    assert.deepEqual(changed, expected, "only the exact observed child-code pointers may change");
    assert.equal(row.nativeHandled, 0);
    assert.ok(successor && typeof successor === "object" && !Array.isArray(successor));
    assert.equal(successor.module, "");
    assert.equal(successor.moduleMap, null);
  }
}

test("JSX child successors retain original goldens, exact observed pointers and qualification limits", () => {
  assert.equal(manifest.schema, 1);
  assert.equal(manifest.issue, 6890);
  assert.equal(manifest.source, "735d38e89d35573b5c12efa16b63fb91be31b7fc");
  assert.deepEqual(manifest.capture, {
    run: 41188284667,
    attempt: 1,
    artifact: 13037114422,
    bytes: 295837,
    sha256: "eeeb89ceff0b7abd83108850343936383523ffa73977918b724d19596a865ef5",
  });
  assert.deepEqual(manifest.history, [
    {
      path: "tests/_fixtures/differential/jsx/child-values-6890-golden-successors.f7c306d.json",
      sha256: "8ac88305e4ad75ba58a3bddda61ceefa93a5f6ce522d7edae6df90ea6a3895f9",
    },
  ]);
  assert.equal(hash(read(manifest.history![0].path)), manifest.history![0].sha256);
  const historical = JSON.parse(read(manifest.history![0].path).toString()) as Manifest;
  assert.equal(historical.source, "f7c306ddc05377bcaec46d502d995f30f525aa6b");
  assert.deepEqual(historical.capture, {
    run: 41185002277,
    attempt: 1,
    artifact: 13036597757,
    bytes: 294242,
    sha256: "9c54b3eff929142bf0a16c211f5eb5ba3b39a2ec721daa2b404d1aca8fef375a",
  });
  for (const packet of [historical, manifest]) {
    assert.deepEqual(packet.qualification, {
      structural: true,
      runtime: false,
      wholeCorpusQualified: false,
      issue6890Complete: false,
    });
    assert.equal(packet.records.length, 11);
    assert.equal(new Set(packet.records.map((row) => row.previous)).size, 11);
    assert.equal(new Set(packet.records.map((row) => row.successor)).size, 11);
    for (const row of packet.records) {
      assert.equal(
        hash(read(row.previous)),
        row.previousSha256,
        "historical complete bytes are immutable",
      );
      assert.equal(
        hash(read(row.successor)),
        row.successorSha256,
        "exact authenticated successor bytes",
      );
      assert.equal(row.runtime, false);
      if (row.kind === "snapshot") {
        const before = read(row.previous).toString(),
          after = read(row.successor).toString();
        assert.ok(before.startsWith("---\n") && after.startsWith("---\n"));
        assert.ok(!before.includes("_vizeJsxChildren("));
        assert.ok(after.includes("_vizeJsxChildren("));
        if (packet === manifest) assert.ok(after.includes("_createTextNode"));
      } else {
        check(row, parse(row.previous), parse(row.successor));
        if (row.source) {
          const source = new URL(row.source, new URL(row.previous, root));
          assert.equal(hash(fs.readFileSync(source)), row.sourceSha256);
        }
      }
    }
  }
});

test("child-code custody rejects unrelated metadata, broadened pointers and runtime promotion", () => {
  for (const row of manifest.records.filter((row) => row.kind !== "snapshot")) {
    const previous = parse(row.previous),
      successor = parse(row.successor);
    assert.throws(
      () => check({ ...row, runtime: true }, previous, successor),
      assert.AssertionError,
    );
    assert.throws(
      () => check({ ...row, changedPointers: ["/components"] }, previous, successor),
      assert.AssertionError,
    );
    if (row.kind === "refusal") {
      const changed = structuredClone(successor);
      assert.ok(changed && typeof changed === "object" && !Array.isArray(changed));
      changed.diagnostics = "[]";
      assert.throws(() => check(row, previous, changed), assert.AssertionError);
      changed.diagnostics = (successor as { [key: string]: Json }).diagnostics;
      changed.module = "export const escaped = true;";
      assert.throws(() => check(row, previous, changed), assert.AssertionError);
    }
  }
});

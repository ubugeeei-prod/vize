import assert from "node:assert/strict";
import { test } from "node:test";
import {
  factory,
  codeEdits,
  compareProjection,
  comparePreamble,
  sharedInsertion,
  type Projection,
} from "../../tools/benchmarks/scripts/direct-ref-factory-parity.ts";
import {
  compareCli,
  expectedErrors,
  exactDebug,
} from "../../tools/benchmarks/scripts/direct-ref-factory-qualification.ts";
import { readFileSync } from "node:fs";

const before = `type $RefInstanceFactory = "authored";\nfunction __setup() {\n  ${factory}  var ref = (undefined as unknown as $RefInstanceFactory<typeof Child>)(Child)({value:1});\n}\n`;
const after = before
  .replace("  " + factory, "")
  .replace(
    " as $RefInstanceFactory<typeof Child>",
    ' as import("vize:virtual-ts").$RefInstanceFactory<typeof Child>',
  );
const packet = (code: string): Projection => ({
  file: "Parent.vue",
  source: "",
  hoisted: true,
  code,
  preRewrite: code,
  rows: [],
  semanticLinks: [],
  propDefaultKeyLinks: [],
  features: [],
  toAuthored: Array(Buffer.byteLength(code) + 1).fill(null),
  toGenerated: [null],
  importMapDebug: "identity",
  importForward: Array.from({ length: Buffer.byteLength(code) + 1 }, (_, i) => i),
  importReverse: Array.from({ length: Buffer.byteLength(code) + 1 }, (_, i) => i),
});
test("only the owned factory declaration and direct probe reference change", () => {
  assert.equal(codeEdits(before, after).uses, 1);
  assert.equal(compareProjection(packet(before), packet(after)), 1);
  assert.equal(
    compareProjection({ ...packet(before), hoisted: false }, { ...packet(before), hoisted: false }),
    0,
  );
});
test("changed authored aliases, unknown metadata, mapping cells, duplicate helpers and unused factories refuse", () => {
  assert.throws(() => codeEdits(before, after.replace('"authored"', '"other"')));
  assert.throws(() =>
    codeEdits(before.replace("  " + factory, "  " + factory + "  " + factory), after),
  );
  assert.throws(() =>
    codeEdits(before.replace("$RefInstanceFactory<typeof Child>", "Other<typeof Child>"), after),
  );
  assert.throws(() => compareProjection(packet(before), { ...packet(after), source: "mutated" }));
  const bad = packet(after);
  bad.importForward[0] = 1;
  assert.throws(() => compareProjection(packet(before), bad));
  const mapped = packet(before);
  mapped.rows = [
    {
      generated: { start: before.indexOf("  type"), end: before.indexOf("  type") + 10 },
      authored: { start: 0, end: 1 },
      subSpans: [],
    },
  ];
  assert.throws(() => compareProjection(mapped, packet(after)));
  assert.throws(() =>
    compareProjection({ ...packet(before), hoisted: false }, { ...packet(after), hoisted: false }),
  );
});
test("the full shared module insertion is exact and cannot admit extra or duplicate declarations", () => {
  const anchor = readFileSync(
    new URL(
      "../../crates/vize_canon/src/virtual_ts/helpers/declarations/shared-template-refs.d.ts.txt",
      import.meta.url,
    ),
    "utf8",
  );
  comparePreamble("prefix" + anchor + "suffix", "prefix" + anchor + sharedInsertion + "suffix");
  assert.throws(() =>
    comparePreamble("prefix" + anchor + "suffix", "prefix" + anchor + sharedInsertion + "suffixX"),
  );
  assert.throws(() => comparePreamble(anchor + anchor, anchor + sharedInsertion + anchor));
});
test("complete CLI parity refuses changed diagnostics, foreign fields and extra files", () => {
  const b = {
    errorCount: 0,
    files: [
      {
        file: "Parent.vue",
        diagnostics: [],
        virtualTs: before.replace("./Child.vue", "./Child.vue.ts"),
      },
    ],
  };
  // No previous .vue import is in this artificial program, so exercise the
  // unchanged population here; owned-edit admission is tested above.
  const same = {
    errorCount: 0,
    files: [{ file: "Plain.vue", diagnostics: [], virtualTs: "const n=1;\n" }],
  };
  assert.deepEqual(compareCli(same, structuredClone(same)), { activeFiles: 0, uses: 0 });
  assert.throws(() => compareCli(same, { ...same, errorCount: 1 }));
  assert.throws(() => compareCli(same, { ...same, foreign: true }));
  assert.throws(() => compareCli(same, { ...same, files: [...same.files, ...same.files] }));
  assert.throws(() => compareCli(b, b));
});
test("the actual pinned inputs own exact number and two NoAny negative positions", () => {
  const parent = readFileSync(
    new URL(
      "../_fixtures/differential/typechecker/direct-ref-factory/Parent.vue.txt",
      import.meta.url,
    ),
    "utf8",
  );
  assert.deepEqual(expectedErrors(parent, "number"), [
    {
      file: "src/Parent.vue",
      line: 17,
      column: 11,
      code: 2322,
      message: "Type 'number' is not assignable to type 'string'.",
    },
  ]);
  assert.deepEqual(expectedErrors(parent, "any"), [
    {
      file: "src/Parent.vue",
      line: 18,
      column: 11,
      code: 2322,
      message: "Type 'true' is not assignable to type 'false'.",
    },
    {
      file: "src/Parent.vue",
      line: 24,
      column: 11,
      code: 2322,
      message: "Type 'true' is not assignable to type 'false'.",
    },
  ]);
  assert.deepEqual(expectedErrors(parent, null), []);
});
test("debug output keeps absolute UTF8 source order, every body and exact output bytes", () => {
  const value = {
    errorCount: 0,
    files: [
      { file: "Z.vue", diagnostics: [], virtualTs: "Z" },
      { file: "A.vue", diagnostics: [], virtualTs: "A" },
    ],
  };
  assert.equal(
    exactDebug(value, "helper", "/project").toString(),
    "\n=== __vize_helpers.d.ts ===\nhelper\n\n=== /project/A.vue ===\nA\n\n=== /project/Z.vue ===\nZ\n",
  );
  assert.throws(() =>
    exactDebug({ ...value, files: [value.files[0], value.files[0]] }, "helper", "/project"),
  );
});

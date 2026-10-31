import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  canonicalMappedGenerated,
  selectSdk,
} from "../../tools/benchmarks/scripts/canon-slot-constructor-audit.ts";
import { slotCases } from "../../tools/benchmarks/scripts/canon-slot-constructor-cases.ts";
import {
  assertAuthoredNativeDiagnostics,
  assertCliDiagnosticContract,
} from "../../tools/benchmarks/scripts/canon-slot-constructor-display.ts";
import { authoredMappings } from "../../tools/benchmarks/scripts/canon-slot-constructor-real-project.ts";
import {
  isRecord,
  jsonValue,
} from "../../tools/benchmarks/scripts/canon-instance-directives-model.ts";

function observedDisplay(id: string, kind: "native" | "cli") {
  const certificate = jsonValue(
    fs.readFileSync(
      new URL(
        "../_fixtures/differential/typechecker/zero-argument-slot-constructor-11013/display-rename-11153.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  assert.ok(isRecord(certificate) && isRecord(certificate.cases));
  const entry = certificate.cases[id];
  assert.ok(isRecord(entry) && isRecord(entry[kind]));
  return entry[kind];
}

test("slot constructor audit canonicalizes only the certified required-slot helper rename", () => {
  assert.equal(
    canonicalMappedGenerated(
      '  const __vize_required_slots_0: $RequiredSlots<$SlotContract_0, { readonly "default": true; }> = {};\n',
    ),
    '  const __vize_required_slots_0: __VizeRequiredSlots<__VizeSlotContract_0, { readonly "default": true; }> = {};\n',
  );
  assert.equal(
    canonicalMappedGenerated(
      '      const __vize_required_slots_17: $RequiredSlots<$SlotContract_17, { readonly "before": true; }> = {};\n',
    ),
    '      const __vize_required_slots_17: __VizeRequiredSlots<__VizeSlotContract_17, { readonly "before": true; }> = {};\n',
  );
});

test("slot constructor audit retains unrelated generated text exactly", () => {
  for (const source of [
    "$RequiredSlots<$SlotContract_0, {}>",
    "const user_slots_0: $RequiredSlots<$SlotContract_0, {}> = {};",
    "const __vize_required_slots_0: $Other<$SlotContract_0, {}> = {};",
    "const __vize_required_slots_0: $RequiredSlots<$Other_0, {}> = {};",
  ])
    assert.equal(canonicalMappedGenerated(source), source);
});

test("real-project component mappings use the same narrow helper rename certificate", () => {
  const source =
      '  const __vize_required_slots_0: $RequiredSlots<$SlotContract_0, { readonly "before": true; }> = {};\n',
    span = {
      text: "QSplitter",
      start: 1,
      end: 10,
      length: 9,
      line: 1,
      column: 2,
      utf8Start: 1,
      utf8End: 10,
      coordinateSystem: "UTF-16",
    },
    result = { text: source, mappings: [[0, Buffer.byteLength(source), 1, 9, 1, 3]] };
  assert.deepEqual(authoredMappings(result, span), [
    {
      generated: canonicalMappedGenerated(source),
      original: 1,
      originalLength: 9,
      kind: 1,
      features: 3,
    },
  ]);
});

test("native helper display successors preserve the complete diagnostic and exact source", () => {
  const row = slotCases().find((entry) => entry.id === "missing-symbol"),
    observed = observedDisplay("missing-symbol", "native");
  assert.ok(row);
  assertAuthoredNativeDiagnostics(row, observed.base, observed.head);
  assert.ok(Array.isArray(observed.head) && isRecord(observed.head[0]));
  const diagnostic = observed.head[0];
  for (const change of [
    { code: 2345 },
    { file: "src/Other.vue" },
    { line: 13 },
    { column: 26 },
    { message: "Property 'another' does not exist on type '$TemplateInstance'." },
  ])
    assert.throws(() =>
      assertAuthoredNativeDiagnostics(row, observed.base, [{ ...diagnostic, ...change }]),
    );
  assert.throws(() => assertAuthoredNativeDiagnostics(row, observed.base, []));
  assert.throws(() =>
    assertAuthoredNativeDiagnostics(
      { ...row, source: row.source + "\n" },
      observed.base,
      observed.head,
    ),
  );
  assert.throws(() =>
    assertAuthoredNativeDiagnostics({ ...row, id: "other" }, observed.base, observed.head),
  );
});

test("CLI display successors require exact truncation, duplicate diagnostics and project metadata", () => {
  const row = slotCases().find((entry) => entry.id === "unknown-prop"),
    observed = observedDisplay("unknown-prop", "cli");
  assert.ok(row);
  assertCliDiagnosticContract(row, observed.base, observed.head);
  assert.ok(isRecord(observed.head) && isRecord(observed.head.result));
  const candidate = observed.head;
  const originalResult = observed.head.result;
  for (const result of [
    { ...originalResult, errorCount: 0 },
    { ...originalResult, programs: [] },
    { ...originalResult, files: [] },
  ])
    assert.throws(() => assertCliDiagnosticContract(row, observed.base, { ...candidate, result }));
  const withoutExactTruncation = JSON.parse(
    JSON.stringify(candidate).replaceAll("676 more", "677 more"),
  );
  assert.throws(() => assertCliDiagnosticContract(row, observed.base, withoutExactTruncation));
  assert.throws(() => assertCliDiagnosticContract(row, {}, candidate));
  assert.throws(() =>
    assertCliDiagnosticContract(
      { ...row, component: row.component + "\n" },
      observed.base,
      candidate,
    ),
  );
});

test("slot constructor audit selects the binary and declarations from one SDK image", (context) => {
  const root = fs.mkdtempSync(path.join(process.env.TMPDIR ?? "/tmp", "vize-slot-sdk-"));
  context.after(() => fs.rmSync(root, { recursive: true }));
  const project = path.join(root, "project"),
    installed = path.join(project, "node_modules/vize"),
    head = path.join(root, "head");
  fs.mkdirSync(installed, { recursive: true });
  fs.mkdirSync(head);
  fs.writeFileSync(
    path.join(installed, "package.json"),
    JSON.stringify({ typescript: { contentMapper: { exec: ["baseline", "content-mapper"] } } }),
  );
  for (const extension of ["ts", "mts", "cts"])
    fs.writeFileSync(path.join(head, `content-mapper-helpers.d.${extension}`), `head-${extension}`);
  fs.mkdirSync(path.join(head, "generated"));
  fs.writeFileSync(path.join(head, "generated/content-mapper-types.d.ts"), "head-generated-types");

  selectSdk(project, "/head/vize", head);

  assert.deepEqual(
    JSON.parse(fs.readFileSync(path.join(installed, "package.json"), "utf8")).typescript
      .contentMapper.exec,
    ["/head/vize", "content-mapper"],
  );
  for (const extension of ["ts", "mts", "cts"])
    assert.equal(
      fs.readFileSync(path.join(installed, `content-mapper-helpers.d.${extension}`), "utf8"),
      `head-${extension}`,
    );
  assert.equal(
    fs.readFileSync(path.join(installed, "generated/content-mapper-types.d.ts"), "utf8"),
    "head-generated-types",
  );
});

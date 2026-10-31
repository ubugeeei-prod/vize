import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  JSX_CHILD_ADMITTED_REVISION,
  JSX_CHILD_API_CASE_ID,
  newJsxChildApiCases,
} from "../../tools/support/compat/github/jsx-child-api-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import type compilerManifest from "../_fixtures/differential/compiler/manifest.json";

type Manifest = typeof compilerManifest;
type Rows = Manifest["jsxChildApiCases"];
type ReadFile = Parameters<typeof newJsxChildApiCases>[2];
const manifestPath = "tests/_fixtures/differential/compiler/manifest.json";
const manifest: Manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const rows = manifest.jsxChildApiCases;
const previous = structuredClone(manifest) as Omit<Manifest, "jsxChildApiCases"> & {
  jsxChildApiCases?: Rows;
};
delete previous.jsxChildApiCases;
const baseline = "be8aa8108652c10b7eb02d963e8862fcae6b542c";
// Squash merges do not retain the admitted branch's commit ancestry. These
// immutable unit-fixture bytes have the original provenance and exact digests;
// the production report still reads the actual comparison Git trees.
const historicalBytes = fs.readFileSync(
  "tests/_fixtures/differential/compiler/jsx-child-values-6890/historical-source-admission.json",
);
assert.equal(
  createHash("sha256").update(historicalBytes).digest("hex"),
  "4b34d12f5695532a31f480c8dab666cf29af25bdfc59871789bc727891263830",
);
const historical = JSON.parse(historicalBytes.toString()) as {
  schema: number;
  baselineRevision: string;
  admittedRevision: string;
  files: { side: "base" | "head"; path: string; sha256: string | null; base64: string | null }[];
};
assert.equal(historical.schema, 1);
assert.equal(historical.baselineRevision, baseline);
assert.equal(historical.admittedRevision, JSX_CHILD_ADMITTED_REVISION);
const files = new Map<string, Buffer | null>();
for (const entry of historical.files) {
  const key = `${entry.side}:${entry.path}`;
  assert(!files.has(key), "duplicate historical byte record");
  const bytes = entry.base64 === null ? null : Buffer.from(entry.base64, "base64");
  assert.equal(
    bytes === null ? null : createHash("sha256").update(bytes).digest("hex"),
    entry.sha256,
  );
  files.set(key, bytes);
}
assert.deepEqual(
  [...files.keys()].sort(),
  [
    ...rows[0].sources.flatMap((source) => [`base:${source.path}`, `head:${source.path}`]),
    ...[...rows[0].inputs, ...rows[0].witnesses].map((pin) => `head:${pin.path}`),
  ].sort(),
  "the exact historical source/input/witness closure is retained",
);
files.set(`base:${manifestPath}`, Buffer.from(JSON.stringify(previous)));
files.set(`head:${manifestPath}`, Buffer.from(JSON.stringify(manifest)));
const read: ReadFile = (side, path) => {
  const key = `${side}:${path}`;
  if (files.has(key)) return files.get(key)!;
  try {
    return fs.readFileSync(path);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return null;
    throw error;
  }
};
const paths = rows[0].sources.map((source) => source.path);
function report(override: ReadFile = read, changed = paths) {
  return analyzeLegacyFix({
    title: "fix(jsx): normalize Vapor expression children as DOM blocks",
    paths: changed,
    readFile: override,
  });
}

test("original complete JSX input and real runtime consumer register only this finite producer", () => {
  const actual = newJsxChildApiCases(undefined, rows, read);
  assert.deepEqual(actual.ids, [JSX_CHILD_API_CASE_ID]);
  assert.deepEqual([...actual.sources], paths);
  assert.equal(actual.sources.size, 18);
  assert.deepEqual(rows[0].qualification, {
    originalPhases: 10,
    qualifiedVaporPhases: 9,
    referenceConflictPhases: 1,
    wholeCorpusQualified: false,
    issue6890Complete: false,
  });
  assert.deepEqual(report().products, [
    { product: "compiler", state: "new-inputs-registered", cases: [JSX_CHILD_API_CASE_ID] },
  ]);
});
test("no missing registration, stale producer, or unrelated compiler source receives credit", () => {
  for (const current of [undefined, []])
    assert.throws(() => newJsxChildApiCases(undefined, current, read));
  assert.equal(
    report(read, [...paths, "crates/vize_atelier_sfc/src/unrelated.rs"]).products[0].state,
    "new-input-missing",
  );
  for (const source of rows[0].sources)
    for (const side of ["base", "head"] as const) {
      const mutant: ReadFile = (which, path) =>
        which === side && path === source.path
          ? Buffer.from("different actual producer")
          : read(which, path);
      assert.throws(() => newJsxChildApiCases(undefined, rows, mutant), `${side}:${source.path}`);
    }
});
test("all original inputs, reference conflict, browser baseline, and executable witnesses are byte-pinned", () => {
  for (const pin of [...rows[0].inputs, ...rows[0].witnesses])
    for (const replacement of [null, Buffer.from("changed bytes")]) {
      const mutant: ReadFile = (side, path) =>
        side === "head" && path === pin.path ? replacement : read(side, path);
      assert.throws(() => newJsxChildApiCases(undefined, rows, mutant), pin.path);
    }
});
test("future IDs, fake runtime selectors, reduced populations, and parity inflation are rejected", () => {
  for (const mutate of [
    (value: Rows) => {
      value[0].id += "-future";
    },
    (value: Rows) => {
      value[0].adapter = "generic-ssr";
    },
    (value: Rows) => {
      value[0].targets = ["ssr"];
    },
    (value: Rows) => {
      value[0].qualification.originalPhases = 9;
    },
    (value: Rows) => {
      value[0].qualification.referenceConflictPhases = 0;
    },
    (value: Rows) => {
      value[0].qualification.wholeCorpusQualified = true;
    },
    (value: Rows) => {
      value[0].qualification.issue6890Complete = true;
    },
    (value: Rows) => {
      value[0].sources.pop();
    },
    (value: Rows) => {
      value[0].witnesses[0].path = "marker-only.rs";
    },
    (value: Rows) => {
      value[0].inputs[0].sha256 = "0".repeat(64);
    },
    (value: Rows) => {
      value.push(structuredClone(value[0]));
    },
  ]) {
    const mutant = structuredClone(rows);
    mutate(mutant);
    assert.throws(() => newJsxChildApiCases(undefined, mutant, read));
  }
  const wider = structuredClone(rows) as (Rows[number] & { nativeHandled?: number })[];
  wider[0].nativeHandled = 1;
  assert.throws(() => newJsxChildApiCases(undefined, wider, read));
});
test("historical input registration grants no source privilege to a later producer", () => {
  const future: ReadFile = (side, path) =>
    side === "base" && paths.includes(path) ? read("head", path) : read(side, path);
  assert.deepEqual(newJsxChildApiCases(rows, rows, future), { ids: [], sources: new Set() });
  assert.throws(() => newJsxChildApiCases(undefined, rows, future));
});

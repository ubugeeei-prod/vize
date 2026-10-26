import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  LANGUAGE_PROJECT_CASE_ID,
  newLanguageProjectCases,
} from "../../tools/support/compat/github/language-project-fixtures.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const pairs: Array<{ source: string; before: string | null; head: string }> = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/tooling/language-project-source-pairs-10740.json"),
    "utf8",
  ),
);
const manifest = JSON.parse(
  fs.readFileSync(
    path.join(root, "tests/_fixtures/differential/typechecker/manifest.json"),
    "utf8",
  ),
);
const rows: unknown[] = manifest.languageProjectCases;
const read = (side: "base" | "head", file: string) => {
  const pair = pairs.find((item) => item.source === file);
  if (pair) {
    const text = side === "base" ? pair.before : pair.head;
    return text === null ? null : Buffer.from(text);
  }
  if (side === "base") return null;
  const source = path.join(root, file);
  return fs.existsSync(source) ? fs.readFileSync(source) : null;
};
const copy = () => structuredClone(manifest.languageProjectCases);

void test("the exact reviewed transition consumes a separate complete language-project protocol case", () => {
  const result = newLanguageProjectCases(undefined, rows, read);
  assert.deepEqual(result.ids, [LANGUAGE_PROJECT_CASE_ID]);
  assert.deepEqual(result.sources, new Set(pairs.map((pair) => pair.source)));
  assert(!manifest.cases.some((row: { id: string }) => row.id === LANGUAGE_PROJECT_CASE_ID));
  assert(
    !manifest.protocolCases.some((row: { id: string }) => row.id === LANGUAGE_PROJECT_CASE_ID),
  );
});

void test("missing or old protocol authority cannot be replaced by an ordinary batch case", () => {
  for (const current of [undefined, []])
    assert.throws(
      () => newLanguageProjectCases(undefined, current, read),
      /requires one new protocol case/,
    );
  assert.throws(() => newLanguageProjectCases(rows, rows, read), /requires one new protocol case/);
  const batch = copy();
  batch[0].targets = ["batch"];
  assert.throws(() => newLanguageProjectCases(undefined, batch, read), /unregistered targets/);
  assert.throws(() => newLanguageProjectCases(undefined, [...rows, ...rows], read), /duplicate ID/);
});

void test("every original input, before witness, producer and target remains checked", () => {
  const stale = copy();
  stale[0].inputs[1].sha256 = "0".repeat(64);
  assert.throws(() => newLanguageProjectCases(undefined, stale, read), /missing or stale input/);
  const producer = copy();
  producer[0].tests[0].sha256 = "0".repeat(64);
  assert.throws(() => newLanguageProjectCases(undefined, producer, read), /missing or stale input/);
  const incomplete = copy();
  incomplete[0].tests.pop();
  assert.throws(() => newLanguageProjectCases(undefined, incomplete, read), /all actual producers/);
  const before = copy();
  before[0].witness.run++;
  assert.throws(
    () => newLanguageProjectCases(undefined, before, read),
    /before provenance changed/,
  );
  const open = copy();
  open[0].nativeParity = true;
  assert.throws(() => newLanguageProjectCases(undefined, open, read), /closed schema/);
});

void test("different or future Canon edits retain the ordinary requirement and receive no source exemption", () => {
  const different = (side: "base" | "head", file: string) => {
    const bytes = read(side, file);
    return side === "head" && file === pairs[0]!.source
      ? Buffer.concat([bytes!, Buffer.from("\n")])
      : bytes;
  };
  assert.throws(
    () => newLanguageProjectCases(undefined, rows, different),
    /unregistered source transition/,
  );
  assert.deepEqual(newLanguageProjectCases(rows, rows, different), { ids: [], sources: new Set() });
  assert.deepEqual(newLanguageProjectCases(undefined, undefined, different), {
    ids: [],
    sources: new Set(),
  });
  const absent = () => null;
  assert.deepEqual(newLanguageProjectCases(undefined, undefined, absent), {
    ids: [],
    sources: new Set(),
  });
});

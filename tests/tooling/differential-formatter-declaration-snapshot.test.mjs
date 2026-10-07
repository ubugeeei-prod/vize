// Custody only: the named Rust snapshot executes on its normal Actions lane.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadFormatterApiManifest } from "../differential/formatter-api.mjs";
import {
  currentFormatterReference,
  formatterReferenceComparisons,
  validateFormatterReferencePass,
} from "../differential/formatter-current-reference.ts";
import { validateDeclarationSnapshotAuthority } from "../differential/formatter-declaration-reference.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = "tests/_fixtures/differential/formatter-history";
const packs = [
  "script",
  "prepared",
  "literal",
  "literal-extra",
  "vue-version",
  "capture",
  "capture-extra",
  "capture-final",
];
const cases = packs.flatMap(
  (name) =>
    loadFormatterApiManifest(path.join(root, directory, `${name}-manifest.json`), root).cases,
);
const qualified = cases.filter((row) => row.currentReference);

void test("nested declaration snapshot keeps the original whole witness and law", (t) => {
  validateDeclarationSnapshotAuthority(root);
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "css-declaration-snapshot-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const authorityPath = "tests/_fixtures/differential/formatter-history/current-snapshot-7866.json";
  const authority = JSON.parse(fs.readFileSync(path.join(root, authorityPath)));
  for (const relative of [
    authorityPath,
    authority.source.path,
    authority.historicalSnapshot.path,
    authority.currentSnapshot.path,
  ]) {
    const target = path.join(temporary, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, relative), target);
  }
  validateDeclarationSnapshotAuthority(temporary);
  const current = path.join(temporary, authority.currentSnapshot.path);
  const historical = path.join(temporary, authority.historicalSnapshot.path);
  fs.copyFileSync(historical, current);
  assert.throws(() => validateDeclarationSnapshotAuthority(temporary));
  fs.copyFileSync(path.join(root, authority.currentSnapshot.path), current);
  fs.appendFileSync(historical, "foreign bytes");
  assert.throws(() => validateDeclarationSnapshotAuthority(temporary));
});

void test("declaration refinement pins exact CSS ownership and preserves historical mismatches", (t) => {
  const fixture = qualified.find((row) => row.currentReference.issue === 7866);
  assert.equal(fixture.api, "format_style");
  assert.equal(fixture.kind, "CSS");
  assert.notDeepEqual(fixture.expected, fixture.currentExpected);
  for (const [row, input, historical] of [
    [{ ...fixture, api: "format_sfc" }, fixture.input, fixture.expected],
    [{ ...fixture, profile: "skip_script_stabilization" }, fixture.input, fixture.expected],
    [{ ...fixture, outcome: "error" }, fixture.input, fixture.expected],
    [{ ...fixture, kind: "Vue" }, fixture.input, fixture.expected],
    [
      { ...fixture, options: { ...fixture.options, userOverrides: { useTabs: true } } },
      fixture.input,
      fixture.expected,
    ],
    [
      { ...fixture, witness: { ...fixture.witness, function: "foreign_owner" } },
      fixture.input,
      fixture.expected,
    ],
    [
      { ...fixture, witness: { ...fixture.witness, sourceSha256: "0".repeat(64) } },
      fixture.input,
      fixture.expected,
    ],
    [fixture, Buffer.from(".replacement{}"), fixture.expected],
    [fixture, fixture.input, fixture.currentExpected],
  ])
    assert.throws(() => currentFormatterReference(root, row, input, historical));
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-declaration-reference-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const authority = fixture.currentReference.authority.path;
  fs.mkdirSync(path.dirname(path.join(temporary, authority)), { recursive: true });
  fs.writeFileSync(path.join(temporary, authority), "{}");
  assert.throws(() =>
    currentFormatterReference(temporary, fixture, fixture.input, fixture.expected),
  );
  const row = {
    currentReference: fixture.currentReference,
    legacy: { state: "matched-reference" },
  };
  const current = formatterReferenceComparisons(fixture, fixture.currentExpected);
  assert.equal(current.referenceComparison.state, "different");
  assert.equal(current.currentReferenceComparison.state, "equal");
  validateFormatterReferencePass(fixture, row, current, fixture.currentExpected);
  for (const output of [
    fixture.expected,
    Buffer.from(fixture.currentExpected.toString().replace("thin 0 0 0", "thin 0")),
  ]) {
    assert.throws(() =>
      validateFormatterReferencePass(
        fixture,
        row,
        formatterReferenceComparisons(fixture, output),
        output,
      ),
    );
  }
  assert.throws(() =>
    validateFormatterReferencePass(
      fixture,
      { ...row, currentReference: undefined },
      current,
      fixture.currentExpected,
    ),
  );
  assert.throws(() =>
    validateFormatterReferencePass(
      fixture,
      row,
      { ...current, referenceComparison: { state: "equal" } },
      fixture.currentExpected,
    ),
  );
});

// Pure controls qualify source custody only, never actual formatter execution.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadFormatterApiManifest } from "../differential/formatter-api.mjs";
import {
  currentFormatterReference,
  currentFormatterReferenceSummary,
  formatterReferenceComparisons,
  validateFormatterReferencePass,
} from "../differential/formatter-current-reference.ts";
import { preservedHuggedInterpolationSnapshot } from "../differential/formatter-hugged-interpolation-reference.ts";
import { validateRootCommentSnapshotTransition } from "../differential/formatter-root-comment-reference.ts";

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

void test("six reviewed current references keep all original300 source and captured outputs", () => {
  assert.equal(cases.length, 300);
  assert.equal(qualified.length, 6);
  const fixture = qualified.find((row) => row.currentReference.issue === 7826);
  assert.equal(fixture.api, "format_sfc");
  assert.equal(fixture.currentReference.issue, 7826);
  assert.notDeepEqual(fixture.expected, fixture.currentExpected);
  assert.deepEqual(fixture.input, fixture.currentExpected);
  assert.equal(fixture.expected.length, 184);
  assert.equal(fixture.currentExpected.length, 185);
  const whole = cases.find((row) => row.id === fixture.id.replace("sfc/0", "style/1"));
  assert(whole && whole.currentReference.issue === 7866);
  const url = cases.find((row) => row.id === "prepared/style-comment-in-url");
  assert(url && !url.currentReference);
  const roots = qualified.filter((row) => row.currentReference.issue === 7877);
  assert.equal(roots.length, 2);
  for (const row of roots) {
    assert.equal(row.api, "format_sfc");
    assert.notDeepEqual(row.expected, row.currentExpected);
    const removedSeparators = row.id.includes("comments-follow-their-block") ? 2 : 1;
    assert.equal(row.expected.length, row.currentExpected.length + removedSeparators);
  }
});

void test("current refinement rejects source, API, history and qualification forgery", (t) => {
  const fixture = qualified.find((row) => row.currentReference.issue === 7826);
  for (const [row, input, historical] of [
    [{ ...fixture, api: "format_style" }, fixture.input, fixture.expected],
    [{ ...fixture, profile: "skip_script_stabilization" }, fixture.input, fixture.expected],
    [{ ...fixture, outcome: "error" }, fixture.input, fixture.expected],
    [{ ...fixture, kind: "TypeScript" }, fixture.input, fixture.expected],
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
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-current-reference-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const authority = fixture.currentReference.authority.path;
  fs.mkdirSync(path.dirname(path.join(temporary, authority)), { recursive: true });
  fs.writeFileSync(path.join(temporary, authority), "{}");
  assert.throws(() =>
    currentFormatterReference(temporary, fixture, fixture.input, fixture.expected),
  );
});

void test("historical mismatch stays a failure while current admission requires exact complete bytes", () => {
  const fixture = qualified.find((row) => row.currentReference.issue === 7826);
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
    Buffer.from(fixture.currentExpected.toString().replace("red;\n\n", "red;\n")),
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
  assert.deepEqual(currentFormatterReferenceSummary([row]), { currentReferenceMatches: 1 });
  assert.deepEqual(currentFormatterReferenceSummary([{ ...row, legacy: { state: "failed" } }]), {
    currentReferenceMatches: 0,
  });
});

void test("both exact root-comment refinements reject forged ownership and old wrong blank lines", () => {
  for (const fixture of qualified.filter((row) => row.currentReference.issue === 7877)) {
    for (const [row, input, historical] of [
      [{ ...fixture, api: "format_style" }, fixture.input, fixture.expected],
      [{ ...fixture, kind: "VueTemplate" }, fixture.input, fixture.expected],
      [{ ...fixture, profile: "skip_script_stabilization" }, fixture.input, fixture.expected],
      [{ ...fixture, outcome: "error" }, fixture.input, fixture.expected],
      [
        { ...fixture, options: { ...fixture.options, userOverrides: { sortBlocks: false } } },
        fixture.input,
        fixture.expected,
      ],
      [
        { ...fixture, witness: { ...fixture.witness, path: "foreign.rs" } },
        fixture.input,
        fixture.expected,
      ],
      [
        { ...fixture, witness: { ...fixture.witness, sourceSha256: "0".repeat(64) } },
        fixture.input,
        fixture.expected,
      ],
      [
        { ...fixture, witness: { ...fixture.witness, inputExpressionRangeBytes: [0, 1] } },
        fixture.input,
        fixture.expected,
      ],
      [fixture, Buffer.from("<template />"), fixture.expected],
      [fixture, fixture.input, fixture.currentExpected],
    ])
      assert.throws(() => currentFormatterReference(root, row, input, historical));
    const row = {
      currentReference: fixture.currentReference,
      legacy: { state: "matched-reference" },
    };
    const comparison = formatterReferenceComparisons(fixture, fixture.currentExpected);
    assert.equal(comparison.referenceComparison.state, "different");
    assert.equal(comparison.currentReferenceComparison.state, "equal");
    validateFormatterReferencePass(fixture, row, comparison, fixture.currentExpected);
    for (const output of [
      fixture.expected,
      Buffer.concat([fixture.currentExpected, Buffer.from("\n")]),
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
    if (fixture.id.includes("comments-follow-their-block")) {
      const superseded = Buffer.from(
        "<!-- template note -->\n\n<!-- script note -->\n<script setup>\n" +
          "const ready = true;\n</script>\n\n<template>\n  <div>ok</div>\n</template>\n",
      );
      assert.throws(() =>
        validateFormatterReferencePass(
          fixture,
          row,
          formatterReferenceComparisons(fixture, superseded),
          superseded,
        ),
      );
    }
    assert.throws(() =>
      validateFormatterReferencePass(
        fixture,
        { ...row, currentReference: undefined },
        comparison,
        fixture.currentExpected,
      ),
    );
    assert.throws(() =>
      validateFormatterReferencePass(
        fixture,
        row,
        { ...comparison, referenceComparison: { state: "equal" } },
        fixture.currentExpected,
      ),
    );
  }
});

void test("the two snapshot transitions retain original bytes and reject restored wrong current output", () => {
  const authority = JSON.parse(
    fs.readFileSync(
      path.join(
        root,
        "tests/_fixtures/differential/formatter-history/current-references-7877.json",
      ),
    ),
  );
  for (const row of authority.cases) {
    const current = fs.readFileSync(path.join(root, row.currentSnapshot.path));
    const historical = fs.readFileSync(path.join(root, row.historicalSnapshot.path));
    const name = row.witness.function;
    const hash = row.historicalSnapshot.sha256;
    assert.equal(validateRootCommentSnapshotTransition(root, name, current, hash), true);
    assert.throws(() => validateRootCommentSnapshotTransition(root, name, historical, hash));
    assert.throws(() => validateRootCommentSnapshotTransition(root, name, current, "0".repeat(64)));
    assert.throws(() =>
      validateRootCommentSnapshotTransition(
        root,
        name,
        Buffer.concat([current, Buffer.from("\n")]),
        hash,
      ),
    );
  }
  assert.equal(
    validateRootCommentSnapshotTransition(root, "foreign_owner", Buffer.from(""), ""),
    false,
  );
});

void test("two exact hugged references retain full historical mismatches and reject foreign scope", () => {
  const rows = qualified.filter((row) => row.currentReference.issue === 7969);
  assert.equal(rows.length, 2);
  for (const fixture of rows) {
    for (const [row, input, historical] of [
      [{ ...fixture, api: "format_style" }, fixture.input, fixture.expected],
      [{ ...fixture, kind: "TypeScript" }, fixture.input, fixture.expected],
      [{ ...fixture, profile: "skip_script_stabilization" }, fixture.input, fixture.expected],
      [{ ...fixture, outcome: "error" }, fixture.input, fixture.expected],
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
      [fixture, Buffer.concat([fixture.input, Buffer.from("\n")]), fixture.expected],
      [fixture, fixture.input, fixture.currentExpected],
    ])
      assert.throws(() => currentFormatterReference(root, row, input, historical));
    const row = {
      currentReference: fixture.currentReference,
      legacy: { state: "matched-reference" },
    };
    const compare = formatterReferenceComparisons(fixture, fixture.currentExpected);
    assert.equal(compare.referenceComparison.state, "different");
    assert.equal(compare.currentReferenceComparison.state, "equal");
    validateFormatterReferencePass(fixture, row, compare, fixture.currentExpected);
    for (const output of [
      fixture.expected,
      Buffer.concat([fixture.currentExpected, Buffer.from("\n")]),
    ])
      assert.throws(() =>
        validateFormatterReferencePass(
          fixture,
          row,
          formatterReferenceComparisons(fixture, output),
          output,
        ),
      );
  }
});

void test("hugged snapshot custody admits only complete corrected current files", (t) => {
  const authorityPath =
    "tests/_fixtures/differential/formatter-history/current-references-7969.json";
  const raw = fs.readFileSync(path.join(root, authorityPath));
  const authority = JSON.parse(raw);
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "hugged-snapshot-custody-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  for (const row of authority.cases) {
    for (const relative of [authorityPath, row.currentSnapshot.path, row.historicalSnapshot.path]) {
      const target = path.join(temporary, relative);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(root, relative), target);
    }
    const artifact = { path: row.currentSnapshot.path, sha256: row.historicalExpectedSha256 };
    const historical = fs.readFileSync(path.join(root, row.historicalSnapshot.path));
    assert.deepEqual(preservedHuggedInterpolationSnapshot(temporary, artifact), historical);
    assert.throws(() =>
      preservedHuggedInterpolationSnapshot(temporary, { ...artifact, sha256: "0".repeat(64) }),
    );
    fs.writeFileSync(path.join(temporary, artifact.path), historical);
    assert.throws(() => preservedHuggedInterpolationSnapshot(temporary, artifact));
  }
  assert.equal(
    preservedHuggedInterpolationSnapshot(root, { path: "foreign.snap.txt", sha256: "" }),
    null,
  );
});

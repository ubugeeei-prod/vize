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

void test("one reviewed current reference keeps all original300 source and captured outputs", () => {
  assert.equal(cases.length, 300);
  assert.equal(qualified.length, 1);
  const fixture = qualified[0];
  assert.equal(fixture.api, "format_sfc");
  assert.equal(fixture.currentReference.issue, 7826);
  assert.notDeepEqual(fixture.expected, fixture.currentExpected);
  assert.deepEqual(fixture.input, fixture.currentExpected);
  assert.equal(fixture.expected.length, 184);
  assert.equal(fixture.currentExpected.length, 185);
  const whole = cases.find((row) => row.id === fixture.id.replace("sfc/0", "style/1"));
  assert(whole && !whole.currentReference);
  const url = cases.find((row) => row.id === "prepared/style-comment-in-url");
  assert(url && !url.currentReference);
});

void test("current refinement rejects source, API, history and qualification forgery", (t) => {
  const fixture = qualified[0];
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
  const fixture = qualified[0];
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

// Pure custody controls; actual formatter executions remain in hosted Actions.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadFormatterApiManifest } from "../differential/formatter-api.mjs";
import { currentFormatterReference } from "../differential/formatter-current-reference.ts";
import {
  JSON_LAYOUT_SOURCE,
  validatePreservedJsonLayoutWitness,
} from "../differential/formatter-json-layout-source.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifest = "tests/_fixtures/differential/formatter-history/literal-manifest.json";
const cases = loadFormatterApiManifest(path.join(root, manifest), root).cases;
const qualified = cases.filter((row) => row.currentReference?.issue === 7928);

void test("six JSON refinements retain original carriers, scalar tokens and historical mismatches", () => {
  assert.equal(qualified.length, 6);
  for (const row of qualified) {
    assert.notDeepEqual(row.expected, row.currentExpected);
    if (row.api === "format_json") {
      assert.deepEqual(
        JSON.parse(row.currentExpected.toString()),
        JSON.parse(row.input.toString()),
      );
    }
    for (const [fixture, input, historical] of [
      [{ ...row, api: "format_sfc" }, row.input, row.expected],
      [{ ...row, kind: "Vue" }, row.input, row.expected],
      [{ ...row, profile: "skip_script_stabilization" }, row.input, row.expected],
      [{ ...row, outcome: "error" }, row.input, row.expected],
      [
        { ...row, options: { ...row.options, userOverrides: { useTabs: true } } },
        row.input,
        row.expected,
      ],
      [{ ...row, witness: { ...row.witness, function: "foreign_owner" } }, row.input, row.expected],
      [row, Buffer.from('{"changed":true}'), row.expected],
      [row, row.input, row.currentExpected],
    ]) {
      assert.throws(() => currentFormatterReference(root, fixture, input, historical));
    }
  }
  const numbers = qualified.find((row) => row.id.includes("preserves-valid-number-tokens"));
  assert.match(numbers.currentExpected.toString(), /6\.02e\+23/);
  assert.match(numbers.currentExpected.toString(), /1E-9/);
});

void test("JSON witness transition rejects altered source, original asset and ownership", (t) => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-json-law-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  for (const relative of [JSON_LAYOUT_SOURCE.owner, JSON_LAYOUT_SOURCE.originalAsset]) {
    const target = path.join(scratch, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, relative), target);
  }
  const entry = {
    path: JSON_LAYOUT_SOURCE.owner,
    sha256: JSON_LAYOUT_SOURCE.originalSha256,
    revisions: [...JSON_LAYOUT_SOURCE.revisions],
  };
  const check = (owner = entry, name = "pretty_prints_minified_object") =>
    validatePreservedJsonLayoutWitness(scratch, owner, name);
  assert.equal(check(), true);
  assert.throws(() => check({ ...entry, sha256: "0".repeat(64) }));
  assert.throws(() => check({ ...entry, revisions: [] }));
  assert.throws(() => check(entry, "unregistered_json_law"));
  const current = path.join(scratch, JSON_LAYOUT_SOURCE.owner);
  fs.appendFileSync(current, "\n// source mutation\n");
  assert.throws(() => check());
  fs.copyFileSync(path.join(root, JSON_LAYOUT_SOURCE.owner), current);
  fs.appendFileSync(path.join(scratch, JSON_LAYOUT_SOURCE.originalAsset), "\n// asset mutation\n");
  assert.throws(() => check());
});

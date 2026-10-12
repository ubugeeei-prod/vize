import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "./lint-default-correctness-reference.mjs";
import {
  authenticateDeepPropsImportCorrection,
  authenticateDeepPropsSourceFamily,
} from "./support/component-value-retained-guards.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = "tests/_fixtures/differential/lint/props-deep-destructure-8090/source-custody";
const read = (name) => fs.readFileSync(path.join(root, name));

await test("the failed import authority stays literal and the whole current family is explicit", () => {
  const original = read(`${directory}/successor.json`);
  assert.equal(
    sha256(original),
    "de01ccf20b54abc69935b2ffa844705b1704bac8f83c07ee832c339ff9aaa161",
  );
  assert.equal(
    sha256(read(`${directory}/import-correction/original-deep-source-custody.mjs.fixture`)),
    "c5466b1a87cd56dced48a802cdc26425ddce20effcb1d6e8933186a054d65463",
  );
  const family = authenticateDeepPropsSourceFamily(root, read);
  const { currentSuccessor, ...historical } = family;
  assert.deepEqual(historical, JSON.parse(original));
  assert.equal(currentSuccessor.sources.length, 12);
  assert.equal(currentSuccessor.sources.filter(({ changed }) => changed).length, 1);
  assert.deepEqual(currentSuccessor.changedPaths, [
    "crates/vize_patina/src/linter/script_rules.rs",
  ]);
  assert.deepEqual(authenticateDeepPropsImportCorrection(root, historical, read), currentSuccessor);
});

await test("the import successor refuses all unknown bodies, archives and its failed parent", () => {
  const incoming = JSON.parse(read(`${directory}/successor.json`));
  const successor = authenticateDeepPropsImportCorrection(root, incoming, read);
  const names = [
    ...new Set([
      `${directory}/import-correction/successor.json`,
      `${directory}/successor.json`,
      ...successor.sources.flatMap((source) => [
        source.path,
        source.before.archive,
        source.after.archive,
      ]),
    ]),
  ];
  for (const name of names) {
    const unknown = (requested) =>
      requested === name ? Buffer.concat([read(requested), Buffer.from(" ")]) : read(requested);
    assert.throws(() => authenticateDeepPropsImportCorrection(root, incoming, unknown), name);
  }
  const changed = successor.sources.find(({ changed }) => changed);
  const failedParent = (name) =>
    name === changed.path ? read(changed.before.archive) : read(name);
  assert.throws(() => authenticateDeepPropsImportCorrection(root, incoming, failedParent));
  // A whole historical family is explicitly historical source custody, never this repair.
  const historical = authenticateDeepPropsSourceFamily(root, failedParent);
  assert.deepEqual(historical, incoming);
  assert.equal(historical.currentSuccessor, undefined);
});

import assert from "node:assert/strict";
import { retainedComponentCallerFs as fs } from "./support/component-value-retained-guards.ts";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "./lint-default-correctness-reference.mjs";
import {
  authenticateDeepPropsSourceFamily,
  authenticateDeepPropsTraitScopeCorrection,
} from "./support/component-value-retained-guards.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = "tests/_fixtures/differential/lint/props-deep-destructure-8090/source-custody";
const read = (name) => fs.readFileSync(path.join(root, name));
const incoming = JSON.parse(read(`${directory}/import-correction/successor.json`));

await test("the whole trait-scope successor preserves both failed families and the old laws", () => {
  const family = authenticateDeepPropsSourceFamily(root, read);
  const { currentSuccessor, ...original } = family;
  const { currentSuccessor: current, ...previous } = currentSuccessor;
  assert.deepEqual(original, JSON.parse(read(`${directory}/successor.json`)));
  assert.deepEqual(previous, incoming);
  assert.deepEqual(authenticateDeepPropsTraitScopeCorrection(root, previous, read), current);
  assert.equal(current.candidate, "ecb8c9b25ee083725858229a2894052bb6de1fbf");
  assert.equal(current.sources.length, 12);
  assert.equal(current.sources.filter(({ changed }) => changed).length, 1);
  assert.deepEqual(current.changedPaths, ["crates/vize_patina/src/linter/script_rules.rs"]);
  const fixture = `${directory}/trait-scope-correction`;
  assert.equal(
    sha256(read(`${fixture}/original-deep-source-custody.mjs.fixture`)),
    "741ed81d1854d9e5a41c95b16f703617e797dd36a03a781f525291ed78703216",
  );
  assert.deepEqual(
    read(`${fixture}/original-import-source-custody.test.mjs.fixture`),
    read("tests/tooling/props-deep-import-source-custody.test.mjs"),
  );
});

await test("the current trait family refuses unknown bodies and both failed source parents", () => {
  const current = authenticateDeepPropsTraitScopeCorrection(root, incoming, read);
  const names = [
    ...new Set([
      `${directory}/trait-scope-correction/successor.json`,
      `${directory}/import-correction/successor.json`,
      `${directory}/successor.json`,
      ...current.sources.flatMap((source) => [
        source.path,
        source.before.archive,
        source.after.archive,
      ]),
    ]),
  ];
  for (const name of names) {
    const unknown = (requested) =>
      requested === name ? Buffer.concat([read(requested), Buffer.from(" ")]) : read(requested);
    assert.throws(() => authenticateDeepPropsTraitScopeCorrection(root, incoming, unknown), name);
    assert.throws(() => authenticateDeepPropsSourceFamily(root, unknown), name);
  }
  const changed = current.sources.find(({ changed }) => changed);
  const original = JSON.parse(read(`${directory}/successor.json`));
  const first = original.sources.find((source) => source.path === changed.path);
  for (const archive of [changed.before.archive, first.after.archive]) {
    const historicalRead = (name) => (name === changed.path ? read(archive) : read(name));
    assert.throws(() => authenticateDeepPropsTraitScopeCorrection(root, incoming, historicalRead));
    const historical = authenticateDeepPropsSourceFamily(root, historicalRead);
    assert.equal(historical.currentSuccessor?.currentSuccessor, undefined);
    assert.deepEqual(
      historical,
      archive === changed.before.archive ? { ...original, currentSuccessor: incoming } : original,
    );
  }
});

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  componentValueRetainedGuardSource,
  retainedComponentGuardFs,
  retainedComponentMigrationFs,
} from "./support/component-value-retained-guards.ts";

const root = path.resolve(import.meta.dirname, "../..");
const read = (file: string) => fs.readFileSync(path.join(root, file));
const capsuleFile =
  "tests/_fixtures/differential/linter/component-import-value-host-before/retained-guard-callers.json";
const sfc = "crates/vize_patina/src/linter/engine/sfc.rs";

void test("complete current sources qualify before two retained production bodies and four complete caller inverses", () => {
  const result = componentValueRetainedGuardSource(root);
  assert.equal(result.receipt.runtimeCredit, false);
  assert.equal(result.receipt.performanceCredit, false);
  assert.deepEqual(result.receipt.projectedProductionBodies, [
    sfc,
    "crates/vize_patina/src/linter/engine/rule_sets.rs",
  ]);
  assert.equal(result.receipt.projectedCallerBodies.length, 4);
  assert.equal(result.receipt.sourceMetadataBodyPairs, 1);
  assert.notDeepEqual(result.read(sfc), read(sfc));
  const capsule = JSON.parse(read(capsuleFile).toString("utf8"));
  for (const entry of capsule.entries) {
    assert.deepEqual(result.read(entry.path), Buffer.from(entry.rawBase64, "base64"));
    assert.notDeepEqual(result.read(entry.path), read(entry.path));
  }
});

void test("every actual selected source refuses foreign or missing input before any projection", () => {
  const result = componentValueRetainedGuardSource(root);
  const before = JSON.parse(
    read(
      "tests/_fixtures/differential/linter/component-import-value-host-before/entries.json",
    ).toString("utf8"),
  );
  const absent = JSON.parse(
    read(
      "tests/_fixtures/differential/linter/component-import-value-host-before/host-supplement-before.json",
    ).toString("utf8"),
  );
  const sourcePaths = new Set<string>(
    [...before.entries, ...absent.entries].map((entry: { path: string }) => entry.path),
  );
  assert.equal(sourcePaths.size, 81);
  const selected = result.receipt.bodies.filter((identity) => sourcePaths.has(identity.path));
  assert.equal(selected.length, 81);
  for (const identity of selected) {
    for (const body of [Buffer.alloc(0), Buffer.concat([read(identity.path), Buffer.from(" ")])]) {
      assert.throws(() =>
        componentValueRetainedGuardSource(root, (file) =>
          file === identity.path ? body : read(file),
        ),
      );
    }
  }
});

void test("caller and capsule changes cannot borrow an authenticated complete parent", () => {
  const result = componentValueRetainedGuardSource(root);
  const capsule = JSON.parse(read(capsuleFile).toString("utf8"));
  const allCallers = [...capsule.entries, ...capsule.originalCallers];
  assert.equal(allCallers.length, 10);
  assert.equal(result.receipt.actualCallerBodies, 10);
  assert.equal(result.receipt.bodies.length, 94);
  for (const entry of allCallers)
    assert.throws(
      () =>
        componentValueRetainedGuardSource(root, (file) =>
          file === entry.path ? Buffer.from("foreign caller") : read(file),
        ),
      /unknown supplied retained caller/u,
    );
  for (const entry of capsule.entries) {
    const before = Buffer.from(entry.rawBase64, "base64");
    const retained = componentValueRetainedGuardSource(root, (file) =>
      file === entry.path ? before : read(file),
    );
    assert.deepEqual(retained.read(entry.path), before);
  }
  for (const body of [Buffer.alloc(0), Buffer.concat([read(capsuleFile), Buffer.from(" ")])]) {
    assert.throws(() =>
      componentValueRetainedGuardSource(root, (file) => (file === capsuleFile ? body : read(file))),
    );
  }
});

void test("retained comparison cannot grant a different build or repository current-source custody", () => {
  const result = componentValueRetainedGuardSource(root);
  assert.throws(() => result.assertBuildSource("0".repeat(40)));
  assert.throws(() => componentValueRetainedGuardSource(path.dirname(root)));
});

void test("the complete bare caller refuses either one-import hybrid before its retained comparison", () => {
  const capsule = JSON.parse(read(capsuleFile).toString("utf8"));
  const caller = "tests/tooling/bare-reserved-default-route-reference.test.mjs";
  const entry = capsule.entries.find((candidate: { path: string }) => candidate.path === caller);
  assert.ok(entry);
  assert.equal(entry.transforms.length, 2);
  const before = Buffer.from(entry.rawBase64, "base64");
  for (const transform of entry.transforms) {
    const hybrid = Buffer.from(before.toString("utf8").replace(transform.before, transform.after));
    assert.notDeepEqual(hybrid, before);
    assert.notDeepEqual(hybrid, read(caller));
    assert.throws(
      () =>
        componentValueRetainedGuardSource(root, (file) => (file === caller ? hybrid : read(file))),
      /unknown supplied retained caller/u,
    );
  }
});

void test("only the historical migration facade exposes the complete original rule set after current-source qualification", () => {
  const file = "crates/vize_patina/src/linter/engine/rule_sets.rs";
  const target = path.join(root, file);
  const retained = componentValueRetainedGuardSource(root);
  const before = retained.read(file);
  assert.notDeepEqual(before, read(file));
  assert.deepEqual(retainedComponentGuardFs.readFileSync(target), read(file));
  assert.deepEqual(retainedComponentMigrationFs.readFileSync(target), before);
  assert.throws(() =>
    componentValueRetainedGuardSource(root, (requested) =>
      requested === file ? before : read(requested),
    ),
  );
  const incomplete = JSON.parse(read(capsuleFile).toString("utf8"));
  incomplete.entries.pop();
  const changed = Buffer.from(JSON.stringify(incomplete, null, 2) + "\n");
  assert.throws(() =>
    componentValueRetainedGuardSource(root, (requested) =>
      requested === capsuleFile ? changed : read(requested),
    ),
  );
});

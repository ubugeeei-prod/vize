import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  beforeScriptConsumedSlots,
  beforeTypedUseSlots,
} from "../../tools/support/compat/content-mapper/authored-component-infer-transition.ts";
import { reviewedSource } from "../../tools/support/compat/content-mapper/reviewed-source.ts";

const root = new URL("../../", import.meta.url);
const identity = (text: string) => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});
const raw = readFileSync(new URL("tests/_fixtures/tooling/script-consumed-slots-13578.json", root));
assert.equal(
  createHash("sha256").update(raw).digest("hex"),
  "87473c12da88cdefdadb4d3fac8e2ac97c61495e45896502783c2298e7a15314",
);
const record = JSON.parse(raw.toString("utf8"));

void test("script-consumed slot producers restore exact predecessors without historical credit", () => {
  assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });
  assert.deepEqual(
    record.files.map((file: { path: string }) => file.path),
    [
      "crates/vize_canon/src/virtual_ts/generator.rs",
      "crates/vize_canon/src/virtual_ts/generator/setup_imports.rs",
      "crates/vize_canon/src/virtual_ts/generator/component_public_types.rs",
    ],
  );
  for (const file of record.files) {
    const current = beforeTypedUseSlots(file.path, readFileSync(reviewedSource(file.path), "utf8"));
    assert.deepEqual(identity(current), file.after);
    const previous = beforeScriptConsumedSlots(file.path, current);
    assert.deepEqual(identity(previous), file.before);
    assert.equal(beforeScriptConsumedSlots(file.path, previous), previous);
    assert.equal(beforeScriptConsumedSlots("other.rs", current), current);
    for (const mutated of [
      current + current,
      current + "\n",
      ...file.hunks.map((hunk: { after: string }) => current.replace(hunk.after, "")),
    ]) {
      assert.equal(beforeScriptConsumedSlots(file.path, mutated), mutated);
      assert.notDeepEqual(identity(mutated), file.before);
    }
  }
});

const typedRaw = readFileSync(new URL("tests/_fixtures/tooling/typed-use-slots-13578.json", root));
assert.equal(
  createHash("sha256").update(typedRaw).digest("hex"),
  "6454233b984533bce96260ee97fd94dd989449749f4fb1b484a10582becd9978",
);
const typedRecord = JSON.parse(typedRaw.toString("utf8"));

void test("the typed useSlots follow-up restores its exact script-consumed predecessor", () => {
  assert.deepEqual(typedRecord.credit, { compiled: false, native: false, performance: false });
  assert.deepEqual(
    typedRecord.files.map((file: { path: string }) => file.path),
    ["crates/vize_canon/src/virtual_ts/generator/setup_imports.rs"],
  );
  for (const file of typedRecord.files) {
    const current = readFileSync(reviewedSource(file.path), "utf8");
    assert.deepEqual(identity(current), file.after);
    const previous = beforeTypedUseSlots(file.path, current);
    assert.deepEqual(identity(previous), file.before);
    const predecessor = record.files.find((other: { path: string }) => other.path === file.path);
    assert.deepEqual(file.before, predecessor.after);
    assert.equal(beforeTypedUseSlots(file.path, previous), previous);
    assert.equal(beforeTypedUseSlots("other.rs", current), current);
    for (const mutated of [
      current + current,
      current + "\n",
      ...file.hunks.map((hunk: { after: string }) => current.replace(hunk.after, "")),
    ]) {
      assert.equal(beforeTypedUseSlots(file.path, mutated), mutated);
      assert.notDeepEqual(identity(mutated), file.before);
    }
  }
});

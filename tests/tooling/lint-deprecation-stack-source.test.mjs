import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import { loadCurrentCorpus } from "./support/lint-current-correctness-corpus.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const sha = (body) => createHash("sha256").update(body).digest("hex");
const bytes = fs.readFileSync(
  path.join(root, "tests/_fixtures/lint-deprecation-stack-source-composition.json.gz"),
);
assert.equal(sha(bytes), "6cc05cd6a209110a6f25aa8955faeb4952a78f6d8e56553d0ab12ddeb1177079");
const packet = JSON.parse(gunzipSync(bytes));
function decode(row) {
  const body = Buffer.from(row.base64, "base64");
  assert.equal(body.toString("base64"), row.base64);
  assert.equal(body.length, row.bytes);
  assert.equal(sha(body), row.sha256);
  assert.equal(
    createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
    row.gitBlob,
  );
  return body;
}
function validate(value) {
  assert.equal(value.schema, "vize.patina.deprecation-stack-source-composition");
  assert.equal(value.version, 1);
  assert.equal(value.parent, "98e9d7188efb1561a34c5ede5100d94073b03fd9");
  assert.equal(value.own, "a44e73dc536098746bdf76ca54fd6c081dde9b0e");
  assert.deepEqual(value.credit, { sourceOnly: true, native: false, performance: false });
  const { current: before, ...parentRest } = value.parentManifest;
  const { current: after, ...afterRest } = value.afterManifest;
  assert.deepEqual(parentRest, afterRest);
  assert.deepEqual(
    afterRest,
    Object.fromEntries(Object.entries(value.ownManifest).filter(([key]) => key !== "current")),
  );
  assert.equal(before.length, 323);
  assert.equal(after.length, 323);
  assert.deepEqual(
    value.changes.map((row) => row.path),
    [
      "crates/vize_maestro/src/ide/diagnostics/default_migration_tests.rs",
      "crates/vize_patina/src/preset/default_migration_tests.rs",
      "crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs",
    ],
  );
  for (let index = 0; index < before.length; index++) {
    const original = before[index];
    const current = after[index];
    assert.equal(current.path, original.path);
    const change = value.changes.find((row) => row.path === original.path);
    if (!change) assert.deepEqual(current, original);
    else {
      for (const [side, identity] of [
        ["before", original],
        ["after", current],
      ]) {
        decode(change[side]);
        const { base64: _base64, ...sealed } = change[side];
        assert.deepEqual(identity, { ...sealed, mode: "100644" });
      }
      if (current.path.startsWith("crates/vize_maestro/")) {
        const beforeBody = decode(change.before).toString();
        const marker = "lint-default-migrations/cases.json";
        assert.equal(beforeBody.split(marker).length, 2);
        assert.equal(
          decode(change.after).toString(),
          beforeBody.replace(marker, "lint-default-migrations/current-range-cases.json"),
        );
      } else
        assert.deepEqual(
          value.ownManifest.current.find((row) => row.path === current.path),
          current,
        );
    }
  }
  assert.deepEqual(
    value.pins.map((row) => row.path),
    [
      "tests/tooling/support/no-mutating-component-join.mjs",
      "tests/tooling/no-mutating-component-join-source.test.mjs",
    ],
  );
  for (const [index, row] of value.pins.entries()) {
    const beforeBody = decode(row.before).toString();
    const afterBody = decode(row.after).toString();
    const oldHash =
      index === 0
        ? sha(JSON.stringify(value.parentManifest, null, 2) + "\n")
        : value.pins[0].before.sha256;
    const newHash =
      index === 0
        ? sha(JSON.stringify(value.afterManifest, null, 2) + "\n")
        : value.pins[0].after.sha256;
    assert.equal(beforeBody.split(oldHash).length, 2);
    assert.equal(afterBody, beforeBody.replace(oldHash, newHash));
    assert.equal(afterBody.replace(newHash, oldHash), beforeBody);
  }
}
void test("the historical whole Stack inverse preserves every inherited source and both original frames", () => {
  validate(packet);
  // Only the existing physical source family qualifies later live descendants.
  for (const mutate of [
    (value) => value.afterManifest.current.pop(),
    (value) => value.afterManifest.current.reverse(),
    (value) => {
      value.afterManifest.current[0].bytes++;
    },
    (value) => value.afterManifest.literalFrames.reverse(),
    (value) => value.changes.pop(),
    (value) => {
      value.pins[0].after.base64 += "foreign";
    },
    (value) => {
      value.credit.native = true;
    },
  ]) {
    const changed = structuredClone(packet);
    mutate(changed);
    assert.throws(() => validate(changed));
  }
});
void test("the current CLI range reader retains the exact historical reader and complete packet inverse", () => {
  assert.equal(
    sha(fs.readFileSync(path.join(root, "tests/tooling/lint-default-correctness-reference.mjs"))),
    "d0fe46840f15d3aee0cad8b0394b4c67dd1f738a9ddaba39f55073fefd9df6b7",
  );
  assert.equal(loadCurrentCorpus(root).cases.length, 15);
});

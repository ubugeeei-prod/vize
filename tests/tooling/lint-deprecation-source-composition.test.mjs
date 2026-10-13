import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";

const root = path.resolve(import.meta.dirname, "../..");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const bytes = fs.readFileSync(
  path.join(root, "tests/_fixtures/lint-deprecation-source-composition.json.gz"),
);
assert.equal(digest(bytes), "65b7af986b182c73aab4a109788500aad2c4fce52168cebc47a3560748389b30");
const receipt = JSON.parse(gunzipSync(bytes));
const names = [
  "crates/vize_patina/src/preset/default_migration_tests.rs",
  "crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs",
  "tests/tooling/lint-default-correctness-reference.mjs",
];
function validate(packet) {
  assert.equal(packet.schema, "vize.patina.deprecation-source-composition");
  assert.equal(packet.version, 1);
  assert.equal(packet.main, "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38");
  assert.equal(packet.prepared, "f652e03ba21cd3f1fdd3438c68b3cd366304a2e0");
  assert.equal(packet.sourceOnly, true);
  assert.equal(packet.native, false);
  assert.equal(packet.performance, false);
  assert.deepEqual(
    packet.changes.map((row) => row.path),
    names,
  );
  const { current: before, ...oldRest } = packet.mainManifest;
  const { current: after, ...newRest } = packet.afterManifest;
  assert.equal(before.length, 323);
  assert.equal(after.length, 323);
  assert.deepEqual(newRest, oldRest);
  const owned = new Map(packet.changes.map((row) => [row.path, row]));
  assert.equal(owned.size, 3);
  for (let index = 0; index < before.length; index++) {
    assert.equal(after[index].path, before[index].path);
    const change = owned.get(before[index].path);
    if (change) {
      assert.deepEqual(change.beforeRow, before[index]);
      assert.deepEqual(change.afterRow, after[index]);
      assert.deepEqual(
        packet.preparedManifest.current.find((row) => row.path === change.path),
        change.afterRow,
      );
    } else assert.deepEqual(after[index], before[index]);
  }
  for (const change of packet.changes) {
    for (const side of ["before", "after"]) {
      const body = Buffer.from(change[side]);
      const row = change[side + "Row"];
      assert.equal(body.length, row.bytes);
      assert.equal(digest(body), row.sha256);
      assert.equal(
        createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
        row.gitBlob,
      );
      assert.equal(row.mode, "100644");
    }
    let composed = Buffer.from(change.before);
    let previous = composed.length + 1;
    for (const edit of change.edits.toReversed()) {
      const removed = Buffer.from(edit.before);
      assert(edit.offset + removed.length <= previous);
      assert.deepEqual(composed.subarray(edit.offset, edit.offset + removed.length), removed);
      composed = Buffer.concat([
        composed.subarray(0, edit.offset),
        Buffer.from(edit.after),
        composed.subarray(edit.offset + removed.length),
      ]);
      previous = edit.offset;
    }
    assert.deepEqual(composed, Buffer.from(change.after));
    let restored = composed;
    for (const edit of change.edits) {
      const location = edit.offset;
      const added = Buffer.from(edit.after);
      assert.deepEqual(restored.subarray(location, location + added.length), added);
      restored = Buffer.concat([
        restored.subarray(0, location),
        Buffer.from(edit.before),
        restored.subarray(location + added.length),
      ]);
      // Restoring each preceding edit returns later offsets to the original source.
    }
    assert.deepEqual(restored, Buffer.from(change.before));
  }
}
void test("three reviewed whole source inverses preserve all 320 other current rows and historical authorities", () => {
  validate(receipt);
  for (const change of receipt.changes) {
    const filename = path.join(root, change.path);
    const stat = fs.lstatSync(filename);
    assert(stat.isFile() && !stat.isSymbolicLink());
    assert.equal(stat.mode & 0o777, 0o644);
    assert.deepEqual(fs.readFileSync(filename), Buffer.from(change.after));
  }
});
void test("whole composition refuses extra, omitted, reordered and mutated closed source rows", () => {
  const mutations = [
    (packet) => packet.afterManifest.current.pop(),
    (packet) => packet.afterManifest.current.reverse(),
    (packet) => packet.afterManifest.current.push(packet.afterManifest.current[0]),
    (packet) => {
      packet.afterManifest.current[0].sha256 = "0".repeat(64);
    },
    (packet) => {
      packet.afterManifest.literalFrames[0].sha256 = "0".repeat(64);
    },
    (packet) => packet.changes.pop(),
    (packet) => {
      packet.changes[0].after += "foreign";
    },
    (packet) => {
      packet.changes[0].edits[0].before += "foreign";
    },
    (packet) => {
      packet.changes[0].edits[0].offset++;
    },
    (packet) => {
      packet.native = true;
    },
  ];
  for (const mutate of mutations) {
    const changed = structuredClone(receipt);
    mutate(changed);
    assert.throws(() => validate(changed));
  }
});

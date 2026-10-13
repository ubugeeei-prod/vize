import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { gunzipSync } from "node:zlib";
import {
  assertJoinCommitted,
  joinGit,
  joinIdentity,
  joinPhysical,
  joinRawBodies,
  joinSHA,
  joinObjectID,
} from "./support/no-mutating-component-join-physical.mjs";
import {
  authenticateNoMutatingComponentJoin,
  authenticateNoMutatingComponentJoinCommitted,
} from "./support/no-mutating-component-join.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const filename =
  "tests/_fixtures/differential/linter/computed-composition-source-successor.json.gz";
const sealed = fs.readFileSync(path.join(root, filename));
assert.equal(joinSHA(sealed), "eb2bf3704e58d5e104d016c5fabc7746632f893f88ccbd53158b1a5d3f7617ca");
const packet = JSON.parse(gunzipSync(sealed));
const tsv = "docs/davinci/plan/consumer-migration-surfaces/linter/vize_patina.tsv";
function decode(row) {
  const body = Buffer.from(row.base64, "base64");
  assert.equal(body.toString("base64"), row.base64);
  return joinIdentity(body, row);
}
const identity = ({ base64: _base64, ...row }) => row;
function cohort(manifest) {
  return [...manifest.current, ...manifest.literalFrames, manifest.externalMaintenanceLaw].toSorted(
    (a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0),
  );
}
function validate(value, current = false) {
  assert.equal(value.schema, "vize.patina.computed.documentary-successor");
  assert.equal(value.version, 1);
  assert.equal(value.base, "6401289e3091def8f6bdfd271ab31837e5f36855");
  assert.equal(value.kind, "composition");
  assert.deepEqual(value.credit, {
    sourceOnly: true,
    native: false,
    performance: false,
    old400: false,
  });
  const oldManifestBody = decode(value.beforeManifest);
  assert.equal(
    joinSHA(oldManifestBody),
    "b447560bba3ab9ff244218986d8730474eba84a7d0ae2d2dde8c959846324d42",
  );
  const before = JSON.parse(oldManifestBody);
  const after = JSON.parse(decode(value.afterManifest));
  const { current: oldRows, ...oldRest } = before;
  const { current: newRows, ...newRest } = after;
  assert.deepEqual(
    newRest,
    oldRest,
    "all immutable pins, frames, controls and credit remain whole",
  );
  assert.equal(oldRows.length, 323);
  assert.equal(newRows.length, 323);
  assert.equal(cohort(before).length, 326);
  assert.deepEqual(value.inventory.before.path, tsv);
  assert.deepEqual(value.inventory.after.path, tsv);
  const oldInventory = decode(value.inventory.before);
  const newInventory = decode(value.inventory.after);
  assert.equal(oldInventory.length, 161345);
  assert.equal(
    joinSHA(oldInventory),
    "2e6aea481b67658a0a55f8b8df79bba8b79b73542f8020072f1e0a4b52d82b05",
  );
  for (let index = 0; index < oldRows.length; index++) {
    assert.equal(newRows[index].path, oldRows[index].path);
    if (oldRows[index].path === tsv) {
      assert.deepEqual(oldRows[index], identity(value.inventory.before));
      assert.deepEqual(newRows[index], identity(value.inventory.after));
    } else
      assert.deepEqual(newRows[index], oldRows[index], "all 322 remaining current rows unchanged");
  }
  let forward = oldInventory;
  for (const edit of value.inventory.clauses.toReversed()) {
    const old = Buffer.from(edit.before);
    assert.deepEqual(forward.subarray(edit.offset, edit.offset + old.length), old);
    forward = Buffer.concat([
      forward.subarray(0, edit.offset),
      Buffer.from(edit.after),
      forward.subarray(edit.offset + old.length),
    ]);
  }
  assert.deepEqual(forward, newInventory, "complete forward raw OXC inventory");
  let inverse = newInventory;
  for (const edit of value.inventory.clauses) {
    const added = Buffer.from(edit.after);
    assert.deepEqual(inverse.subarray(edit.offset, edit.offset + added.length), added);
    inverse = Buffer.concat([
      inverse.subarray(0, edit.offset),
      Buffer.from(edit.before),
      inverse.subarray(edit.offset + added.length),
    ]);
  }
  assert.deepEqual(
    inverse,
    oldInventory,
    "whole predecessor TSV inverse, including every neighbor",
  );
  assert.deepEqual(
    value.pins.map(({ before: row }) => row.path),
    [
      "tests/tooling/support/no-mutating-component-join.mjs",
      "tests/tooling/no-mutating-component-join-source.test.mjs",
    ],
  );
  const oldHashes = [joinSHA(oldManifestBody), value.pins[0].before.sha256];
  const newHashes = [value.afterManifest.sha256, value.pins[0].after.sha256];
  for (const [index, pair] of value.pins.entries()) {
    assert.equal(pair.before.path, pair.after.path);
    const old = decode(pair.before).toString();
    const body = decode(pair.after).toString();
    assert.equal(old.split(oldHashes[index]).length, 2);
    assert.equal(body, old.replace(oldHashes[index], newHashes[index]));
    assert.equal(
      body.replace(newHashes[index], oldHashes[index]),
      old,
      "whole predecessor helper/law and every refusal control",
    );
  }
  assert(value.producers.length > 0);
  assert.equal(new Set(value.producers.map(({ path: file }) => file)).size, value.producers.length);
  for (const producer of value.producers) {
    assert(producer.path.startsWith("crates/vize_patina/"));
    assert(!oldRows.some(({ path: file }) => file === producer.path));
    assert.equal(producer.path, producer.after.path);
    decode(producer.after);
    if (producer.before) {
      assert.equal(producer.path, producer.before.path);
      decode(producer.before);
    }
  }
  if (!current) return;
  const predecessor = [
    ...cohort(before),
    identity(value.beforeManifest),
    ...value.pins.map(({ before: row }) => identity(row)),
    ...value.producers.filter(({ before: row }) => row).map(({ before: row }) => identity(row)),
  ].toSorted((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  joinRawBodies(root, value.base, predecessor);
  for (const producer of value.producers.filter(({ before: row }) => !row))
    assert.equal(
      joinGit(root, ["ls-tree", "-z", value.base, "--", producer.path]).length,
      0,
      "genuine producer predecessor absence",
    );
  const rows = [
    ...cohort(after),
    identity(value.afterManifest),
    ...value.pins.map(({ after: row }) => identity(row)),
    ...value.producers.map(({ after: row }) => identity(row)),
  ];
  for (const file of [filename, "tests/tooling/computed-documentary-source.test.mjs"]) {
    const body = fs.readFileSync(path.join(root, file));
    rows.push({
      path: file,
      mode: "100644",
      bytes: body.length,
      sha256: joinSHA(body),
      gitBlob: joinObjectID("blob", body),
    });
  }
  rows.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  assert.equal(new Set(rows.map(({ path: file }) => file)).size, rows.length);
  for (const row of rows) joinPhysical(root, row);
  const head = assertJoinCommitted(root, rows);
  joinRawBodies(root, head, rows);
  assert.equal(authenticateNoMutatingComponentJoinCommitted(root), head);
  const qualified = authenticateNoMutatingComponentJoin(root);
  assert.equal(qualified.completeCurrentClosedBodies, 323);
  assert.equal(qualified.frames.length, 2);
  assert.equal(qualified.native, false);
  assert.equal(qualified.performance, false);
  assert.equal(qualified.oldMigrationMeasurement, null);
  assert.equal(qualified.oldSetupMeasurement, null);
  assert.equal(qualified.currentOld111Receipt, null);
}
void test("whole current computed inventory and producer closure preserve the complete predecessor source law", () =>
  validate(packet, true));
void test("missing, hybrid, unrelated inventory, changed pins, producer bytes and inherited credit refuse", () => {
  for (const mutate of [
    (value) => {
      value.afterManifest.base64 += "foreign";
    },
    (value) => {
      value.inventory.clauses.pop();
    },
    (value) => {
      value.inventory.clauses[0].offset++;
    },
    (value) => {
      value.inventory.after.bytes++;
    },
    (value) => {
      value.pins.pop();
    },
    (value) => {
      value.pins[0].after.base64 += "foreign";
    },
    (value) => {
      value.producers[0].after.bytes++;
    },
    (value) => {
      value.credit.old400 = true;
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

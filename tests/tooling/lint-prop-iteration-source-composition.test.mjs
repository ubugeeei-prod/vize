import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = "tests/_fixtures/ci/lint-prop-iteration-source-composition/proof.json";
const digest = "94f8cdc93f2525a01af35976f52009599ae5491f3198fa56aea6615a7d15208e";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1")
    .update("blob " + body.length + "\0")
    .update(body)
    .digest("hex");
const bytes = fs.readFileSync(path.join(root, fixture));
assert.equal(sha(bytes), digest, "complete immutable cumulative authority");
const original = JSON.parse(bytes);
function decode(cell) {
  const body = Buffer.from(cell.body, "base64");
  assert.equal(body.toString("base64"), cell.body);
  assert.equal(body.length, cell.bytes);
  assert.equal(sha(body), cell.sha256);
  assert.equal(blob(body), cell.blob);
  return body;
}
function inverse(body, remove, restore) {
  assert(remove.length > 0);
  const offset = body.indexOf(remove);
  assert(offset >= 0);
  assert.equal(body.indexOf(remove, offset + 1), -1);
  return Buffer.concat([body.subarray(0, offset), restore, body.subarray(offset + remove.length)]);
}
function verify(proof) {
  assert.deepEqual(proof, original, "only this complete reviewed parent and child composition");
  assert.deepEqual(proof.credit, {
    sourceOnly: true,
    native: false,
    compiled: false,
    performance: false,
    matrix: false,
  });
  const frames = Object.fromEntries(
    Object.entries(proof.frames).map(([name, cell]) => [name, JSON.parse(decode(cell))]),
  );
  const metadata = (frame) =>
    Object.fromEntries(Object.entries(frame).filter(([k]) => k !== "current"));
  for (const frame of Object.values(frames)) {
    assert.deepEqual(metadata(frame), metadata(frames.base));
    assert.equal(frame.current.length, 323);
    assert.deepEqual(
      frame.current.map((row) => row.path),
      frames.base.current.map((row) => row.path),
    );
  }
  const changed = frames.after.current.filter(
    (row, index) => JSON.stringify(row) !== JSON.stringify(frames.parent.current[index]),
  );
  assert.deepEqual(
    changed.map((row) => row.path),
    proof.changes.map((row) => row.path),
  );
  assert.equal(changed.length, 6);
  for (const change of proof.changes) {
    let body = decode(change.bodies.after);
    decode(change.bodies.base);
    decode(change.bodies.own);
    const parent = decode(change.bodies.parent);
    for (const edit of change.inverseEdits)
      body = inverse(body, Buffer.from(edit.remove, "base64"), Buffer.from(edit.restore, "base64"));
    assert.deepEqual(body, parent, "whole incoming parent retained: " + change.path);
    const row = changed.find((entry) => entry.path === change.path);
    assert.equal(row.sha256, change.bodies.after.sha256);
    assert.equal(row.bytes, change.bodies.after.bytes);
    assert.equal(row.gitBlob, change.bodies.after.blob);
  }
  for (const row of proof.checksumInverses) {
    const before = decode(row.before);
    const after = decode(row.after);
    assert.deepEqual(
      inverse(after, Buffer.from(row.afterDigest), Buffer.from(row.beforeDigest)),
      before,
      "complete original helper/law changed only its one coupled checksum",
    );
  }
}
void test("the sealed 323-body composition retains the complete incoming iteration authority", () => {
  verify(original);
});
void test("partial, reordered, foreign, body-edited and credited composition declarations refuse", () => {
  const mutations = [
    (p) => p.changes.pop(),
    (p) => p.changes.reverse(),
    (p) => p.changes.push(structuredClone(p.changes[0])),
    (p) => {
      p.parent = p.own;
    },
    (p) => {
      p.frames.after.body += "AA==";
    },
    (p) => {
      p.changes[0].inverseEdits[0].restore = "AA==";
    },
    (p) => {
      p.checksumInverses[0].afterDigest = "0".repeat(64);
    },
    (p) => {
      p.credit.native = true;
    },
    (p) => {
      p.credit.matrix = true;
    },
  ];
  for (const mutate of mutations) {
    const candidate = structuredClone(original);
    mutate(candidate);
    assert.throws(() => verify(candidate));
  }
});

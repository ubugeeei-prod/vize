import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
const root = path.resolve(import.meta.dirname, "../..");
const filename =
  "tests/_fixtures/differential/linter/component-positive-template-reads/source-custody/stack-composition.json.gz";
const bytes = fs.readFileSync(path.join(root, filename));
const sha = (body) => createHash("sha256").update(body).digest("hex");
assert.equal(sha(bytes), "2d2559e7143f5b0dfc93eefdcad7535210928939e0c26895f2b87e12f9a132f4");
const original = JSON.parse(gunzipSync(bytes));
function decode(cell) {
  assert.equal(cell.mode, "100644");
  const body = Buffer.from(cell.body, "base64");
  assert.equal(body.toString("base64"), cell.body);
  assert.equal(body.length, cell.bytes);
  assert.equal(sha(body), cell.sha256);
  assert.equal(
    createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
    cell.gitBlob,
  );
  return body;
}
const wanted = [
  "crates/vize_croquis/src/binding_occurrences.rs",
  "crates/vize_croquis/src/drawer/occurrences.rs",
  "crates/vize_croquis/src/setup_context/ssr.rs",
  "crates/vize_patina/src/linter/engine/rule_sets.rs",
  "docs/davinci/plan/consumer-migration-surfaces/linter/vize_patina.tsv",
];
function validate(proof) {
  assert.deepEqual(proof, original, "only the frozen complete cumulative composition");
  assert.equal(proof.parent, "efc67bcc6e9dee3cee75918e7e2a1208b3fb7701");
  assert.equal(proof.own, "3be0fd30abe3cb7375b770b53bca2cdb6434f0e8");
  assert.deepEqual(proof.credit, {
    sourceOnly: true,
    native: false,
    performance: false,
    matrix: false,
  });
  const frames = Object.fromEntries(
    Object.entries(proof.frames).map(([side, cell]) => [side, JSON.parse(decode(cell))]),
  );
  const metadata = (frame) =>
    Object.fromEntries(Object.entries(frame).filter(([key]) => key !== "current"));
  for (const frame of Object.values(frames)) {
    assert.equal(frame.current.length, 323);
    assert.deepEqual(
      frame.current.map((row) => row.path),
      frames.parent.current.map((row) => row.path),
    );
    assert.deepEqual(metadata(frame), metadata(frames.parent));
  }
  const changed = frames.after.current.filter(
    (row, i) => JSON.stringify(row) !== JSON.stringify(frames.parent.current[i]),
  );
  assert.deepEqual(
    changed.map((row) => row.path),
    wanted,
  );
  assert.deepEqual(
    proof.changes.map((row) => row.path),
    wanted,
  );
  for (const row of proof.changes) {
    const before = decode(row.bodies.parent),
      own = decode(row.bodies.own),
      after = decode(row.bodies.after);
    let forward = Buffer.from(before),
      inverse = Buffer.from(after);
    for (const [a, b, c, d, encodedOld, encodedNew] of [...row.inverse].reverse()) {
      assert([a, b, c, d].every((value) => Number.isSafeInteger(value) && value >= 0));
      assert(a <= b && c <= d);
      const old = Buffer.from(encodedOld, "base64"),
        next = Buffer.from(encodedNew, "base64");
      assert.deepEqual(forward.subarray(a, b), old);
      assert.deepEqual(inverse.subarray(c, d), next);
      forward = Buffer.concat([forward.subarray(0, a), next, forward.subarray(b)]);
      inverse = Buffer.concat([inverse.subarray(0, c), old, inverse.subarray(d)]);
    }
    assert.deepEqual(forward, after);
    assert.deepEqual(inverse, before, "whole incoming parent retained");
    if (!row.path.endsWith("vize_patina.tsv")) assert.deepEqual(after, own);
    else {
      assert.equal(row.inverse.length, 1);
      const [a, b, , , encodedOld, encodedNew] = row.inverse[0];
      assert.equal(a, b);
      assert.equal(encodedOld, "");
      assert.equal(Buffer.from(encodedNew, "base64").toString().trimEnd().split("\n").length, 2);
      assert(
        Buffer.from(encodedNew, "base64")
          .toString()
          .split("\n")
          .filter(Boolean)
          .every((line) => line.includes("component_positive_template_reads.rs")),
      );
    }
    for (const [side, cell] of Object.entries(row.bodies)) {
      const { body: _body, ...identity } = cell;
      assert.deepEqual(
        frames[side].current.find((entry) => entry.path === row.path),
        identity,
      );
    }
  }
  assert.deepEqual(
    proof.pins.map((row) => row.path),
    [
      "tests/tooling/support/no-mutating-component-join.mjs",
      "tests/tooling/no-mutating-component-join-source.test.mjs",
    ],
  );
  for (const [index, row] of proof.pins.entries()) {
    const before = decode(row.parent),
      after = decode(row.after);
    const oldDigest = index === 0 ? proof.frames.parent.sha256 : proof.pins[0].parent.sha256;
    const newDigest = index === 0 ? proof.frames.after.sha256 : proof.pins[0].after.sha256;
    assert.equal(row.oldDigest, oldDigest);
    assert.equal(row.newDigest, newDigest);
    assert.equal(before.toString().split(oldDigest).length, 2);
    assert.equal(before.toString().replace(oldDigest, newDigest), after.toString());
    assert.equal(after.toString().replace(newDigest, oldDigest), before.toString());
  }
}
void test("the complete historical composition preserves all 323 source rows and every inherited frame", () => {
  validate(original);
  // The existing five physical/refusal laws own live qualification for descendants.
});
void test("omitted, duplicated, reordered, foreign and credited composition fields refuse", () => {
  for (const mutate of [
    (value) => value.changes.pop(),
    (value) => value.changes.reverse(),
    (value) => value.changes.push(value.changes[0]),
    (value) => (value.changes[0].bodies.after.body += "foreign"),
    (value) => value.changes[0].inverse[0][0]++,
    (value) => value.pins.pop(),
    (value) => (value.pins[0].newDigest = "foreign"),
    (value) => (value.frames.after.body += "foreign"),
    (value) => (value.credit.native = true),
  ]) {
    const value = structuredClone(original);
    mutate(value);
    assert.throws(() => validate(value));
  }
});

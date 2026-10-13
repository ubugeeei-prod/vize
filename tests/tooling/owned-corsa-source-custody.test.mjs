import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  ownedCorsaSource,
  qualifyOwnedCorsaSource,
} from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { nativeLexicalCompositionBodyPairs } from "../performance/support/warm-type-backed-native-lexical-composition-host.ts";

const root = new URL("../../", import.meta.url);
const capsuleBytes = fs.readFileSync(
  new URL("tests/_fixtures/ci/owned-corsa-source-before.json", root),
);
assert.equal(
  createHash("sha256").update(capsuleBytes).digest("hex"),
  "80fce9f7a5737d4aaf708911310b0e07a9f52801e87d8ffb7f56170c45e4747b",
);
const capsules = JSON.parse(capsuleBytes);
assert.equal(capsules.source, ownedCorsaSource.baseline);
const before = new Map(
  capsules.files.map(({ file, mode, blob, bytes, sha256, rawBase64 }) => {
    const body = Buffer.from(rawBase64, "base64");
    assert.equal(body.toString("base64"), rawBase64);
    assert.equal(body.length, bytes);
    assert.equal(createHash("sha256").update(body).digest("hex"), sha256);
    assert.equal(
      createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      blob,
    );
    return [file, { mode, blob, body }];
  }),
);
assert.equal(new Set(capsules.files.map(({ file }) => file)).size, capsules.files.length);
const actual = new Map();
for (const { file } of ownedCorsaSource.rows) {
  const physical = new URL(file, root);
  const stat = fs.lstatSync(physical);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
  const body = fs.readFileSync(physical);
  const blob = createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
  actual.set(file, { before: before.get(file) ?? null, after: { mode: "100644", blob, body } });
}
const reader = (side, file) => actual.get(file)?.[side] ?? null;
const input = {
  baseline: ownedCorsaSource.baseline,
  production: ownedCorsaSource.rows.map(({ file }) => file),
  alreadyQualified: new Set(),
  reader,
  historicalPairs: nativeLexicalCompositionBodyPairs,
};
const qualify = (change = {}) => qualifyOwnedCorsaSource({ ...input, ...change });

void test("retained whole baseline capsules and physical237 after bodies grant only source authority", () => {
  const value = qualify();
  assert(value);
  assert.equal(value.completeProductionBodies, 237);
  assert.equal(value.originalProductionBodies, 121);
  assert.equal(value.newArchiveMemberBodies, 112);
  assert.equal(value.additiveLicenseBodies, 4);
  assert.equal(value.historicalProduction.length, 121);
  assert.equal(value.runtimeCredit, false);
  assert.equal(value.performanceCredit, false);
});
void test("all474 actual pair sides refuse missing, foreign whole bytes and nonregular/executable identity", () => {
  let sides = 0;
  for (const { file } of ownedCorsaSource.rows)
    for (const side of ["before", "after"]) {
      sides++;
      const original = reader(side, file);
      for (const changed of [
        null,
        { mode: "100644", blob: "0".repeat(40), body: Buffer.from("foreign whole body") },
        ...(original
          ? [
              { ...original, mode: "100755" },
              { ...original, mode: "120000" },
              { ...original, blob: "0".repeat(40) },
            ]
          : []),
      ]) {
        if (changed === original) continue;
        assert.equal(
          qualify({ reader: (s, p) => (s === side && p === file ? changed : reader(s, p)) }),
          null,
          side + ":" + file,
        );
      }
    }
  assert.equal(sides, 474);
});
void test("missing, duplicate, unknown, inherited-only and reversed source footprints refuse", () => {
  assert.equal(qualify({ production: [] }), null);
  assert.equal(qualify({ baseline: "f".repeat(40) }), null);
  assert.equal(qualify({ alreadyQualified: new Set(input.production) }), null);
  assert.equal(qualify({ production: [...input.production, "vendor/corsa/unknown.rs"] }), null);
  for (const file of input.production) {
    assert.equal(qualify({ production: input.production.filter((p) => p !== file) }), null);
    assert.equal(qualify({ production: [...input.production, file] }), null);
  }
  assert.equal(
    qualify({ reader: (side, file) => reader(side === "before" ? "after" : "before", file) }),
    null,
  );
});
void test("old or mixed four-Cargo cells cannot become a weaker projection", () => {
  const value = qualify();
  assert(value);
  for (const file of [
    "Cargo.lock",
    "Cargo.toml",
    "vendor/corsa_core/Cargo.toml",
    "vendor/corsa_lsp/Cargo.toml",
  ])
    for (const side of ["before", "after"])
      if (value.historicalReader(side, file)?.blob !== reader(side, file)?.blob)
        assert.equal(
          qualify({
            reader: (s, p) =>
              s === side && p === file ? value.historicalReader(s, p) : reader(s, p),
          }),
          null,
        );
  for (const [file, before, after] of nativeLexicalCompositionBodyPairs) {
    const projectedBefore = value.historicalReader("before", file);
    const projectedAfter = value.historicalReader("after", file);
    const sha = (body) => createHash("sha256").update(body).digest("hex");
    assert.equal(projectedBefore ? sha(projectedBefore.body) : null, before);
    assert.equal(sha(projectedAfter.body), after);
  }
});
void test("changed original121 tables or partial original membership refuse before projection", () => {
  assert.equal(qualify({ historicalPairs: nativeLexicalCompositionBodyPairs.slice(1) }), null);
  for (const [file] of nativeLexicalCompositionBodyPairs)
    for (const column of [1, 2]) {
      const changed = nativeLexicalCompositionBodyPairs.map((row) =>
        row[0] === file
          ? row.map((value, index) => (index === column ? "f".repeat(64) : value))
          : row,
      );
      assert.equal(qualify({ historicalPairs: changed }), null);
    }
});
void test("historical buffers and metadata cannot mutate stored capsule authority", () => {
  const first = qualify();
  assert(first);
  const record = first.historicalReader("after", "Cargo.lock");
  const body = Buffer.from(record.body);
  record.body.fill(0);
  record.sha256 = "f".repeat(64);
  record.mode = "100755";
  const second = qualify();
  assert(second);
  assert.deepEqual(second.historicalReader("after", "Cargo.lock").body, body);
  assert.equal(second.historicalReader("after", "Cargo.lock").mode, "100644");
  assert.throws(() => {
    ownedCorsaSource.rows[0].after.sha256 = "f".repeat(64);
  }, TypeError);
});

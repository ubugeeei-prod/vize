import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  projectOwnedCorsaTestSources,
  ownedCorsaTestSourceIdentities,
} from "../../tools/support/compat/github/owned-corsa-test-source.mjs";
import { ownedCorsaSource } from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { projectOwnedDualSourceFrame } from "../performance/support/owned-corsa-dual-source.ts";
const root = path.resolve(import.meta.dirname, "../..");
const load = (relative) => JSON.parse(fs.readFileSync(path.join(root, relative)));
const cell = (value) => ({
  mode: value.mode,
  blob: value.blob,
  bytes: value.bytes,
  sha256: value.sha256,
  body: Buffer.from(value.rawBase64, "base64"),
});
const testRows = load("tools/support/compat/github/owned-corsa-test-source.json").entries;
const testCells = (side) => new Map(testRows.map((row) => [row.file, cell(row[side])]));
void test("both whole relocated tests are atomic and preserve the complete old source bodies", () => {
  const old = testCells("retainedBefore"),
    current = testCells("currentAfter");
  assert.equal(
    projectOwnedCorsaTestSources(ownedCorsaSource.rows, (file) => old.get(file)).context,
    null,
  );
  const value = projectOwnedCorsaTestSources(ownedCorsaSource.rows, (file) => current.get(file));
  assert.equal(value.context.completeWholeTestBodies, 2);
  for (const [file, before] of old) assert.deepEqual(value.reader(file).body, before.body);
  for (const [file, after] of current)
    for (const changed of [
      null,
      { ...after, mode: "100755" },
      { ...after, blob: "f".repeat(40) },
      { ...after, body: after.body.subarray(1) },
      { ...after, body: Buffer.concat([after.body, Buffer.from(" ")]) },
      beforeFor(file),
    ]) {
      const copy = new Map(current);
      copy.set(file, changed);
      assert.equal(
        projectOwnedCorsaTestSources(ownedCorsaSource.rows, (name) => copy.get(name)),
        null,
      );
    }
  function beforeFor(file) {
    return old.get(file);
  }
  assert.equal(
    Reflect.set(ownedCorsaTestSourceIdentities.get(testRows[0].file), "blob", "forged"),
    false,
  );
  assert.equal(
    Reflect.set(ownedCorsaTestSourceIdentities, "get", () => null),
    false,
  );
});
void test("all19 original allocation inputs survive the owned13 and complete pinned upstream6 split", () => {
  const source = cell(testRows[0].currentAfter).body.toString();
  const original = cell(testRows[0].retainedBefore).body.toString();
  const paths = (text) => [...text.matchAll(/^    "([^"\n]+\.rs)",$/gmu)].map((match) => match[1]);
  assert.deepEqual(paths(source), paths(original));
  assert.equal(paths(source).length, 19);
  const capture = /r####"(.*)"####;/u.exec(source);
  assert(capture);
  const data = JSON.parse(capture[1]);
  assert.equal(data.revision, "34bb731bdcaf5d4c154596b0c32912cbffd60b18");
  assert.equal(
    data.archiveSHA256,
    "d581851ddaa32172c3e87cb394446cae00814fc069fd6204bb5336b8a1db3f66",
  );
  assert.equal(data.paths.length, 6);
  const captured = new Map();
  for (const row of data.paths) {
    const body = Buffer.from(row.body);
    assert.equal(row.mode, "100644");
    assert.equal(body.length, row.bytes);
    assert.equal(createHash("sha256").update(body).digest("hex"), row.sha256);
    assert.equal(
      createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      row.blob,
    );
    captured.set(row.path, body.toString());
  }
  let actual = 0;
  for (const file of paths(source)) {
    const content =
      captured.get(file) ??
      fs.readFileSync(path.join(root, file.replace(/^src\/core\//u, "vendor/")), "utf8");
    if (!captured.has(file)) actual++;
    const body = content.split("\n#[cfg(test)]")[0];
    assert(!body.includes("format!("), file);
    assert(!/(?<![a-zA-Z0-9_])(?:String|Vec)(?![a-zA-Z0-9_])/u.test(body), file);
  }
  assert.equal(actual, 13);
});
void test("every complete SDK dual successor rejects missing, partial, mixed, mode and physical drift", () => {
  const data = load("tests/performance/support/owned-corsa-dual-source.json");
  const rows = data.parts.flatMap(([name]) => load("tests/performance/support/" + name));
  assert.equal(rows.length, 10);
  const raw = new Map(rows.map((row) => [row.file, cell(row.current)]));
  const physical = (file) => raw.get(file);
  const head = execFileSync("git", ["--no-replace-objects", "rev-parse", "HEAD"], { cwd: root })
    .toString()
    .trim();
  const positive = projectOwnedDualSourceFrame(
    root,
    head,
    data.originalBeforeSource,
    raw,
    physical,
  );
  for (const row of rows)
    assert.deepEqual(positive.raw.get(row.file).body, cell(row.original).body);
  for (const row of rows) {
    for (const changed of [
      null,
      { ...cell(row.current), mode: "100755" },
      { ...cell(row.current), body: Buffer.from("unknown whole source") },
      cell(row.original),
    ]) {
      const changedRaw = new Map(raw);
      changedRaw.set(row.file, changed);
      assert.throws(() =>
        projectOwnedDualSourceFrame(root, head, data.originalBeforeSource, changedRaw, physical),
      );
    }
    assert.throws(() =>
      projectOwnedDualSourceFrame(root, head, data.originalBeforeSource, raw, (file) =>
        file === row.file ? { ...physical(file), blob: "f".repeat(40) } : physical(file),
      ),
    );
  }
});

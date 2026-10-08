import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { playgroundImporter, verifyEventFixtureSource } from "./stock-authority.ts";
import { RawGitRepository } from "../installed-alias-replay/raw-git.ts";
import { originalHead, stockGraph } from "../installed-alias-replay/vue.ts";

const sourceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../../..");
test("different historical lock bytes share only an explicitly authenticated whole importer and exact26 graph", () => {
  const comparison = verifyEventFixtureSource(sourceRoot);
  assert.equal(comparison.lockfiles.wholeBytesEqual, false);
  assert.notEqual(comparison.lockfiles.originalSha256, comparison.lockfiles.eventSha256);
  assert.equal(comparison.playgroundImporter.wholeBytesEqual, true);
  assert.equal(comparison.lockedGraph.wholeRecordsEqual, true);
  assert.equal(comparison.lockedGraph.records.length, 26);
  assert.equal(
    comparison.lockedGraph.records.find((record) => record.name === "vue")?.version,
    "3.6.0-rc.6",
  );
  assert.equal(
    comparison.lockedGraph.records.find((record) => record.name === "typescript")?.version,
    "6.0.3",
  );
  assert.ok(comparison.lockedGraph.algorithm.includes("JSON.stringify(stockGraph(lock))"));
});
test("literal importer authority includes every field and refuses absent or ambiguous roots", () => {
  const lock =
    "lockfileVersion: '9.0'\nimporters:\n  root:\n    value: root\n  playground:\n    dependencies:\n      vue:\n        version: frozen\n    extraContract: retained\n  second:\n    value: other\npackages:\n";
  assert.equal(
    playgroundImporter(lock),
    "  playground:\n    dependencies:\n      vue:\n        version: frozen\n    extraContract: retained\n",
  );
  assert.throws(
    () => playgroundImporter(lock.replace("  playground:", "  missing:")),
    /unique literal/,
  );
  assert.throws(
    () => playgroundImporter(lock.replace("  second:", "  playground:")),
    /unique literal/,
  );
});
test("same Vue version with a modified integrity is a different complete dependency authority", () => {
  const source = new RawGitRepository(sourceRoot)
    .file(originalHead, "pnpm-lock.yaml")
    .toString("utf8");
  const original = stockGraph(source),
    vue = original.find((record) => record.name === "vue")!;
  const changed = source.replace(
    `integrity: ${vue.integrity}`,
    `integrity: ${vue.integrity.replace(/^sha512-(.)/u, (_, first: string) => `sha512-${first === "A" ? "B" : "A"}`)}`,
  );
  const changedGraph = stockGraph(changed);
  assert.equal(changedGraph.find((record) => record.name === "vue")?.version, "3.6.0-rc.6");
  assert.notDeepEqual(changedGraph, original);
});

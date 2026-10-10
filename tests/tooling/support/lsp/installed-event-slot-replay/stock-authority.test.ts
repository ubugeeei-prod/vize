import assert from "node:assert/strict";
import path from "node:path";
import fs from "node:fs";
import { tmpdir } from "node:os";
import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import {
  playgroundImporter,
  verifyEventFixtureSource,
  compareEventFixtureSources,
} from "./stock-authority.ts";
import { originalHead, stockGraph } from "../installed-alias-replay/vue.ts";

const fixtureRoot = new URL(
  "../../../../_fixtures/differential/lsp/installed-event-slot-replay/stock/",
  import.meta.url,
);
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const manifestBytes = fs.readFileSync(new URL("manifest.json.txt", fixtureRoot));
assert.equal(
  digest(manifestBytes),
  "d4b43406d1e871060e6ddbe93f2de4917fd476cfab9f2ae2483f83a451f2bec5",
);
const manifest = JSON.parse(manifestBytes.toString("utf8"));
assert.equal(manifest.runtimeSourceFallback, false);
const originalSources: Record<string, Buffer> = {},
  eventSources: Record<string, Buffer> = {};
for (const record of manifest.records) {
  const gzip = fs.readFileSync(new URL(record.name, fixtureRoot));
  assert.equal(digest(gzip), record.gzipSha256);
  const bytes = gunzipSync(gzip);
  assert.equal(digest(bytes), record.rawSha256);
  (record.head === originalHead ? originalSources : eventSources)[record.file] = bytes;
}
for (const file of Object.keys(originalSources))
  if (file !== "pnpm-lock.yaml") eventSources[file] = originalSources[file];
test("different historical lock bytes share only an explicitly authenticated whole importer and exact26 graph", () => {
  const comparison = compareEventFixtureSources(originalSources, eventSources);
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
  const source = originalSources["pnpm-lock.yaml"].toString("utf8");
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

test("pure fixture inputs cannot replace the mandatory raw repository source authority", () => {
  const root = fs.mkdtempSync(
    path.join(fs.realpathSync(tmpdir()), "event-stock-no-runtime-fallback-"),
  );
  try {
    assert.throws(() => verifyEventFixtureSource(root), /raw Git object read failed/);
  } finally {
    fs.rmSync(root, { recursive: true });
  }
  const changed = { ...eventSources, "playground/package.json": Buffer.from("{}") };
  assert.throws(() => compareEventFixtureSources(originalSources, changed));
});

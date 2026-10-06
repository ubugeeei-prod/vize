import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  PRESERVED_FORMATTER_SOURCE,
  resolvePreservedFormatterSource,
} from "../differential/formatter-history-source-artifact.ts";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("assertion-only source witness rejects owner, archived source and snapshot drift without Git", (t) => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-preserved-source-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const authority = PRESERVED_FORMATTER_SOURCE;
  const snapshots = fs
    .readdirSync(path.join(root, "crates/vize_glyph/tests/snapshots"))
    .filter((name) => name.startsWith("preserve_authored_content__"))
    .map((name) => `crates/vize_glyph/tests/snapshots/${name}`);
  assert.equal(snapshots.length, 8);
  const rootAuthority =
    "tests/_fixtures/differential/formatter-history/current-references-7877.json";
  const rootSnapshots = JSON.parse(fs.readFileSync(path.join(root, rootAuthority))).cases.map(
    (row) => row.historicalSnapshot.path,
  );
  assert.equal(rootSnapshots.length, 2);
  const files = [
    authority.owner,
    authority.originalAsset,
    ...snapshots,
    rootAuthority,
    ...rootSnapshots,
  ];
  for (const relative of files) {
    const target = path.join(scratch, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, relative), target);
  }
  assert(!fs.existsSync(path.join(scratch, ".git")));
  const artifact = { path: authority.owner, sha256: authority.originalSha256 };
  const original = fs.readFileSync(path.join(root, authority.originalAsset));
  assert.equal(original.length, 3907);
  assert(resolvePreservedFormatterSource(scratch, artifact).equals(original));
  assert.equal(resolvePreservedFormatterSource(scratch, { ...artifact, path: "unknown.rs" }), null);
  assert.throws(() =>
    resolvePreservedFormatterSource(scratch, { ...artifact, sha256: "0".repeat(64) }),
  );
  assert.throws(() =>
    resolvePreservedFormatterSource(scratch, { ...artifact, sha256: authority.currentSha256 }),
  );
  for (const relative of files) {
    const target = path.join(scratch, relative);
    const saved = fs.readFileSync(target);
    fs.writeFileSync(target, Buffer.concat([saved, Buffer.from("// drift\n")]));
    assert.throws(() => resolvePreservedFormatterSource(scratch, artifact));
    fs.rmSync(target);
    assert.throws(() => resolvePreservedFormatterSource(scratch, artifact));
    fs.writeFileSync(target, saved);
  }
  const currentFile = path.join(scratch, authority.owner);
  const current = fs.readFileSync(currentFile);
  fs.writeFileSync(
    currentFile,
    current.toString().replace("assert_eq!(formatted.as_str(), expected);", "assert!(true);"),
  );
  assert.throws(() => resolvePreservedFormatterSource(scratch, artifact));
  fs.writeFileSync(currentFile, original);
  assert(resolvePreservedFormatterSource(scratch, artifact).equals(original));
  // Archived authority cannot escape the supplied repository through a symlink.
  fs.writeFileSync(currentFile, current);
  fs.rmSync(path.join(scratch, authority.originalAsset));
  fs.symlinkSync(
    path.join(root, authority.originalAsset),
    path.join(scratch, authority.originalAsset),
  );
  assert.throws(() => resolvePreservedFormatterSource(scratch, artifact));
});

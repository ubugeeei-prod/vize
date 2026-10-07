// Custody only: the named Rust snapshot executes on its normal Actions lane.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { validateDeclarationSnapshotAuthority } from "../differential/formatter-declaration-reference.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("nested declaration snapshot keeps the original whole witness and law", (t) => {
  validateDeclarationSnapshotAuthority(root);
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "css-declaration-snapshot-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const authorityPath = "tests/_fixtures/differential/formatter-history/current-snapshot-7866.json";
  const authority = JSON.parse(fs.readFileSync(path.join(root, authorityPath)));
  for (const relative of [
    authorityPath,
    authority.source.path,
    authority.historicalSnapshot.path,
    authority.currentSnapshot.path,
  ]) {
    const target = path.join(temporary, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, relative), target);
  }
  validateDeclarationSnapshotAuthority(temporary);
  const current = path.join(temporary, authority.currentSnapshot.path);
  const historical = path.join(temporary, authority.historicalSnapshot.path);
  fs.copyFileSync(historical, current);
  assert.throws(() => validateDeclarationSnapshotAuthority(temporary));
  fs.copyFileSync(path.join(root, authority.currentSnapshot.path), current);
  fs.appendFileSync(historical, "foreign bytes");
  assert.throws(() => validateDeclarationSnapshotAuthority(temporary));
});

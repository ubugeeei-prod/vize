// #7928 retains the original JSON source law and permits its six explicit
// whitespace expectation corrections. Original capture pins stay authoritative.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

export const JSON_LAYOUT_SOURCE = {
  owner: "crates/vize_glyph/src/json/tests.rs",
  originalSha256: "6eab822e3e37f81798f1e4254c4a3438f9ee80eead89bd6618fe966e1ecb5f49",
  currentSha256: "b30b6fdaa8a2db9d12c8bbef66daa4d138e20c0f7cf73cd0ab21800daf0de9af",
  originalAsset:
    "tests/_fixtures/differential/formatter-history/source-witnesses/json-tests.19ceb.txt",
  revisions: [
    "19ceb1ad1481497cfeab49eba4643f7e97229743",
    "cc87bb5960ea9e49e82672205df919de58bb4b24",
  ],
} as const;

function preservedSource(root: string) {
  const current = fs.readFileSync(path.join(root, JSON_LAYOUT_SOURCE.owner));
  if (sha256(current) === JSON_LAYOUT_SOURCE.originalSha256) return current;
  assert.equal(sha256(current), JSON_LAYOUT_SOURCE.currentSha256, "current JSON law changed");
  const original = fs.readFileSync(path.join(root, JSON_LAYOUT_SOURCE.originalAsset));
  assert.equal(sha256(original), JSON_LAYOUT_SOURCE.originalSha256, "original JSON law changed");
  const names = [...original.toString().matchAll(/^#\[test\]\nfn ([a-z][a-z0-9_]+)\(/gm)];
  assert.equal(names.length, 22, "complete original JSON law inventory changed");
  for (const [, name] of names) {
    assert(current.toString().includes(`fn ${name}(`), "live original JSON law is missing");
  }
  return original;
}

export function resolvePreservedJsonLayoutSource(
  root: string,
  artifact: { path: string; sha256: string },
) {
  if (
    artifact.path !== JSON_LAYOUT_SOURCE.owner ||
    artifact.sha256 !== JSON_LAYOUT_SOURCE.originalSha256
  )
    return null;
  return preservedSource(root);
}

export function validatePreservedJsonLayoutWitness(
  root: string,
  entry: { path: string; sha256: string; revisions: string[] },
  name: string,
) {
  if (entry.path !== JSON_LAYOUT_SOURCE.owner) return false;
  assert.equal(entry.sha256, JSON_LAYOUT_SOURCE.originalSha256, "original JSON law pin changed");
  assert.deepEqual(
    entry.revisions,
    [...JSON_LAYOUT_SOURCE.revisions],
    "JSON law revisions changed",
  );
  const original = preservedSource(root);
  assert(original.toString().includes(`fn ${name}(`), "unregistered original JSON law");
  return true;
}

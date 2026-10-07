import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { preservedHuggedInterpolationSnapshot } from "./formatter-hugged-interpolation-reference.ts";
import { sha256 } from "./manifest.mjs";

// #7704 changes production entry/indent logic. Frozen capture receipts bind
// the complete cc87 sources; current-law hashes separately prove all original
// assertion bodies are retained. These source assets grant no output credit.
const OWNERS: Record<string, { originalSha: string; currentSha: string; asset: string }> = {
  "crates/vize_glyph/src/style.rs": {
    originalSha: "8e13635eab09e08a32282d372ef5379c22210cc4117fbccf05d164df76233a61",
    currentSha: "5c99c5fd5c17af14f7ba03482a910d7ec1e55c11fa7b98d279acd81bd48db748",
    asset: "tests/_fixtures/differential/formatter-history/source-witnesses/style.cc87.txt",
  },
  "crates/vize_glyph/src/formatter/block_indent.rs": {
    originalSha: "3fb14d8c80972c6222de66e01196e7b0072039688a75722c3ea76bf0addec023",
    currentSha: "6fd2362a08932f4ecbe3ce9607787f960a48111cf4106260f854bc0242a8ffaf",
    asset: "tests/_fixtures/differential/formatter-history/source-witnesses/block_indent.cc87.txt",
  },
};

export function resolvePreservedLayoutSource(
  root: string,
  artifact: { path: string; sha256: string },
) {
  const snapshot = preservedHuggedInterpolationSnapshot(root, artifact);
  if (snapshot) return snapshot;
  const owner = OWNERS[artifact.path];
  if (!owner || artifact.sha256 !== owner.originalSha) return null;
  const current = fs.readFileSync(path.join(root, artifact.path));
  if (sha256(current) === owner.originalSha) return current;
  assert.equal(sha256(current), owner.currentSha, "current layout owner changed");
  const original = fs.readFileSync(path.join(root, owner.asset));
  assert.equal(sha256(original), owner.originalSha, "complete original layout owner changed");
  return original;
}

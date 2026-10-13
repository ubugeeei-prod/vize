import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { preservedHuggedInterpolationSnapshot } from "./formatter-hugged-interpolation-reference.ts";
import { sha256 } from "./manifest.mjs";

// #7704 changes production entry/indent logic. Frozen capture receipts bind
// the complete cc87 sources. These source assets grant no output credit.
const OWNERS: Record<string, { originalSha: string; asset: string }> = {
  "crates/vize_glyph/src/style.rs": {
    originalSha: "8e13635eab09e08a32282d372ef5379c22210cc4117fbccf05d164df76233a61",
    asset: "tests/_fixtures/differential/formatter-history/source-witnesses/style.cc87.txt",
  },
  "crates/vize_glyph/src/formatter/block_indent.rs": {
    originalSha: "3fb14d8c80972c6222de66e01196e7b0072039688a75722c3ea76bf0addec023",
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
  // The live owner is ordinary work after the original was preserved; only the
  // preserved original asset is authoritative.
  const original = fs.readFileSync(path.join(root, owner.asset));
  assert.equal(sha256(original), owner.originalSha, "complete original layout owner changed");
  return original;
}

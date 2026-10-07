import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

// Preserve the audit's original thirteen CLI obligations. The appended #7876
// case is qualified separately and never becomes historical execution credit.
const AUTHORITY = {
  path: "tests/_fixtures/differential/formatter/manifest.json",
  originalSha256: "40acde7c1eba953724a1a05b60f4744948bb5f48687ed43272981e30bf5c65a4",
  currentSha256: "10f2e78e0c4593a4e108cd99f4d20175d405c210c8e7b76fbf47f6edb3a2ae44",
  originalAsset:
    "tests/_fixtures/differential/formatter-regressions/directive-print-width-7876/shared-manifest.eed471b.original.json.txt",
  originalCases: 13,
  addedCase: "formatter/sfc/directive-prefix-print-width",
} as const;

export function preservedDirectiveWidthCliManifest(
  root: string,
  pin: { path: string; sha256: string; cases: number },
  current: Buffer,
): Buffer | null {
  if (pin.path !== AUTHORITY.path) return null;
  assert.equal(pin.sha256, AUTHORITY.originalSha256, "original CLI manifest pin changed");
  assert.equal(pin.cases, AUTHORITY.originalCases, "original CLI case count changed");
  if (sha256(current) === AUTHORITY.originalSha256) return null;
  assert.equal(sha256(current), AUTHORITY.currentSha256, "current CLI manifest changed");
  const base = fs.realpathSync(root);
  const file = fs.realpathSync(path.resolve(base, AUTHORITY.originalAsset));
  const relative = path.relative(base, file);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  const original = fs.readFileSync(file);
  assert.equal(sha256(original), AUTHORITY.originalSha256, "original CLI manifest changed");
  const before = JSON.parse(original.toString());
  const after = JSON.parse(current.toString());
  const { cases: originalCases, ...originalEnvelope } = before;
  const { cases: currentCases, ...currentEnvelope } = after;
  assert.deepEqual(currentEnvelope, originalEnvelope, "CLI envelope changed");
  assert.equal(originalCases.length, AUTHORITY.originalCases);
  assert.equal(currentCases.length, AUTHORITY.originalCases + 1);
  assert.deepEqual(currentCases.slice(0, AUTHORITY.originalCases), originalCases);
  assert.equal(currentCases.at(-1).id, AUTHORITY.addedCase);
  return original;
}

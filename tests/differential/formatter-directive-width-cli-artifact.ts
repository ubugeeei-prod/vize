import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

// Preserve the audit's original thirteen CLI obligations. The three later
// cases are qualified separately and never become historical execution credit.
const AUTHORITY = {
  path: "tests/_fixtures/differential/formatter/manifest.json",
  originalSha256: "40acde7c1eba953724a1a05b60f4744948bb5f48687ed43272981e30bf5c65a4",
  mainSha256: "cb09a449c5b3503418e839f38b1d338dcf5b6c0f9fd6b7f8f154fbfbdcb86da6",
  currentSha256: "6ab3a71cbb9ec75a4073a315c1f9610b12673c60ff2178517aed94cbb60621e0",
  originalAsset:
    "tests/_fixtures/differential/formatter-regressions/directive-print-width-7876/shared-manifest.eed471b.original.json.txt",
  mainAsset:
    "tests/_fixtures/differential/formatter-regressions/directive-print-width-7876/shared-manifest.b1b9895.main.json.txt",
  originalCases: 13,
  mainCases: 15,
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
  function preserved(asset: string, digest: string) {
    const file = fs.realpathSync(path.resolve(base, asset));
    const relative = path.relative(base, file);
    assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
    const bytes = fs.readFileSync(file);
    assert.equal(sha256(bytes), digest, "preserved CLI manifest changed");
    return bytes;
  }
  const original = preserved(AUTHORITY.originalAsset, AUTHORITY.originalSha256);
  const main = JSON.parse(preserved(AUTHORITY.mainAsset, AUTHORITY.mainSha256).toString());
  const before = JSON.parse(original.toString());
  const after = JSON.parse(current.toString());
  const { cases: originalCases, ...originalEnvelope } = before;
  const { cases: mainCases, ...mainEnvelope } = main;
  const { cases: currentCases, ...currentEnvelope } = after;
  assert.deepEqual(currentEnvelope, originalEnvelope, "CLI envelope changed");
  assert.deepEqual(mainEnvelope, originalEnvelope, "main CLI envelope changed");
  assert.equal(originalCases.length, AUTHORITY.originalCases);
  assert.equal(mainCases.length, AUTHORITY.mainCases);
  assert.equal(currentCases.length, AUTHORITY.mainCases + 1);
  assert.deepEqual(mainCases.slice(0, AUTHORITY.originalCases), originalCases);
  assert.deepEqual(
    mainCases.slice(AUTHORITY.originalCases).map((item: any) => item.id),
    [
      "formatter/sfc/script-trailing-line-comment-parens",
      "formatter/sfc/typed-arrow-last-argument",
    ],
  );
  assert.deepEqual(currentCases.slice(0, AUTHORITY.originalCases), originalCases);
  assert.deepEqual(currentCases.slice(0, AUTHORITY.mainCases), mainCases);
  assert.equal(currentCases.at(-1).id, AUTHORITY.addedCase);
  return original;
}

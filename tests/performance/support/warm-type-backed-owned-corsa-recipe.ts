// A complete locked-source successor; the actual runtime recipe stays observable.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { authenticateHistoricalIncomingLock } from "../../tooling/support/native-sdk-incoming-lock.mjs";

type Cell = { mode: string; blob: string; body: Uint8Array } | null;
type Reader = (side: "before" | "after", file: string) => Cell;
const sha = (body: Uint8Array) => createHash("sha256").update(body).digest("hex");
const read = (relative: string, expected: string) => {
  const url = new URL(relative, import.meta.url);
  const stat = fs.lstatSync(url);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
  const body = fs.readFileSync(url);
  assert.equal(sha(body), expected, "whole reviewed recipe metadata");
  return JSON.parse(body.toString());
};
const original = read(
  "../../tooling/fixtures/native-sdk-historical-source.json",
  "899d7c384501209b2322b64416cb4f7be6a9532a46aed6c11e7cc0e7eb5e87ab",
);
const context = read(
  "../../tooling/support/native-sdk-final-source-context.json",
  "c9e0b29dfb08d7e4a933ebe7a25281fb54cc99e418f0417ed9800e8b2348edea",
);
assert.equal(
  context.originalFrameSHA256,
  "899d7c384501209b2322b64416cb4f7be6a9532a46aed6c11e7cc0e7eb5e87ab",
);
const pair = authenticateHistoricalIncomingLock(context, original.currentFrames);
const file = "pnpm-lock.yaml";
function exact(cell: Cell, expected: typeof pair.actualIncomingAfter) {
  return (
    !!cell &&
    cell.mode === expected.mode &&
    cell.blob === expected.blob &&
    cell.body.byteLength === expected.bytes &&
    sha(cell.body) === expected.sha256 &&
    Buffer.from(cell.body).equals(expected.body)
  );
}

/** Called only after the complete237 graph and complete caller bodies authenticate. */
export function qualifyOwnedCorsaPreservedRecipe(reader: Reader) {
  const before = reader("before", file),
    after = reader("after", file);
  if (exact(before, pair.retainedHistoricalAfter) && exact(after, pair.retainedHistoricalAfter))
    return null;
  assert(exact(before, pair.actualIncomingAfter), "complete actual before recipe source");
  assert(exact(after, pair.actualIncomingAfter), "complete actual after recipe source");
  return Object.freeze({
    authority: "exact-two-whole-current-lock-bodies-with-retained-source-counterfactual",
    file,
    actualBeforeSHA256: pair.actualIncomingAfter.sha256,
    actualAfterSHA256: pair.actualIncomingAfter.sha256,
    actualBytes: pair.actualIncomingAfter.bytes,
    actualBlob: pair.actualIncomingAfter.blob,
    retainedHistoricalSHA256: pair.retainedHistoricalAfter.sha256,
    runtimeCredit: false,
    performanceCredit: false,
    dependencyCredit: false,
    toolchainCredit: false,
  });
}
export function preservedRecipeDigest(
  recipe: ReturnType<typeof qualifyOwnedCorsaPreservedRecipe>,
  originalDigest: (side: "before" | "after", file: string) => string | null,
) {
  return (side: "before" | "after", requested: string) =>
    recipe && requested === file
      ? pair.retainedHistoricalAfter.sha256
      : originalDigest(side, requested);
}

/** Whole old source bytes are available only after the exact two real current locks qualify. */
export function preservedRecipeBody(reader: Reader, file: string) {
  assert.equal(file, "pnpm-lock.yaml");
  return qualifyOwnedCorsaPreservedRecipe(reader)
    ? Buffer.from(pair.retainedHistoricalAfter.body)
    : Buffer.from(reader("after", file)!.body);
}

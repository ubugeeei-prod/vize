import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import {
  javascriptSfcBodyPairs,
  javascriptSfcProductionPaths,
} from "../../../tools/support/compat/github/javascript-sfc-source-context.mjs";

type Digest = (side: "before" | "after", file: string) => string | null;

type Cell = { mode: "100644"; blob: string; byteLength: number; sha256: string; body: string };
type Clause = [number, number, number, number, string, string];
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const blob = (body: Buffer) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
function regular(file: URL) {
  const entry = fs.lstatSync(file);
  assert(entry.isFile() && !entry.isSymbolicLink() && (entry.mode & 0o777) === 0o644);
  return fs.readFileSync(file);
}
const correctionPath = "tools/support/compat/github/javascript-sfc-star-alias-source.json";
const correctionSHA256 = "3d765b1da0b37f7d0ffd162e01ca01ec4161eb34d8ff9e8f373b76fc4cb818ca";
const correctionBytes = regular(new URL("../../../" + correctionPath, import.meta.url));
assert.equal(sha(correctionBytes), correctionSHA256, "whole finite Star alias source correction");
const correction = JSON.parse(correctionBytes.toString()) as {
  schema: string;
  version: number;
  sourceOnly: boolean;
  originalJavaScriptPairs: number;
  currentProductionPaths: number;
  pair: { path: string; currentBefore: Cell; currentAfter: Cell; inverse: Clause[] };
  context: Array<{ path: string; before: null; after: Omit<Cell, "body"> }>;
};
assert.equal(correction.schema, "vize.javascript-star-alias.exact-test-source-correction");
assert.equal(correction.version, 1);
assert.equal(correction.sourceOnly, true);
assert.equal(correction.originalJavaScriptPairs, 93);
assert.equal(correction.currentProductionPaths, 30);
assert.equal(
  correction.pair.path,
  "crates/vize_canon/src/batch/virtual_project/star_exports/tests.rs",
);
function decode(cell: Cell) {
  assert.equal(cell.mode, "100644");
  const body = Buffer.from(cell.body, "base64");
  assert.equal(body.toString("base64"), cell.body);
  assert.equal(body.length, cell.byteLength);
  assert.equal(sha(body), cell.sha256);
  assert.equal(blob(body), cell.blob);
  return body;
}
const original = decode(correction.pair.currentBefore),
  current = decode(correction.pair.currentAfter);
let forward: Buffer = Buffer.from(original),
  backward: Buffer = Buffer.from(current);
for (const [a, b, c, d, before, after] of [...correction.pair.inverse].reverse()) {
  assert([a, b, c, d].every((value) => Number.isSafeInteger(value) && value >= 0));
  assert(a <= b && c <= d);
  const old = Buffer.from(before, "base64"),
    next = Buffer.from(after, "base64");
  assert.deepEqual(forward.subarray(a, b), old);
  assert.deepEqual(backward.subarray(c, d), next);
  forward = Buffer.concat([forward.subarray(0, a), next, forward.subarray(b)]);
  backward = Buffer.concat([backward.subarray(0, c), old, backward.subarray(d)]);
}
assert.deepEqual(forward, current, "whole alias test forward");
assert.deepEqual(backward, original, "whole alias test inverse");
assert.deepEqual(
  correction.context.map(({ path }) => path),
  [
    "tools/support/compat/github/javascript-sfc-options-source.json",
    "tools/support/compat/github/javascript-sfc-options-source.mjs",
  ],
);
for (const row of correction.context) {
  assert.equal(row.before, null);
  assert.equal(row.after.mode, "100644");
  const body = regular(new URL("../../../" + row.path, import.meta.url));
  assert.equal(body.length, row.after.byteLength);
  assert.equal(blob(body), row.after.blob);
  assert.equal(sha(body), row.after.sha256);
}
/** One immutable named pair; no general source getter or historical loan is exposed. */
export const javascriptSfcStarAliasCorrection = Object.freeze({
  path: correction.pair.path,
  currentBefore: Object.freeze({ ...correction.pair.currentBefore }),
  currentAfter: Object.freeze({ ...correction.pair.currentAfter }),
});
const warmHelper = "tests/performance/support/warm-type-backed-javascript-sfc-host.ts";
/** The whole helper pin is external Star custody; the loaded snapshot avoids a self-hash cycle. */
export const javascriptSfcStarAliasMetadataPairs = Object.freeze([
  ...correction.context.map((row) => Object.freeze([row.path, null, row.after.sha256] as const)),
  Object.freeze([correctionPath, null, correctionSHA256] as const),
  Object.freeze([warmHelper, null, sha(regular(new URL(import.meta.url)))] as const),
]);

/** Original29/JS93 stays closed; actual30 additionally requires its one exact named pair. */
export function qualifyJavaScriptSfcHost(
  production: string[],
  qualified: ReadonlySet<string>,
  digest: Digest,
) {
  if (new Set(production).size !== production.length) return null;
  const alias = javascriptSfcStarAliasCorrection;
  const composed = production.includes(alias.path);
  const files = new Set([...javascriptSfcProductionPaths, ...(composed ? [alias.path] : [])]);
  if (production.every((file) => qualified.has(file))) return null;
  if (production.some((file) => !qualified.has(file) && !files.has(file))) return null;
  if (
    JSON.stringify(
      production.filter((file) => files.has(file)).toSorted((a, b) => a.localeCompare(b)),
    ) !== JSON.stringify([...files].toSorted((a, b) => a.localeCompare(b)))
  )
    return null;
  for (const [file, before, after] of javascriptSfcBodyPairs)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  if (composed) {
    if (
      digest("before", alias.path) !== alias.currentBefore.sha256 ||
      digest("after", alias.path) !== alias.currentAfter.sha256
    )
      return null;
    for (const [file, before, after] of javascriptSfcStarAliasMetadataPairs)
      if (digest("before", file) !== before || digest("after", file) !== after) return null;
  }
  return {
    authority: "complete-javascript-sfc-source-pairs",
    completeBodyPairs: javascriptSfcBodyPairs.length,
    ...(composed ? { starAliasTestPairs: 1 } : {}),
    runtimeCredit: false,
    original400Credit: false,
    performanceCredit: false,
    files: [...files],
  };
}

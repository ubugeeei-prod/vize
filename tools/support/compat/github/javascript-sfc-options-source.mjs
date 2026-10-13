import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const url = new URL("./javascript-sfc-options-source.json", import.meta.url);
const stat = fs.lstatSync(url);
assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o644);
const bytes = fs.readFileSync(url);
assert.equal(sha(bytes), "823c3fb00c9fdcedfcead6bc1fa9cf7fb038fedc26b94e957b1069bd4c927f2c");
const data = JSON.parse(bytes);
assert.equal(data.schema, "vize.javascript-options.complete-source-successor");
assert.equal(data.version, 1);
assert.equal(data.baseBodyPairs, 88);
assert.equal(data.currentBodyPairs, 93);
assert.equal(data.sourceOnly, true);
assert.deepEqual(
  data.existing.map(({ path }) => path),
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues.rs",
    "crates/vize_canon/src/batch/virtual_project/build.rs",
    "crates/vize_canon/src/batch/virtual_project/project/artifacts.rs",
    "tools/support/compat/github/native-javascript-sfc-membership.sh",
  ],
);
assert.equal(data.supplemental.length, 5);
assert.equal(new Set(data.supplemental.map(({ path }) => path)).size, 5);
assert.equal(data.supplemental.filter(({ before }) => before === null).length, 4);
const control = "crates/vize_canon/src/batch/type_checker/tests/options_api_required_props.rs";
assert.deepEqual(
  data.supplemental.find(({ path }) => path === control).before,
  data.supplemental.find(({ path }) => path === control).after,
);
function decode(cell) {
  if (cell === null) return null;
  assert.deepEqual(
    Object.keys(cell).toSorted((a, b) => a.localeCompare(b)),
    ["blob", "body", "byteLength", "sha256"],
  );
  const body = Buffer.from(cell.body, "base64");
  assert.equal(body.toString("base64"), cell.body);
  assert.equal(body.length, cell.byteLength);
  assert.equal(sha(body), cell.sha256);
  assert.equal(blob(body), cell.blob);
  return body;
}
const retained = new Map();
let exactCurrent = null;
for (const row of data.existing) {
  const before = decode(row.retainedAfter),
    after = decode(row.currentAfter);
  let forward = Buffer.from(before),
    inverse = Buffer.from(after);
  for (const [oldStart, oldEnd, newStart, newEnd, oldEncoded, newEncoded] of [
    ...row.inverse,
  ].reverse()) {
    const old = Buffer.from(oldEncoded, "base64"),
      current = Buffer.from(newEncoded, "base64");
    assert.deepEqual(forward.subarray(oldStart, oldEnd), old);
    assert.deepEqual(inverse.subarray(newStart, newEnd), current);
    forward = Buffer.concat([forward.subarray(0, oldStart), current, forward.subarray(oldEnd)]);
    inverse = Buffer.concat([inverse.subarray(0, newStart), old, inverse.subarray(newEnd)]);
  }
  assert.deepEqual(forward, after, "whole source forward: " + row.path);
  assert.deepEqual(inverse, before, "whole source inverse: " + row.path);
  retained.set(row.path, Buffer.from(before));
}
for (const row of data.supplemental) {
  decode(row.before);
  decode(row.after);
  assert(row.after && !data.existing.some(({ path }) => path === row.path));
}
export const javascriptSfcOptionsContextPaths = Object.freeze([
  "tools/support/compat/github/javascript-sfc-options-source.mjs",
  "tools/support/compat/github/javascript-sfc-options-source.json",
]);
export const javascriptSfcOptionsProductionPaths = Object.freeze([...data.productionAdditions]);
/** The immutable original88 table and event assertion are authenticated before exact source overlay. */
export function javascriptSfcOptionsRows(rows) {
  assert.equal(rows.length, 88);
  assert.equal(new Set(rows.map(({ path }) => path)).size, 88);
  assert.equal(sha(JSON.stringify(rows)), data.baseRowsSHA256);
  for (const entry of data.existing) {
    const row = rows.find(({ path }) => path === entry.path);
    assert(row);
    assert.deepEqual(row.after, entry.retainedAfter);
  }
  const current = rows.map((row) => {
    const entry = data.existing.find(({ path }) => path === row.path);
    return entry ? { ...row, after: structuredClone(entry.currentAfter) } : row;
  });
  for (const row of data.supplemental) {
    assert(!current.some(({ path }) => path === row.path));
    current.push(structuredClone(row));
  }
  assert.equal(current.length, 93);
  exactCurrent = structuredClone(current);
  return structuredClone(current);
}
/** Full actual93 authentication owns this source-only retained model; no runtime or400 credit. */
export function retainedJavaScriptSfcOptionsAfter(file, actual) {
  assert(
    exactCurrent && actual instanceof Map && actual.size === 93,
    "one complete current93 source model",
  );
  for (const row of exactCurrent) {
    assert(Buffer.isBuffer(actual.get(row.path)), row.path);
    assert.deepEqual(actual.get(row.path), decode(row.after), row.path);
  }
  assert(actual.has(file), "complete current source body required: " + file);
  return Buffer.from(retained.get(file) ?? actual.get(file));
}
export const javascriptSfcOptionsSourceReceipt = Object.freeze({
  authority: "complete-javascript-options-source-successor",
  baseBodyPairs: 88,
  currentBodyPairs: 93,
  correctedExistingBodies: 4,
  supplementalSourceBodies: 5,
  unchangedOriginalOptionsSourceBodies: 1,
  sourceOnly: true,
  runtimeCredit: false,
  original400Credit: false,
  performanceCredit: false,
});

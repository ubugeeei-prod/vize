import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const object = (kind, body) =>
  createHash("sha1").update(`${kind} ${body.length}\0`).update(body).digest("hex");
const file = new URL("./javascript-sfc-star-source.json", import.meta.url);
const stat = fs.lstatSync(file);
assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o644);
const bytes = fs.readFileSync(file);
assert.equal(sha(bytes), "e202062b0362e21f5b7847ab70911b3d3f7c7e5c7ca7c6d51fb88ca35944a56f");
const manifest = JSON.parse(bytes);
assert.equal(manifest.parts.length, 6);
const cells = manifest.parts.map(([name, checksum], index) => {
  assert.equal(name, `javascript-sfc-star-source-${String(index).padStart(2, "0")}.json`);
  const part = new URL(name, import.meta.url);
  const entry = fs.lstatSync(part);
  assert(entry.isFile() && !entry.isSymbolicLink() && (entry.mode & 0o777) === 0o644);
  const body = fs.readFileSync(part);
  assert.equal(sha(body), checksum);
  return JSON.parse(body);
});
const data = { ...manifest, cells };
assert.equal(data.schema, "vize.javascript-star.complete-current-source-context");
assert.equal(data.version, 1);
assert.equal(data.sourceOnly, true);
assert.equal(data.bodyPairs, 93);
assert.equal(data.beforeSource, "3a680d0a0b59b720af0d23a70a85cf56c5c4f175");
assert.deepEqual(
  data.cells.map(({ path }) => path),
  [
    ".github/workflows/typechecker-native-phases.yml",
    "crates/vize_canon/src/batch/virtual_project/build.rs",
    "crates/vize_canon/src/batch/virtual_project/module_projection.rs",
    "crates/vize_canon/src/batch/virtual_project/project/artifacts.rs",
    "docs/davinci/plan/consumer-migration-surfaces/typechecker/vize_canon.tsv",
    "tests/performance/support/warm-type-backed-host-qualifications.ts",
  ],
);
const commit = Buffer.from(data.beforeCommit.rawBase64, "base64");
assert.equal(commit.toString("base64"), data.beforeCommit.rawBase64);
assert.equal(commit.length, data.beforeCommit.byteLength);
assert.equal(sha(commit), data.beforeCommit.sha256);
assert.equal(object("commit", commit), data.beforeSource);
assert(commit.toString().startsWith("tree 577d549f8a9c6b0a51f6bf99bc23fb35eec5942a\n"));
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
  assert.equal(object("blob", body), cell.blob);
  return body;
}
function inverse(before, after, clauses, name) {
  let forward = Buffer.from(before),
    reversed = Buffer.from(after);
  for (const [i, j, k, l, oldEncoded, newEncoded] of [...clauses].reverse()) {
    for (const n of [i, j, k, l]) assert(Number.isSafeInteger(n) && n >= 0);
    assert(i <= j && k <= l);
    const old = Buffer.from(oldEncoded, "base64"),
      current = Buffer.from(newEncoded, "base64");
    assert.deepEqual(forward.subarray(i, j), old, name);
    assert.deepEqual(reversed.subarray(k, l), current, name);
    forward = Buffer.concat([forward.subarray(0, i), current, forward.subarray(j)]);
    reversed = Buffer.concat([reversed.subarray(0, k), old, reversed.subarray(l)]);
  }
  assert.deepEqual(forward, after, "complete context forward: " + name);
  assert.deepEqual(reversed, before, "complete context inverse: " + name);
}
for (const row of data.cells) {
  decode(row.retainedBefore);
  const currentBefore = decode(row.currentBefore),
    optionsAfter = decode(row.optionsAfter),
    currentAfter = decode(row.currentAfter);
  assert(currentBefore && optionsAfter && currentAfter);
  inverse(optionsAfter, currentAfter, row.optionsInverse, row.path);
  inverse(currentBefore, currentAfter, row.starInverse, row.path);
}
const wholeRowsSHA = (rows) =>
  sha(
    JSON.stringify(
      rows.map(({ path, before, after }) => [
        path,
        ...[before, after].map((cell) =>
          cell === null ? null : [cell.byteLength, cell.sha256, cell.blob, cell.body],
        ),
      ]),
    ),
  );
const wholeBeforeSHA = (rows) =>
  sha(
    JSON.stringify(
      rows.map(({ path, before }) => [
        path,
        before === null ? null : [before.byteLength, before.sha256, before.blob, before.body],
      ]),
    ),
  );
let exactJoined = null,
  exactOptions = null;
export const javascriptSfcStarContextPaths = Object.freeze([
  "tools/support/compat/github/javascript-sfc-star-source.mjs",
  "tools/support/compat/github/javascript-sfc-star-source.json",
  ...manifest.parts.map(([name]) => "tools/support/compat/github/" + name),
]);
/** Passive whole source capsules grant no current or historical qualification. */
export const javascriptSfcStarContextCells = () => structuredClone(data.cells);

/** Keep every original88/Options93 cell immutable; this is one exact six-body context. */
export function javascriptSfcStarRows(options) {
  assert.equal(options.length, 93);
  for (const row of options) {
    assert.deepEqual(
      Object.keys(row).toSorted((a, b) => a.localeCompare(b)),
      ["after", "before", "path"],
    );
    decode(row.before);
    decode(row.after);
  }
  assert.equal(new Set(options.map(({ path }) => path)).size, 93);
  assert.equal(wholeRowsSHA(options), data.optionsRowsSHA256);
  const joined = options.map((row) => {
    const cell = data.cells.find(({ path }) => path === row.path);
    if (!cell) return structuredClone(row);
    assert.deepEqual(row.before, cell.retainedBefore);
    assert.deepEqual(row.after, cell.optionsAfter);
    return {
      path: row.path,
      before: structuredClone(cell.currentBefore),
      after: structuredClone(cell.currentAfter),
    };
  });
  assert.equal(wholeBeforeSHA(joined), data.currentBeforeRowsSHA256);
  exactJoined = structuredClone(joined);
  exactOptions = structuredClone(options);
  return structuredClone(joined);
}
/** A complete current93 body map must succeed before any Options93 historical model is exposed. */
export function retainedJavaScriptSfcStarOptions(actual) {
  assert(exactJoined && exactOptions && actual instanceof Map && actual.size === 93);
  for (const row of exactJoined) {
    assert(Buffer.isBuffer(actual.get(row.path)), row.path);
    assert.deepEqual(actual.get(row.path), decode(row.after), row.path);
  }
  return new Map(exactOptions.map(({ path, after }) => [path, Buffer.from(decode(after))]));
}
export const javascriptSfcStarSourceReceipt = Object.freeze({
  authority: "complete-javascript-star-current-source-context",
  bodyPairs: 93,
  beforeSource: data.beforeSource,
  wholeContextSourceBodies: 6,
  retainedOriginalSourceBodies: 88,
  retainedOptionsSourceBodies: 93,
  sourceOnly: true,
  runtimeCredit: false,
  original400Credit: false,
  performanceCredit: false,
});

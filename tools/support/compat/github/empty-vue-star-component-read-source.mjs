import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { after } from "node:test";
import { pathToFileURL } from "node:url";
import {
  joinGit,
  joinObjectID,
  joinPhysical,
  joinRawBodies,
  joinSHA,
} from "../../../../tests/tooling/support/no-mutating-component-join-physical.mjs";
import {
  listStarCounterpartFiles,
  provisionStarSourceCounterpart,
} from "./source-counterpart-snapshot.mjs";

import { ensureHostedLiteralSource } from "../../../../tests/performance/support/warm-type-backed-git-reader-literal-source.ts";

const directory = "tools/support/compat/github/";
const metadataDirectory =
  "tests/_fixtures/differential/linter/component-positive-template-reads/source-custody/";
const source = "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38";
const parts = [
  [
    "empty-vue-star-component-read-source-0.json",
    "db2f6c3c612c77aec5e5d6d0285e41624ec175c5338f533623e13d5c441b8ef3",
    250,
  ],
  [
    "empty-vue-star-component-read-source-1.json",
    "ec5b90ddef5a94e49443a084ccc46d9db91389e0ab5b416a99b11fe7c7a74cc6",
    165,
  ],
];
const cells = [
  "empty-vue-star-component-read-cells.json",
  "05778895222cd80b8cb2c0f6b93ee385aca1f8afe50895025c7f952dd7e3ee1d",
];
const loaded = new Map();
function metadata(name, checksum) {
  const file = metadataDirectory + name;
  const body = fs.readFileSync(new URL("../../../../" + file, import.meta.url));
  assert.equal(joinSHA(body), checksum, "whole named component-read metadata");
  loaded.set(file, body);
  return JSON.parse(body);
}
const rows = parts.flatMap(([name, checksum, count]) => {
  const values = metadata(name, checksum);
  assert.equal(values.length, count);
  return values;
});
const data = metadata(...cells);
assert.equal(data.beforeSource, source);
assert.equal(data.sourceOnly, true);
assert.equal(rows.length, 415);
assert.equal(new Set(rows.map(([file]) => file)).size, 415);
assert.deepEqual(
  rows.map(([file]) => file),
  rows.map(([file]) => file).toSorted((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
);
const changed = rows.filter(([, before, next]) => before.blob !== next.blob);
assert.deepEqual(
  changed.map(([file]) => file),
  data.changes.map(({ path }) => path),
);
function decode(cell) {
  const body = Buffer.from(cell.body, "base64");
  assert.equal(body.toString("base64"), cell.body);
  assert.equal(body.length, cell.byteLength);
  assert.equal(joinSHA(body), cell.sha256);
  assert.equal(joinObjectID("blob", body), cell.blob);
  assert.equal(cell.mode, "100644");
  return body;
}
for (const row of data.changes) {
  const old = decode(row.before),
    next = decode(row.after);
  let forward = Buffer.from(old),
    inverse = Buffer.from(next);
  for (const [a, b, c, d, oldText, nextText] of [...row.inverse].reverse()) {
    assert([a, b, c, d].every((value) => Number.isSafeInteger(value) && value >= 0));
    assert(a <= b && c <= d);
    const previous = Buffer.from(oldText, "base64"),
      following = Buffer.from(nextText, "base64");
    assert.deepEqual(forward.subarray(a, b), previous);
    assert.deepEqual(inverse.subarray(c, d), following);
    forward = Buffer.concat([forward.subarray(0, a), following, forward.subarray(b)]);
    inverse = Buffer.concat([inverse.subarray(0, c), previous, inverse.subarray(d)]);
  }
  assert.deepEqual(forward, next, "whole positive-read forward");
  assert.deepEqual(inverse, old, "whole positive-read inverse");
  const declared = rows.find(([file]) => file === row.path);
  for (const [index, cell] of [
    [1, row.before],
    [2, row.after],
  ]) {
    const { body: _body, ...identity } = cell;
    assert.deepEqual(declared[index], identity);
  }
}
const executing = [
  directory + "empty-vue-star-component-read-source.mjs",
  directory + "source-counterpart-snapshot.mjs",
  "tests/tooling/empty-vue-star-component-read-source.test.mjs",
  "tests/tooling/warm-type-backed-empty-vue-star-custody.test.ts",
];
for (const file of executing)
  loaded.set(file, fs.readFileSync(new URL("../../../../" + file, import.meta.url)));
const identity = (file, body) => ({
  path: file,
  mode: "100644",
  bytes: body.length,
  sha256: joinSHA(body),
  gitBlob: joinObjectID("blob", body),
});
const declared = (side) =>
  rows.map(([file, before, next]) => {
    const cell = side === "before" ? before : next;
    return {
      path: file,
      mode: cell.mode,
      bytes: cell.byteLength,
      sha256: cell.sha256,
      gitBlob: cell.blob,
    };
  });
export const componentReadSourcePaths = Object.freeze(rows.map(([file]) => file));
export function authenticateStarComponentReadSource(root) {
  root = fs.realpathSync(root);
  const head = joinGit(root, ["rev-parse", "HEAD"]).toString().trim();
  assert.equal(joinGit(root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  for (const row of declared("after")) joinPhysical(root, row);
  ensureHostedLiteralSource(root, source);
  const before = joinRawBodies(root, source, declared("before"));
  const current = joinRawBodies(root, head, declared("after"));
  for (const row of data.changes) {
    assert.deepEqual(before.get(row.path), decode(row.before));
    assert.deepEqual(current.get(row.path), decode(row.after));
  }
  const authored = [...loaded]
    .map(([file, body]) => identity(file, body))
    .toSorted((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  joinRawBodies(root, head, authored);
  for (const row of authored) joinPhysical(root, row);
  assert.equal(joinGit(root, ["rev-parse", "HEAD"]).toString().trim(), head);
  return Object.freeze({
    head,
    closureBodies: 415,
    changedBodies: changed.length,
    originalStarBodies: 322,
    originalJavaScriptBodies: 93,
    sourceOnly: true,
    nativeCredit: false,
    original400Credit: false,
    performanceCredit: false,
  });
}
export async function runRetainedComponentReadStarCustody(root) {
  const current = prepareRootComponentReadHistory(root);
  const names = listStarCounterpartFiles(root, source, componentReadSourcePaths);
  const snapshot = provisionStarSourceCounterpart(root, source, names);
  let registered = false;
  try {
    const runner = await import(
      pathToFileURL(path.join(snapshot.root, directory + "empty-vue-star-custody-runner.mjs")).href
    );
    const retained = await runner.runRetainedEmptyVueStarCustody(snapshot.root);
    assert.equal(retained.registeredOriginalTests, 8);
    assert.equal(retained.currentBodies, 322);
    after(() => {
      try {
        assert.equal(authenticateStarComponentReadSource(root).head, current.head);
      } finally {
        snapshot.cleanup();
      }
    });
    registered = true;
    return Object.freeze({ ...current, currentBodies: 322, registeredOriginalTests: 8 });
  } finally {
    if (!registered) snapshot.cleanup();
  }
}
export async function runRetainedComponentReadStarLaws(root) {
  const current = prepareRootComponentReadHistory(root);
  const names = listStarCounterpartFiles(root, source, componentReadSourcePaths);
  const snapshot = provisionStarSourceCounterpart(root, source, names);
  let registered = false;
  try {
    await import(
      pathToFileURL(
        path.join(snapshot.root, "tests/tooling/empty-vue-star-javascript-source.test.mjs"),
      ).href
    );
    after(() => {
      try {
        assert.equal(authenticateStarComponentReadSource(root).head, current.head);
      } finally {
        snapshot.cleanup();
      }
    });
    registered = true;
    return current;
  } finally {
    if (!registered) snapshot.cleanup();
  }
}

/** Literal history is prepared only through the original owned-Actions refusal boundary. */
export function prepareRootComponentReadHistory(root) {
  const current = authenticateStarComponentReadSource(root);
  assert.equal(data.history.length, 14);
  assert.equal(new Set(data.history).size, 14);
  for (const historical of data.history) ensureHostedLiteralSource(root, historical);
  assert.equal(joinGit(root, ["rev-parse", "HEAD"]).toString().trim(), current.head);
  return current;
}

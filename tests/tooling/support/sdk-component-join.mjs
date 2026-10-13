import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  authenticateNoMutatingComponentJoinCommitted,
  authenticateNoMutatingComponentJoinPhysical,
} from "./no-mutating-component-join.mjs";
import {
  joinGit,
  joinIdentity,
  joinObjectID,
  joinPhysical,
  joinRawBodies,
  joinSHA,
} from "./no-mutating-component-join-physical.mjs";
import {
  qualifyAsciiOriginalCallers,
  retainedAsciiCallerFrame,
} from "./sdk-ascii-symbol-original-callers.ts";
import { ensureHostedLiteralSource } from "../../performance/support/warm-type-backed-git-reader-literal-source.ts";
import { ownedCorsaSource } from "../../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { assertOwnedCorsaAfterContext } from "../../../tools/support/compat/github/owned-corsa-clippy-source.mjs";

const stem = "tests/performance/support/";
const driver = "tests/tooling/warm-type-backed-git-reader-nodeep.test.ts";
const alias = "tests/tooling/support/sdk-ascii-symbol-original-callers.ts";
const order = (a, b) => Buffer.compare(Buffer.from(a.path), Buffer.from(b.path));
const cell = (file, body) => ({
  path: file,
  mode: "100644",
  gitBlob: joinObjectID("blob", body),
  bytes: body.length,
  sha256: joinSHA(body),
});
function regular(root, name) {
  const target = path.join(root, name);
  const stat = fs.lstatSync(target);
  assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o644, name);
  assert.equal(fs.realpathSync(target), path.join(fs.realpathSync(root), name));
  return fs.readFileSync(target);
}
const dataBytes = regular(import.meta.dirname, "sdk-component-join.json");
assert.equal(
  joinSHA(dataBytes),
  "227aef4e28ba150eee128c8659293cd441a11766c23047ccfed5b0a48be86a44",
);
const data = JSON.parse(dataBytes);
assert.equal(data.schema, "vize.sdk-component-join.current-source-only");
assert.equal(data.version, 1);
assert.equal(data.sdkSource, "bee0fb6bbb4bea9947c1416f542068c1511657f1");
assert.equal(data.incomingSource, "7d6e0350202643010db3a0f474eb235c8f1042c3");
assert.equal(data.originalDriver, driver);
assert.equal(data.changes.length, 10);
assert.equal(new Set(data.changes.map(({ file }) => file)).size, 10);
const privateBytes = regular(import.meta.dirname, "sdk-component-join-files.json");
assert.equal(joinSHA(privateBytes), data.privateFilesSHA256);
const privateFiles = JSON.parse(privateBytes);
assert.equal(privateFiles.length, 187);
assert.equal(new Set(privateFiles).size, 187);
const ownFiles = [
  "tests/tooling/support/sdk-component-join.mjs",
  "tests/tooling/support/sdk-component-join-replay.mjs",
  "tests/tooling/support/sdk-component-join.json",
  "tests/tooling/support/sdk-component-join-files.json",
  alias,
  "tests/tooling/sdk-ascii-symbol-original-callers.test.ts",
  "tests/tooling/sdk-component-join-source.test.mjs",
];
// Final119 independently pins these modules; this additionally binds a supplied private root
// to the complete regular source snapshot of the executing, enrolled adapter.
const executingRoot = path.resolve(import.meta.dirname, "../../..");
const ownRows = ownFiles.map((file) => cell(file, regular(executingRoot, file))).toSorted(order);
const receipts = new WeakMap();
function rawCommit(root, source) {
  assert.match(source, /^[a-f0-9]{40}$/u);
  ensureHostedLiteralSource(root, source);
  const bytes = joinGit(root, ["cat-file", "commit", source]);
  assert.equal(joinObjectID("commit", bytes), source);
  assert.match(bytes.toString(), /^tree [a-f0-9]{40}\n/u);
  return bytes;
}
function sourceRows(root, source, files) {
  const packet = joinGit(root, ["ls-tree", "-r", "-z", source, "--", ...files]);
  assert.deepEqual(Buffer.from(packet.toString()), packet);
  const entries = packet
    .toString()
    .split("\0")
    .filter(Boolean)
    .map((entry) => {
      const found = /^100644 blob ([a-f0-9]{40})\t(.+)$/u.exec(entry);
      assert(found && files.includes(found[2]), "whole literal regular source");
      return { blob: found[1], file: found[2] };
    });
  assert.equal(entries.length, files.length);
  assert.equal(new Set(entries.map(({ file }) => file)).size, files.length);
  const batch = joinGit(
    root,
    ["cat-file", "--batch"],
    entries.map(({ blob }) => blob).join("\n") + "\n",
  );
  let cursor = 0;
  const rows = entries.map(({ blob, file }) => {
    const end = batch.indexOf(10, cursor);
    assert(end >= cursor);
    const header = /^([a-f0-9]{40}) blob ([0-9]+)$/u.exec(batch.subarray(cursor, end).toString());
    assert(header && header[1] === blob);
    const length = Number(header[2]);
    assert(Number.isSafeInteger(length) && length >= 0);
    const bytes = batch.subarray(end + 1, end + 1 + length);
    assert.equal(bytes.length, length);
    const row = cell(file, bytes);
    assert.equal(row.gitBlob, blob);
    assert.equal(batch[end + 1 + length], 10);
    cursor = end + 2 + length;
    return row;
  });
  assert.equal(cursor, batch.length);
  return rows.toSorted(order);
}
function review(rows, bodies) {
  const metadataFile = stem + "warm-type-backed-git-reader-dual-frame.json";
  const metadataBytes = bodies.get(metadataFile);
  assert.equal(joinSHA(metadataBytes), data.dualManifestSHA256);
  const metadata = JSON.parse(metadataBytes);
  const value = metadata.skeleton;
  assert.deepEqual(
    metadata.parts.map(({ file }) => file),
    Array.from(
      { length: 24 },
      (_, i) => `warm-type-backed-git-reader-dual-frame-part-${String(i).padStart(2, "0")}.json`,
    ),
  );
  for (const part of metadata.parts) {
    const bytes = bodies.get(stem + part.file);
    joinIdentity(bytes, { ...part, path: stem + part.file, gitBlob: part.blob });
    const values = JSON.parse(bytes);
    assert.equal(values.length, part.entries);
    const keys = part.section.split(".");
    const field = keys.pop();
    const owner = keys.reduce((value, key) => value[key], value);
    assert.equal(owner[field].length, part.start);
    owner[field].push(...values);
  }
  assert.equal(joinSHA(Buffer.from(JSON.stringify(value, null, 2) + "\n")), data.dualReviewSHA256);
  assert.equal(value.rows.length, 112);
  assert.equal(new Set(value.rows.map(({ file }) => file)).size, 112);
  for (const row of value.rows) {
    const expected = rows.find(({ path }) => path === row.file);
    assert.deepEqual(expected, {
      path: row.file,
      mode: row.final.mode,
      gitBlob: row.final.blob,
      bytes: row.final.bytes,
      sha256: row.final.sha256,
    });
  }
  return value;
}
export function qualifyCurrentSdkAsciiCallers(root) {
  const expected = retainedAsciiCallerFrame("owned-sdk-reader");
  const rows = expected
    .map(({ file, mode, blob, body }) => ({ ...cell(file, body), mode, gitBlob: blob }))
    .toSorted(order);
  const head = joinGit(root, ["rev-parse", "HEAD"]).toString().trim();
  assert.equal(joinGit(root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  const raw = joinRawBodies(root, head, rows);
  const entries = rows.map((row) => {
    const body = joinPhysical(root, row);
    assert.deepEqual(body, raw.get(row.path));
    return { file: row.path, mode: row.mode, blob: row.gitBlob, body };
  });
  const result = qualifyAsciiOriginalCallers(entries);
  assert.equal(result.kind, "owned-sdk-reader");
  assert.equal(joinGit(root, ["rev-parse", "HEAD"]).toString().trim(), head);
  return { ...result, currentHead: head, actualCurrentBodies: 10 };
}
/** Whole current323, current112 and current237 precede every historical SDK source view. */
export function authenticateSdkComponentJoin(root) {
  const head = joinGit(root, ["rev-parse", "HEAD"]).toString().trim();
  rawCommit(root, head);
  rawCommit(root, data.sdkSource);
  rawCommit(root, data.incomingSource);
  const joinSupport = [
    "no-mutating-component-join.mjs",
    "no-mutating-component-join-physical.mjs",
    "no-mutating-component-join-frame.mjs",
    "no-mutating-component-join-runner.mjs",
    "no-mutating-props-current-caller-frame.mjs",
  ].map((file) => "tests/tooling/support/" + file);
  const originalJoinRows = sourceRows(root, data.incomingSource, joinSupport);
  joinRawBodies(root, head, originalJoinRows);
  for (const row of originalJoinRows) joinPhysical(root, row);
  const ownBodies = joinRawBodies(root, head, ownRows);
  for (const row of ownRows) assert.deepEqual(joinPhysical(root, row), ownBodies.get(row.path));
  const joined = authenticateNoMutatingComponentJoinPhysical(root);
  const current = authenticateNoMutatingComponentJoinCommitted(root);
  assert.equal(current, head);
  const oldRows = sourceRows(root, data.sdkSource, privateFiles);
  const oldBodies = joinRawBodies(root, data.sdkSource, oldRows);
  const model = review(oldRows, oldBodies);
  assert.equal(joinSHA(oldBodies.get(driver)), data.originalDriverSHA256);
  assert.equal(oldBodies.get(driver).toString().split("\n").length - 1, 233);
  const expected = model.rows
    .map(({ file }) => {
      const change = data.changes.find((value) => value.file === file);
      const old = oldRows.find(({ path }) => path === file);
      if (!change) return old;
      assert.deepEqual(cell(file, oldBodies.get(file)), {
        path: file,
        mode: change.before.mode,
        gitBlob: change.before.blob,
        bytes: change.before.bytes,
        sha256: change.before.sha256,
      });
      return {
        path: file,
        mode: change.after.mode,
        gitBlob: change.after.blob,
        bytes: change.after.bytes,
        sha256: change.after.sha256,
      };
    })
    .toSorted(order);
  const incoming = data.changes
    .map(({ file, after }) => ({
      path: file,
      mode: after.mode,
      gitBlob: after.blob,
      bytes: after.bytes,
      sha256: after.sha256,
    }))
    .toSorted(order);
  joinRawBodies(root, data.incomingSource, incoming);
  const raw = joinRawBodies(root, head, expected);
  for (const row of expected) assert.deepEqual(joinPhysical(root, row), raw.get(row.path));
  const extra = oldRows.filter((row) => !expected.some(({ path }) => path === row.path));
  const rawExtra = joinRawBodies(root, head, extra);
  for (const row of extra) assert.deepEqual(joinPhysical(root, row), rawExtra.get(row.path));
  const graphSupport = [
    "owned-corsa-source-custody.mjs",
    "owned-corsa-clippy-source.mjs",
    "owned-corsa-clippy-source.json",
    "owned-corsa-source-manifest.json",
    ...Array.from(
      { length: 20 },
      (_, i) => `owned-corsa-source-part-${String(i).padStart(2, "0")}.json`,
    ),
  ].map((file) => "tools/support/compat/github/" + file);
  const supportRows = sourceRows(root, data.sdkSource, graphSupport);
  joinRawBodies(root, head, supportRows);
  for (const row of supportRows) joinPhysical(root, row);
  const graphBodies = new Map(
    ownedCorsaSource.rows.map(({ file }) => {
      const body = regular(root, file);
      return [file, { mode: "100644", blob: joinObjectID("blob", body), body }];
    }),
  );
  const graph = assertOwnedCorsaAfterContext(ownedCorsaSource.rows, (file) =>
    graphBodies.get(file),
  );
  assert.equal(graph.context.wholeCurrentBodies, 237);
  assert.equal(graph.context.wholeCorrectedRustBodies, 2);
  const graphRows = [...graphBodies].map(([file, value]) => cell(file, value.body)).toSorted(order);
  joinRawBodies(root, head, graphRows);
  const aliasBody = regular(root, alias);
  assert.deepEqual(
    aliasBody,
    oldBodies.get("tests/tooling/support/ascii-symbol-original-callers.ts"),
  );
  joinRawBodies(root, head, [cell(alias, aliasBody)]);
  const ascii = qualifyCurrentSdkAsciiCallers(root);
  const receipt = Object.freeze({
    authority: "actual-sdk-and-no-mutating-current-source-join",
    head,
    currentSdkBodies: 112,
    currentSdkSuccessors: 10,
    currentNoMutatingBodies: 323,
    currentGraphBodies: 237,
    actualSdkAsciiBodies: ascii.actualCurrentBodies,
    originalDriverRows: 233,
    retainedHistoricalDriverSource: data.sdkSource,
    sourceOnly: true,
    runtimeCredit: false,
    nativeCredit: false,
    performanceCredit: false,
    currentOld111Receipt: null,
  });
  assert.equal(joined.completeCurrentClosedBodies, 323);
  assert.equal(joinGit(root, ["rev-parse", "HEAD"]).toString().trim(), head);
  receipts.set(receipt, { root: fs.realpathSync(root), oldRows, oldBodies, model });
  return receipt;
}
export function admittedSdkHistoricalView(root, receipt) {
  const value = receipts.get(receipt);
  assert(value && value.root === fs.realpathSync(root), "same verified current source receipt");
  const current = authenticateSdkComponentJoin(root);
  assert.equal(current.head, receipt.head);
  return {
    sdkSource: data.sdkSource,
    driver,
    oldRows: structuredClone(value.oldRows),
    oldBodies: new Map([...value.oldBodies].map(([file, body]) => [file, Buffer.from(body)])),
    model: structuredClone(value.model),
  };
}

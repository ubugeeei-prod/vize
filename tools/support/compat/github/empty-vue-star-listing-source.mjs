import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { after } from "node:test";
import { pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";
import { parse } from "yaml";
import {
  listStarCounterpartFiles,
  provisionStarSourceCounterpart,
  primeStarCounterpartHistory,
} from "./source-counterpart-snapshot.mjs";

const base = "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38";
const prior = "19b451f951137814a8a8d36b57d200a23100344e";
// The main commit that merged this ledger (#8809). Its rows describe that tree,
// which never changes. Later edits to the files it names are ordinary work, so
// they are read from this commit instead of being pinned at HEAD.
const recorded = "53db87849ea0d4d5d60d95f87acf4088566337b2";
const execution = "crates/vize_canon/tests/empty_vue_star_exports/execution.rs";
const lexicalCounterparts = new Set([
  "tests/performance/support/warm-type-backed-host-qualifications.ts",
  "tests/tooling/warm-type-backed-lexical-dollar-host.test.ts",
]);
const metadataPath = "tools/support/compat/github/empty-vue-star-listing-source.json";
const sha = (body) => createHash("sha256").update(body).digest("hex");
function assertBytes(actual, expected, message) {
  assert(Buffer.isBuffer(actual) && Buffer.isBuffer(expected) && actual.equals(expected), message);
}
const blob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const git = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", "--literal-pathspecs", ...args], {
    cwd: root,
    input,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
  });
function regular(root, file) {
  let target = root;
  for (const segment of file.split("/")) {
    target = path.join(target, segment);
    assert(!fs.lstatSync(target).isSymbolicLink(), "no source symlink: " + file);
  }
  const stat = fs.lstatSync(target);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644, "complete regular source: " + file);
  return fs.readFileSync(target);
}
const moduleRoot = fs.realpathSync(new URL("../../../../", import.meta.url));
const metadataBody = regular(moduleRoot, metadataPath);
assert.equal(sha(metadataBody), "be19ac68eb39c8964a0faf91bd04a5ce1a8bb4159901025fcc430e631d1bc924");
const metadata = JSON.parse(metadataBody);
assert.equal(metadata.schema, "vize.empty-vue-star.literal-bd-listing-source");
assert.equal(metadata.base, base);
assert.equal(metadata.prior, prior);
assert.equal(metadata.sourceOnly, true);
const parts = metadata.parts.map(([file, checksum]) => {
  const body = regular(moduleRoot, file);
  assert.equal(sha(body), checksum, "whole named listing table");
  return { file, body, rows: JSON.parse(body) };
});
const rows = parts.flatMap((part) => part.rows);
assert.equal(rows.length, metadata.cells);
assert.equal(new Set(rows.map(([file]) => file)).size, rows.length);
const named = new Map(rows.map((row) => [row[0], row]));
const loaded = new Map([
  [metadataPath, metadataBody],
  ...parts.map(({ file, body }) => [file, body]),
  ...metadata.executing.map((file) => [file, regular(moduleRoot, file)]),
]);
const mainInverseBody = regular(moduleRoot, metadata.mainUnion.inverseCapsule.file);
assert.equal(sha(mainInverseBody), metadata.mainUnion.inverseCapsule.compressedSHA256);
const mainInverseRaw = gunzipSync(mainInverseBody);
assert.equal(sha(mainInverseRaw), metadata.mainUnion.inverseCapsule.rawSHA256);
const mainInverses = JSON.parse(mainInverseRaw);
assert.equal(mainInverses.length, metadata.mainUnion.inverseCapsule.count);
const proofs = new WeakMap();
function raw(root, source, files) {
  const tree = git(root, ["ls-tree", "-z", source, "--", ...files]);
  assertBytes(Buffer.from(tree.toString()), tree);
  const entries = tree
    .toString()
    .split("\0")
    .filter(Boolean)
    .map((record) => {
      const match = /^100644 blob ([a-f0-9]{40})\t(.+)$/u.exec(record);
      assert(match && files.includes(match[2]), "complete regular committed listing source");
      return { blob: match[1], file: match[2] };
    });
  assert.equal(new Set(entries.map(({ file }) => file)).size, entries.length);
  const stream = git(
    root,
    ["cat-file", "--batch"],
    entries.map(({ blob: id }) => id).join("\n") + "\n",
  );
  const result = new Map();
  let offset = 0;
  for (const entry of entries) {
    const end = stream.indexOf(10, offset);
    const header = /^([a-f0-9]{40}) blob ([0-9]+)$/u.exec(stream.subarray(offset, end).toString());
    assert(header && header[1] === entry.blob);
    const body = Buffer.from(stream.subarray(end + 1, end + 1 + Number(header[2])));
    assert.equal(body.length, Number(header[2]));
    assert.equal(blob(body), entry.blob);
    offset = end + 1 + body.length;
    assert.equal(stream[offset++], 10);
    result.set(entry.file, body);
  }
  assert.equal(offset, stream.length);
  return result;
}
function confirm(owned) {
  assert.equal(git(owned.root, ["rev-parse", "HEAD"]).toString().trim(), owned.head);
  assert.equal(git(owned.root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  for (const [file, body] of owned.live)
    assertBytes(regular(owned.root, file), body, "same complete current source: " + file);
}

const setupWorkflows = new Set([".github/workflows/typechecker-native-phases.yml"]);
/** Recorded by its historical bytes only; the current workflow may change freely. */
const unpinnedWorkflows = new Set([".github/workflows/davinci-canon-scaling.yml"]);
/** Historical source stays literal; current CI may use the approved setup changes. */
export function assertStarListingWorkflowSetup(file, before, after) {
  assert(setupWorkflows.has(file), "only the reviewed current workflow contract");
  const documents = [before, after].map((body) => parse(body.toString()));
  for (const document of documents) {
    if (file === ".github/workflows/typechecker-native-phases.yml") {
      const paths = document.on.pull_request.paths;
      const helper = "tools/benchmarks/scripts/type-snapshot-parser-recovery.ts";
      const index = paths.indexOf(helper);
      if (index !== -1) {
        assert.equal(index, paths.lastIndexOf(helper));
        assert.equal(paths[index - 1], "tools/benchmarks/scripts/typecheck-command.mjs");
        paths.splice(index, 1);
      }
    }
    for (const job of Object.values(document.jobs))
      for (const step of job.steps ?? []) {
        if (step.uses === "wild-linker/action@0bbbfa5df4380cab8e63cb8505a1ce65e1d10203") {
          assert.deepEqual(step.with, { "wild-version": "0.9.0" });
          step.uses = "./.github/actions/setup-mold";
          delete step.with;
        }
        if (
          step.uses === "dtolnay/rust-toolchain@7e38f4b43b4db5c8dd498af069a4f6196df1d067" &&
          step.with?.toolchain === "1.99.0"
        )
          step.with.toolchain = "stable";
      }
  }
  assert.deepEqual(documents[1], documents[0], "unchanged current workflow commands and topology");
}

/** Authenticate the whole actual BD cohort and named entrypoints before any source counterpart. */
export function authenticateStarListingSource(root) {
  root = fs.realpathSync(root);
  const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  assert.match(head, /^[a-f0-9]{40}$/u);
  assert.equal(git(root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  primeStarCounterpartHistory(root);
  for (const source of [base, prior, recorded]) {
    const body = git(root, ["cat-file", "commit", source]);
    assert.equal(
      createHash("sha1").update(`commit ${body.length}\0`).update(body).digest("hex"),
      source,
      "whole literal source identity",
    );
  }
  const files = [
    ...new Set([
      ...named.keys(),
      ...loaded.keys(),
      ...metadata.mainUnion.cells.map(([file]) => file),
    ]),
  ];
  assert.equal(new Set(files).size, files.length);
  const before = raw(root, base, files),
    current = raw(root, recorded, files),
    live = raw(root, head, files);
  const projectedCurrent = new Map(current);
  const canonical = metadata.mainUnion.canonicalSuccessor;
  if (canonical) {
    const [file, expectedBlob, length, checksum] = canonical.after;
    assert.equal(file, "docs/davinci/decisions/2026-09-27-level-restructure.md");
    let body = current.get(file);
    assert.equal(blob(body), expectedBlob);
    assert.equal(body.length, length);
    assert.equal(sha(body), checksum);
    for (const [start, end, from, to] of [...canonical.clauses].reverse()) {
      assertBytes(body.subarray(start, end), Buffer.from(from, "base64"));
      body = Buffer.concat([
        body.subarray(0, start),
        Buffer.from(to, "base64"),
        body.subarray(end),
      ]);
    }
    projectedCurrent.set(file, body);
  }
  const incoming = raw(
    root,
    metadata.mainUnion.source,
    metadata.mainUnion.cells.map(([file]) => file),
  );
  // The union records one incoming main change. Shared files outside the named
  // BD cohort keep evolving on main; once one moves past its recorded successor
  // it is no longer the live source, so only its incoming body stays authenticated.
  const superseded = new Set();
  for (const [file, oldBlob, oldLength, oldSHA, nextBlob, nextLength, nextSHA] of metadata.mainUnion
    .cells) {
    const old = incoming.get(file),
      next = projectedCurrent.get(file);
    assert.equal(blob(old), oldBlob);
    assert.equal(old.length, oldLength);
    assert.equal(sha(old), oldSHA);
    if (!named.has(file) && !loaded.has(file) && blob(next) !== nextBlob) {
      superseded.add(file);
      continue;
    }
    assert.equal(blob(next), nextBlob);
    assert.equal(next.length, nextLength);
    assert.equal(sha(next), nextSHA);
  }
  for (const pair of mainInverses) {
    if (superseded.has(pair.file)) continue;
    let body = Buffer.from(projectedCurrent.get(pair.file));
    for (const [start, end, from, to] of [...pair.clauses].reverse()) {
      assertBytes(body.subarray(start, end), Buffer.from(from, "base64"));
      body = Buffer.concat([
        body.subarray(0, start),
        Buffer.from(to, "base64"),
        body.subarray(end),
      ]);
    }
    assertBytes(body, incoming.get(pair.file), "complete actual incoming main counterpart");
  }
  const sdkCells = metadata.sdkUnion.cells;
  assert.deepEqual(
    sdkCells.map(({ file }) => file),
    [
      "tests/_fixtures/vue-ecosystem-fixtures.json",
      "tests/tooling/empty-vue-star-javascript-source.test.mjs",
      "tests/tooling/warm-type-backed-empty-vue-star-custody.test.ts",
    ],
    "only the three actual SDK overlaps",
  );
  const sdkBefore = raw(
    root,
    metadata.sdkUnion.source,
    sdkCells.map(({ file }) => file),
  );
  const sdkIncoming = raw(
    root,
    metadata.sdkUnion.actualIncoming,
    sdkCells.map(({ file }) => file),
  );
  for (const cell of sdkCells) {
    const old = sdkBefore.get(cell.file),
      body = current.get(cell.file);
    assertBytes(sdkIncoming.get(cell.file), old, "actual merged SDK whole preimage");
    for (const [actual, expected] of [
      [old, cell.before],
      [body, cell.after],
    ]) {
      assert.equal(expected.mode, "100644");
      assert.equal(blob(actual), expected.blob);
      assert.equal(actual.length, expected.length);
      assert.equal(sha(actual), expected.sha256);
    }
    let restored = Buffer.from(body);
    for (const [start, end, from, to] of [...cell.clauses].reverse()) {
      assertBytes(restored.subarray(start, end), Buffer.from(from, "base64"));
      restored = Buffer.concat([
        restored.subarray(0, start),
        Buffer.from(to, "base64"),
        restored.subarray(end),
      ]);
    }
    assertBytes(restored, old, "complete original SDK overlap inverse");
  }
  for (const [
    file,
    beforeBlob,
    beforeLength,
    beforeSHA,
    afterBlob,
    afterLength,
    afterSHA,
  ] of rows) {
    const old = before.get(file);
    let body = current.get(file);
    assert(old && body, "complete named source pair: " + file);
    assert.equal(blob(old), beforeBlob);
    assert.equal(old.length, beforeLength);
    assert.equal(sha(old), beforeSHA);
    if (setupWorkflows.has(file) || unpinnedWorkflows.has(file)) {
      assert.equal(beforeBlob, afterBlob, "the recorded workflow did not change in the listing");
      if (setupWorkflows.has(file)) assertStarListingWorkflowSetup(file, old, body);
      body = old;
    }
    assert.equal(blob(body), afterBlob);
    assert.equal(body.length, afterLength);
    assert.equal(sha(body), afterSHA);
  }
  for (const [file, expected] of loaded) {
    assert.equal(before.get(file), undefined, "genuine named source predecessor absence: " + file);
    assertBytes(live.get(file), expected, "complete committed executing source: " + file);
  }
  for (const pair of metadata.inverses) {
    let body = Buffer.from(current.get(pair.file));
    for (const [start, end, from, to] of [...pair.clauses].reverse()) {
      const next = Buffer.from(from, "base64"),
        old = Buffer.from(to, "base64");
      assertBytes(body.subarray(start, end), next, "one exact named counterpart clause");
      body = Buffer.concat([body.subarray(0, start), old, body.subarray(end)]);
    }
    assertBytes(body, before.get(pair.file), "complete literal BD counterpart: " + pair.file);
  }
  assert.deepEqual(
    metadata.inverses.map(({ file }) => file),
    metadata.changed,
  );
  const owned = { root, head, before, current, live };
  confirm(owned);
  const packet = Object.freeze({
    authority: "complete-literal-bd-star-listing-source",
    head,
    base,
    currentBodies: files.length,
    originalStarBodies: 322,
    originalJSBodies: 93,
    runtimeCredit: false,
    original400Credit: false,
    performanceCredit: false,
  });
  proofs.set(packet, owned);
  return packet;
}
function owned(packet) {
  const value = proofs.get(packet);
  assert(value, "only a genuine named listing source packet");
  confirm(value);
  return value;
}
/** Only this exact singleton, with the complete current source cohort on both sides. */
export function qualifyStarListingBD(production, qualified, digest) {
  if (production.length !== 1 || production[0] !== execution || qualified.has(execution))
    return null;
  for (const [file, , , beforeSHA, , , afterSHA] of rows)
    if (digest("before", file) !== beforeSHA || digest("after", file) !== afterSHA) return null;
  for (const [file, body] of loaded)
    if (digest("before", file) !== null || digest("after", file) !== sha(body)) return null;
  return {
    authority: "exact-reviewed-bd-star-listing-singleton",
    files: [execution],
    completeBodyPairs: 1,
    retainedCurrentBodyPairs: rows.length - 1,
    runtimeCredit: false,
    performanceCredit: false,
    original400Credit: false,
  };
}
/** The actual merged SDK before-image is a separate exact singleton authority. */
export function qualifyStarListingSDK(production, qualified, digest) {
  if (production.length !== 1 || production[0] !== execution || qualified.has(execution))
    return null;
  const before = new Map(metadata.sdkShippingBefore.map(([file, checksum]) => [file, checksum]));
  for (const [file, , , originalBefore, , , afterSHA] of rows)
    if (
      digest("before", file) !== (before.has(file) ? before.get(file) : originalBefore) ||
      digest("after", file) !== afterSHA
    )
      return null;
  for (const [file, body] of loaded)
    if (
      digest("before", file) !== (before.get(file) ?? null) ||
      digest("after", file) !== sha(body)
    )
      return null;
  return {
    authority: "exact-reviewed-sdk-star-listing-singleton",
    files: [execution],
    completeBodyPairs: 1,
    retainedCurrentBodyPairs: rows.length - 1,
    runtimeCredit: false,
    performanceCredit: false,
    original400Credit: false,
  };
}
/** The new incoming main before-image stays separate from the original SDK law. */
export function qualifyStarListingMain(production, qualified, digest) {
  if (production.length !== 1 || production[0] !== execution || qualified.has(execution))
    return null;
  const before = new Map(metadata.mainShippingBefore);
  for (const [file, , , oldSHA, , , nextSHA] of rows)
    if (
      digest("before", file) !== (before.get(file) ?? oldSHA) ||
      digest("after", file) !== nextSHA
    )
      return null;
  // An executing file that is also a recorded main cell is governed by that
  // cell below, not by whatever the file contains today.
  const cells = new Set(metadata.mainUnion.cells.map(([file]) => file));
  for (const [file, body] of loaded)
    if (
      !cells.has(file) &&
      (digest("before", file) !== (before.get(file) ?? null) || digest("after", file) !== sha(body))
    )
      return null;
  for (const [file, , , oldSHA, , , nextSHA] of metadata.mainUnion.cells)
    if (
      digest("before", file) !== oldSHA ||
      digest("after", file) !==
        (metadata.mainUnion.canonicalSuccessor?.after[0] === file
          ? metadata.mainUnion.canonicalSuccessor.after[3]
          : nextSHA)
    )
      return null;
  return {
    authority: "exact-reviewed-incoming-main-star-listing-singleton",
    files: [execution],
    sourceOnly: true,
    runtimeCredit: false,
    performanceCredit: false,
    original400Credit: false,
  };
}
const oldTests = new Map(metadata.tests.map(([file, source]) => [file, source]));
/** The full named current law precedes each unchanged literal historical test body. */
export async function runStarListingOriginalTest(root, file) {
  assert.equal(process.env.STAR_JS_SOURCE_ROOT, undefined, "no ambient historical source owner");
  assert.equal(process.env.JS_STAR_SOURCE_INDEX, undefined, "no private current-source override");
  const packet = authenticateStarListingSource(root),
    value = owned(packet);
  const source = oldTests.get(file);
  assert(source, "one named original listing/JS source law");
  const initial =
    source === base
      ? [...named.keys()]
      : source === metadata.mainUnion.source
        ? metadata.mainUnion.frameInputs
        : [file];
  const files = listStarCounterpartFiles(value.root, source, initial);
  const fixture = provisionStarSourceCounterpart(value.root, source, files);
  let retained = false;
  try {
    await import(pathToFileURL(path.join(fixture.root, file)).href);
    after(() => {
      try {
        owned(packet);
      } finally {
        fixture.cleanup();
      }
    });
    retained = true;
    return packet;
  } finally {
    if (!retained) fixture.cleanup();
  }
}
/** The original lexical tables retain their recorded bodies after current source verification. */
export function starListingLexicalReader(copiedRoot) {
  const packet = authenticateStarListingSource(moduleRoot),
    value = owned(packet);
  const read = (file) => {
    assert(metadata.javascriptPaths.includes(file), "one closed current lexical JS93 source");
    const current = regular(copiedRoot, file);
    assertBytes(current, value.live.get(file), "complete actual copied lexical source");
    if (lexicalCounterparts.has(file) || setupWorkflows.has(file) || unpinnedWorkflows.has(file)) {
      owned(packet);
      return Buffer.from(value.before.get(file));
    }
    return Buffer.from(current);
  };
  return Object.assign(read, { confirmSource: () => owned(packet) });
}

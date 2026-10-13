import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  emptyVueStarBodyPairs,
  emptyVueStarMetadataPairs,
  emptyVueStarFiles,
  qualifyEmptyVueStarHost,
} from "../../../../tests/performance/support/warm-type-backed-empty-vue-star-host.ts";
import { ensureHostedLiteralSource } from "../../../../tests/performance/support/warm-type-backed-git-reader-literal-source.ts";

import { javascriptSfcStarAliasCorrection } from "../../../../tests/performance/support/warm-type-backed-javascript-sfc-host.ts";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const object = (kind, body) =>
  createHash("sha1")
    .update(kind + " " + body.length + "\0")
    .update(body)
    .digest("hex");
const ownData = "72b18a29a597a17a4977508e47785be9d090b44ed4466c4506d024bc90425762";
function regular(file) {
  const stat = fs.lstatSync(file);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644, "regular source: " + file);
  return fs.readFileSync(file);
}
function metadata(name, expected) {
  const body = regular(new URL(name, import.meta.url));
  assert.equal(sha(body), expected, "whole reviewed metadata: " + name);
  return { body, data: JSON.parse(body) };
}
const executing = new Map();
const own = metadata("empty-vue-star-javascript-source.json", ownData);
executing.set("tools/support/compat/github/empty-vue-star-javascript-source.json", own.body);
const data = own.data;
assert.equal(data.schema, "vize.empty-vue-star.javascript-current-source");
assert.equal(data.version, 1);
assert.equal(data.sourceOnly, true);
assert.equal(data.compiledBodies, 281);
assert.equal(data.starMetadataBodies, 9);
assert.equal(data.originalAuxiliaryBodies, 9);
assert.equal(data.javascriptAuxiliaryBodies, 17);
const auxiliary = data.parts.flatMap((part, index) => {
  assert.equal(part.file, "empty-vue-star-javascript-aux-" + index + ".json");
  const { body, data: rows } = metadata(part.file, part.sha256);
  executing.set("tools/support/compat/github/" + part.file, body);
  assert.equal(part.mode, "100644");
  assert.equal(body.length, part.byteLength);
  assert.equal(object("blob", body), part.blob);
  assert.equal(rows.length, part.entries);
  return rows;
});
assert.equal(auxiliary.length, 26);
assert.equal(new Set(auxiliary.map(({ path }) => path)).size, 26);
const shared = metadata("javascript-sfc-star-source.json", data.sharedJavaScriptCapsulesSHA256);
assert.equal(shared.data.beforeSource, "3a680d0a0b59b720af0d23a70a85cf56c5c4f175");
const beforeCommit = Buffer.from(shared.data.beforeCommit.rawBase64, "base64");
assert.equal(beforeCommit.toString("base64"), shared.data.beforeCommit.rawBase64);
assert.equal(beforeCommit.length, shared.data.beforeCommit.byteLength);
assert.equal(sha(beforeCommit), shared.data.beforeCommit.sha256);
assert.equal(object("commit", beforeCommit), shared.data.beforeSource);
assert.equal(data.beforeSource, shared.data.beforeSource);
assert.equal(data.beforeTree, "577d549f8a9c6b0a51f6bf99bc23fb35eec5942a");
assert(beforeCommit.toString().startsWith("tree " + data.beforeTree + "\n"));
const context = shared.data.parts.map(([file, digest], index) => {
  assert.equal(file, "javascript-sfc-star-source-" + String(index).padStart(2, "0") + ".json");
  return metadata(file, digest).data;
});
assert.deepEqual(
  context.map(({ path }) => path),
  [
    ".github/workflows/typechecker-native-phases.yml",
    "crates/vize_canon/src/batch/virtual_project/build.rs",
    "crates/vize_canon/src/batch/virtual_project/module_projection.rs",
    "crates/vize_canon/src/batch/virtual_project/project/artifacts.rs",
    "docs/davinci/plan/consumer-migration-surfaces/typechecker/vize_canon.tsv",
    "tests/performance/support/warm-type-backed-host-qualifications.ts",
  ],
);
function decode(cell) {
  const body = Buffer.from(cell.body, "base64");
  assert.equal(body.toString("base64"), cell.body);
  assert.equal(body.length, cell.byteLength);
  assert.equal(sha(body), cell.sha256);
  assert.equal(object("blob", body), cell.blob);
  return { mode: "100644", blob: cell.blob, body };
}
const corrections = new Map(
  context.map((row) => [
    row.path,
    {
      before: decode(row.currentBefore),
      after: decode(row.currentAfter),
    },
  ]),
);
assert(!corrections.has(javascriptSfcStarAliasCorrection.path));
corrections.set(javascriptSfcStarAliasCorrection.path, {
  before: decode(javascriptSfcStarAliasCorrection.currentBefore),
  after: decode(javascriptSfcStarAliasCorrection.currentAfter),
});
executing.set(
  "tools/support/compat/github/empty-vue-star-javascript-source.mjs",
  regular(new URL(import.meta.url)),
);
executing.set(
  "tests/tooling/empty-vue-star-javascript-source.test.mjs",
  regular(
    new URL("../../../../tests/tooling/empty-vue-star-javascript-source.test.mjs", import.meta.url),
  ),
);
executing.set(
  "tools/support/compat/github/empty-vue-star-custody-runner.mjs",
  regular(new URL("empty-vue-star-custody-runner.mjs", import.meta.url)),
);
assert.equal(executing.size, 6);
const pairs = [...emptyVueStarBodyPairs, ...emptyVueStarMetadataPairs];
assert.equal(pairs.length, 290);
assert.equal(new Set(pairs.map(([file]) => file)).size, 290);
const files = [
  ...pairs.map(([file]) => file),
  ...auxiliary.map(({ path }) => path),
  ...executing.keys(),
];
assert.equal(files.length, 322);
assert.equal(new Set(files).size, files.length);
const proofs = new WeakMap();
const git = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    input,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
  });
function raw(root, source) {
  const entries = git(root, ["ls-tree", "-z", source, "--", ...files]);
  assert.deepEqual(Buffer.from(entries.toString()), entries);
  const rows = entries
    .toString()
    .split("\0")
    .filter(Boolean)
    .map((entry) => {
      const match = /^100644 blob ([a-f0-9]{40})\t(.+)$/u.exec(entry);
      assert(match && files.includes(match[2]), "one regular raw source");
      return { file: match[2], blob: match[1] };
    });
  assert.equal(new Set(rows.map(({ file }) => file)).size, rows.length);
  const stream = git(root, ["cat-file", "--batch"], rows.map(({ blob }) => blob).join("\n") + "\n");
  const result = new Map(files.map((file) => [file, null]));
  let offset = 0;
  for (const row of rows) {
    const end = stream.indexOf("\n", offset);
    const match = /^([a-f0-9]{40}) blob ([0-9]+)$/u.exec(stream.subarray(offset, end).toString());
    assert(match && match[1] === row.blob);
    const body = Buffer.from(stream.subarray(end + 1, end + 1 + Number(match[2])));
    assert.equal(body.length, Number(match[2]));
    assert.equal(object("blob", body), row.blob);
    offset = end + 1 + body.length;
    assert.equal(stream[offset++], 10);
    result.set(row.file, { mode: "100644", blob: row.blob, body });
  }
  assert.equal(offset, stream.length);
  return result;
}
function exact(value, expected, file) {
  if (expected === null) assert.equal(value, null, "genuine source absence: " + file);
  else {
    assert(value && value.mode === expected.mode, "regular whole source: " + file);
    assert.equal(value.blob, expected.blob, "source blob: " + file);
    assert.equal(value.body.length, expected.byteLength, "source length: " + file);
    assert.equal(sha(value.body), expected.sha256, "source digest: " + file);
  }
}
function physical(root, file) {
  let directory = root;
  for (const segment of file.split("/").slice(0, -1)) {
    directory = path.join(directory, segment);
    assert(fs.lstatSync(directory).isDirectory(), "no ancestor symlink: " + file);
  }
  return regular(path.join(root, file));
}
function currentHead(root) {
  return git(root, ["rev-parse", "HEAD"]).toString().trim();
}
/** Actual source first; the immutable original Star model grants no runtime credit. */
export function authenticateEmptyVueStarJoinedSource(root, head = currentHead(root)) {
  root = fs.realpathSync(root);
  assert.match(head, /^[a-f0-9]{40}$/u);
  assert.equal(currentHead(root), head, "same actual current source owner");
  assert.equal(git(root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  ensureHostedLiteralSource(root, data.beforeSource);
  assert.deepEqual(git(root, ["cat-file", "commit", data.beforeSource]), beforeCommit);
  const before = raw(root, data.beforeSource),
    after = raw(root, head);
  for (const [file, , expected] of pairs) {
    const previous = before.get(file),
      actual = after.get(file);
    assert(previous && previous.mode === "100644", "complete actual main Star source: " + file);
    assert.equal(sha(previous.body), expected, "actual main whole original Star source: " + file);
    if (corrections.has(file)) {
      assert.deepEqual(previous, corrections.get(file).before);
      assert.deepEqual(actual, corrections.get(file).after, "exact fused Star source: " + file);
    } else assert.deepEqual(actual, previous, "all other original Star bodies stay exact: " + file);
  }
  for (const { path: file, before: previous, after: expected } of auxiliary) {
    exact(before.get(file), previous, file);
    exact(after.get(file), expected, file);
  }
  for (const [file, body] of executing) {
    assert.equal(before.get(file), null, "genuine new adapter before absence: " + file);
    assert(after.get(file) && after.get(file).mode === "100644");
    assert.deepEqual(
      after.get(file).body,
      body,
      "selected raw matches loaded source snapshot: " + file,
    );
  }
  for (const file of files) {
    const actual = after.get(file);
    assert(actual, "complete current source: " + file);
    assert.deepEqual(physical(root, file), actual.body, "physical equals selected raw: " + file);
  }
  // Only after full322 current bodies, authenticate every original A93 cell/absence.
  ensureHostedLiteralSource(root, data.originalSource);
  const original = raw(root, data.originalSource);
  for (const [file, expected] of pairs)
    assert.equal(
      original.get(file) ? sha(original.get(file).body) : null,
      expected,
      "whole original predecessor: " + file,
    );
  const retained = qualifyEmptyVueStarHost([...emptyVueStarFiles], new Set(), (side, file) => {
    assert(
      pairs.some(([known]) => known === file),
      "unknown retained qualifier source",
    );
    const cell = (side === "before" ? original : before).get(file);
    return cell ? sha(cell.body) : null;
  });
  assert(retained, "the complete unchanged original Star qualifier must pass");
  assert.equal(currentHead(root), head, "same source owner after complete qualification");
  const packet = Object.freeze({
    authority: "complete-current-javascript-and-retained-star-source",
    head,
    beforeSource: data.beforeSource,
    compiledBodies: 281,
    starMetadataBodies: 9,
    nativeAndOriginalCallerBodies: 9,
    javascriptAuxiliaryBodies: 17,
    executingAdapterBodies: 6,
    completeCurrentBodies: files.length,
    changedStarProductionBodies: 4,
    retainedStarQualifier: Object.freeze({ ...retained, files: Object.freeze(retained.files) }),
    sourceOnly: true,
    runtimeCredit: false,
    performanceCredit: false,
    original400Credit: false,
  });
  proofs.set(packet, { root, head, before, after });
  return packet;
}
/** Cloned known main3a bodies only; forged, stale and unknown requests refuse. */
export function retainedEmptyVueStarBody(packet, file) {
  const owned = proofs.get(packet);
  assert(owned, "only a genuinely admitted current Star packet can loan a source");
  assert.equal(currentHead(owned.root), owned.head, "same source owner before historical loan");
  assert.equal(git(owned.root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  for (const [currentFile, actual] of owned.after)
    assert.deepEqual(
      physical(owned.root, currentFile),
      actual.body,
      "same current source before loan: " + currentFile,
    );
  assert(files.includes(file), "unknown retained Star source refuses");
  const cell = owned.before.get(file);
  assert(cell, "a genuine main absence cannot become a historical Star source");
  return Buffer.from(cell.body);
}

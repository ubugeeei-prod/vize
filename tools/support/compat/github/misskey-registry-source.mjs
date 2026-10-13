import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { after } from "node:test";
import { pathToFileURL } from "node:url";
import {
  listStarCounterpartFiles,
  provisionStarSourceCounterpart,
} from "./source-counterpart-snapshot.mjs";
import { ensureHostedLiteralSource } from "../../../../tests/performance/support/warm-type-backed-git-reader-literal-source.ts";

const base = "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38";
const metadataPath = "tools/support/compat/github/misskey-registry-source.json";
const sha = (body) => createHash("sha256").update(body).digest("hex");
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
assert.equal(sha(metadataBody), "0aea60acd69112d59c06ab347f0ccce804c3fc202d999444fb7cb6569f631a2d");
const metadata = JSON.parse(metadataBody);
assert.equal(metadata.schema, "vize.empty-vue-star.literal-bd-misskey-registry-source");
assert.equal(metadata.base, base);
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
const proofs = new WeakMap();
function raw(root, source, files) {
  const tree = git(root, ["ls-tree", "-z", source, "--", ...files]);
  assert.deepEqual(Buffer.from(tree.toString()), tree);
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
  for (const [file, body] of owned.current)
    assert.deepEqual(regular(owned.root, file), body, "same complete current source: " + file);
}

/** Authenticate the whole actual BD cohort and named entrypoints before any source counterpart. */
export function authenticateMisskeyRegistrySource(root) {
  assert.equal(process.env.STAR_JS_SOURCE_ROOT, undefined, "no ambient historical source owner");
  assert.equal(process.env.JS_STAR_SOURCE_INDEX, undefined, "no private source override");
  root = fs.realpathSync(root);
  assert.equal(
    fs.realpathSync(git(root, ["rev-parse", "--show-toplevel"]).toString().trim()),
    root,
  );
  for (const key of [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_INDEX_FILE",
    "GIT_NAMESPACE",
    "GIT_SHALLOW_FILE",
  ])
    assert.equal(process.env[key], undefined, "no ambient source override: " + key);
  const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  assert.match(head, /^[a-f0-9]{40}$/u);
  assert.equal(git(root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  ensureHostedLiteralSource(root, base);
  for (const source of [base]) {
    const body = git(root, ["cat-file", "commit", source]);
    assert.equal(
      createHash("sha1").update(`commit ${body.length}\0`).update(body).digest("hex"),
      source,
      "whole literal source identity",
    );
  }
  const files = [...named.keys(), ...loaded.keys()];
  assert.equal(new Set(files).size, files.length);
  const before = raw(root, base, files),
    current = raw(root, head, files);
  for (const [
    file,
    beforeBlob,
    beforeLength,
    beforeSHA,
    afterBlob,
    afterLength,
    afterSHA,
  ] of rows) {
    const old = before.get(file),
      body = current.get(file);
    assert(old && body, "complete named source pair: " + file);
    assert.equal(blob(old), beforeBlob);
    assert.equal(old.length, beforeLength);
    assert.equal(sha(old), beforeSHA);
    assert.equal(blob(body), afterBlob);
    assert.equal(body.length, afterLength);
    assert.equal(sha(body), afterSHA);
  }
  for (const [file, expected] of loaded) {
    assert.equal(before.get(file), undefined, "genuine named source predecessor absence: " + file);
    assert.deepEqual(current.get(file), expected, "complete committed executing source: " + file);
  }
  for (const pair of metadata.inverses) {
    let body = Buffer.from(current.get(pair.file));
    for (const [start, end, from, to] of [...pair.clauses].reverse()) {
      const next = Buffer.from(from, "base64"),
        old = Buffer.from(to, "base64");
      assert.deepEqual(body.subarray(start, end), next, "one exact named counterpart clause");
      body = Buffer.concat([body.subarray(0, start), old, body.subarray(end)]);
    }
    assert.deepEqual(body, before.get(pair.file), "complete literal BD counterpart: " + pair.file);
  }
  assert.deepEqual(
    metadata.inverses.map(({ file }) => file),
    metadata.changed,
  );
  const registry = "tests/_fixtures/vue-ecosystem-fixtures.json";
  const previous = JSON.parse(before.get(registry));
  const successor = JSON.parse(current.get(registry));
  const misskey = (value) => value.projects.find((entry) => entry.id === "misskey");
  assert.equal(misskey(previous).typecheckPerformance.baseline, undefined);
  assert.deepEqual(misskey(successor).typecheckPerformance.baseline.prepare, [
    "pnpm",
    "--filter",
    "misskey-js",
    "exec",
    "tsgo",
    "--project",
    "tsconfig.json",
    "--outDir",
    "built",
    "--declaration",
    "true",
    "--emitDeclarationOnly",
    "true",
  ]);
  delete misskey(successor).typecheckPerformance.baseline;
  assert.deepEqual(successor, previous, "all registry neighbors, pins and budgets unchanged");
  const owned = { root, head, before, current };
  confirm(owned);
  const packet = Object.freeze({
    authority: "complete-literal-bd-misskey-registry-source",
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
const oldTests = new Map(metadata.tests.map(([file, source]) => [file, source]));
/** Only a genuine same-owner packet may provision either whole original source law. */
export function misskeyRegistryCounterpart(packet, file) {
  const value = owned(packet);
  const source = oldTests.get(file);
  assert(source, "one named original registry/Star source law");
  for (const historical of metadata.history) ensureHostedLiteralSource(value.root, historical);
  const files = listStarCounterpartFiles(value.root, source, [...named.keys()]);
  return provisionStarSourceCounterpart(value.root, source, files);
}
/** The full named current law precedes each unchanged literal historical test body. */
export async function runMisskeyRegistryOriginalTest(root, file) {
  const packet = authenticateMisskeyRegistrySource(root);
  const fixture = misskeyRegistryCounterpart(packet, file);
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

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { ensureHostedLiteralSource } from "./warm-type-backed-git-reader-literal-source.ts";
import { projectOwnedDualSourceFrame } from "./owned-corsa-dual-source.ts";

export type Cell = { mode: string; blob: string; bytes: number; sha256: string };
export type Body = Cell & { body: Buffer };
type Reader = {
  head(root: string): string;
  commit(root: string, source: string): Buffer;
  replacements(root: string): Buffer;
  frame(root: string, source: string): Map<string, Body>;
  physical(root: string, file: string): Body | null;
};
type Role = "sdk" | "component";
type Row = { file: string; final: Cell; sdk: Cell | null; component: Cell | null };
type Original = {
  source: string;
  directory: string;
  module: string;
  files: Array<Cell & { file: string }>;
  rows: Array<{ file: string; currentAfter: Cell }>;
  retained: Array<Cell & { file: string }>;
};
type Review = {
  version: number;
  scope: string;
  beforeSource: string;
  beforeCommit: Cell;
  beforeRows: Array<{ file: string; cell: Cell | null }>;
  rows: Row[];
  original: Record<Role, Original>;
  changes: Record<Role, string[]>;
  driver: string;
  privatePrototype: boolean;
};
const hash = (body: Uint8Array | string) => createHash("sha256").update(body).digest("hex");
const object = (kind: string, body: Buffer) =>
  createHash("sha1").update(`${kind} ${body.length}\0`).update(body).digest("hex");
export function bodyCell(body: Buffer, mode = "100644"): Cell {
  return { mode, blob: object("blob", body), bytes: body.length, sha256: hash(body) };
}
const reviewName = "warm-type-backed-git-reader-dual-frame.json";
const reviewFile = new URL("./" + reviewName, import.meta.url);
const reviewSHA256 = "9b42d71684332a5f6ad729ab95deb8434da0707cbe37eaa31997e0db6335d549";
const manifestSHA256 = "83bb4b913b063ff03d47d2c67112f5a6f55f46b1b84f3e964e52c5a44f33aa8f";
function reviewBytes() {
  const stat = fs.lstatSync(reviewFile);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
  const manifest = fs.readFileSync(reviewFile);
  assert.equal(hash(manifest), manifestSHA256);
  const metadata = JSON.parse(manifest.toString());
  assert.equal(metadata.version, 1);
  assert.equal(metadata.reconstructedSHA256, reviewSHA256);
  assert.deepEqual(
    metadata.parts.map(({ file }: { file: string }) => file),
    Array.from(
      { length: 24 },
      (_, i) => `warm-type-backed-git-reader-dual-frame-part-${String(i).padStart(2, "0")}.json`,
    ),
  );
  for (const part of metadata.parts) {
    const file = new URL("./" + part.file, import.meta.url);
    const stat = fs.lstatSync(file);
    assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
    const body = fs.readFileSync(file);
    assert.equal(part.mode, "100644");
    assert.equal(body.length, part.bytes);
    assert.equal(hash(body), part.sha256);
    assert.equal(object("blob", body), part.blob);
    const values = JSON.parse(body.toString());
    assert(Array.isArray(values));
    assert.equal(values.length, part.entries);
    const keys = part.section.split(".");
    const field = keys.pop();
    const parent = keys.reduce(
      (value: Record<string, unknown>, key: string) => value[key],
      metadata.skeleton,
    );
    assert(Array.isArray(parent[field]));
    assert.equal(parent[field].length, part.start);
    parent[field].push(...values);
  }
  const body = Buffer.from(JSON.stringify(metadata.skeleton, null, 2) + "\n");
  assert.equal(body.length, metadata.reconstructedBytes);
  assert.equal(hash(body), reviewSHA256);
  return body;
}
const bytes = reviewBytes();
function freezeReview(value: unknown): unknown {
  // The authenticated JSON is frozen once before any authority view is exposed.
  if (value && typeof value === "object") {
    for (const child of Object.values(value)) freezeReview(child);
    Object.freeze(value);
  }
  return value;
}
const review = freezeReview(JSON.parse(bytes.toString())) as Review;
assert.equal(review.version, 1);
assert.equal(review.scope, "finite-dual-current-source-frame");
assert.equal(review.rows.length, 112);
assert.equal(new Set(review.rows.map(({ file }) => file)).size, 112);
assert.equal(review.beforeRows.length, 112);
assert.deepEqual(
  review.beforeRows.map(({ file }) => file),
  review.rows.map(({ file }) => file),
);
assert.deepEqual(review.changes.sdk, [
  "crates/vize_patina/src/linter/engine/rule_sets.rs",
  "crates/vize_patina/src/linter/engine/sfc.rs",
  "tests/tooling/warm-type-backed-ascii-symbol-host.test.ts",
  "tests/tooling/warm-type-backed-default-migration-host.test.ts",
  "tests/tooling/warm-type-backed-git-reader-nodeep.test.ts",
]);
assert.deepEqual(review.changes.component, [
  "tests/tooling/ascii-symbol-original-callers.test.ts",
  "tests/tooling/support/ascii-symbol-original-callers.ts",
  "tests/tooling/warm-type-backed-git-reader-nodeep.test.ts",
]);
for (const role of ["sdk", "component"] as const) {
  const original = review.original[role];
  assert.equal(original.rows.length, 111);
  assert.equal(new Set(original.rows.map(({ file }) => file)).size, 111);
  assert.equal(original.retained.length, 113);
  const changes: string[] = [];
  for (const row of original.rows) {
    const next = review.rows.find(({ file }) => file === row.file);
    assert(next);
    assert.deepEqual(next[role], row.currentAfter);
    if (JSON.stringify(next.final) !== JSON.stringify(row.currentAfter)) changes.push(row.file);
  }
  assert.deepEqual(changes.sort(), review.changes[role]);
}
assert.deepEqual(review.original.sdk.retained, review.original.component.retained);

const git = (root: string, args: string[]) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
    maxBuffer: 64 * 1024 * 1024,
  });
const actualReader: Reader = Object.freeze({
  head: (root) => git(root, ["rev-parse", "HEAD"]).toString().trim(),
  commit: (root, source) => {
    ensureHostedLiteralSource(root, source);
    return git(root, ["cat-file", "commit", source]);
  },
  replacements: (root) => git(root, ["for-each-ref", "--format=%(refname)", "refs/replace"]),
  frame(root, source) {
    assert.match(source, /^[a-f0-9]{40}$/u);
    const files = review.rows.map(({ file }) => file);
    const entries = git(root, ["ls-tree", "-z", source, "--", ...files]);
    assert.deepEqual(Buffer.from(entries.toString()), entries);
    const headers = entries
      .toString()
      .split("\0")
      .filter(Boolean)
      .map((entry) => {
        const match = /^100644 blob ([a-f0-9]{40})\t(.+)$/u.exec(entry);
        assert(match && files.includes(match[2]), "one raw regular source required");
        return { blob: match[1], file: match[2] };
      });
    assert.equal(new Set(headers.map(({ file }) => file)).size, headers.length);
    const bodies = execFileSync("git", ["--no-replace-objects", "cat-file", "--batch"], {
      cwd: root,
      input: headers.map(({ blob }) => blob).join("\n") + "\n",
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
      maxBuffer: 64 * 1024 * 1024,
    });
    const result = new Map<string, Body>();
    let offset = 0;
    for (const header of headers) {
      const end = bodies.indexOf("\n", offset);
      const match = /^([a-f0-9]{40}) blob ([0-9]+)$/u.exec(bodies.subarray(offset, end).toString());
      assert(match && match[1] === header.blob);
      const body = bodies.subarray(end + 1, end + 1 + Number(match[2]));
      const cell = bodyCell(body);
      assert.equal(cell.blob, header.blob);
      assert.equal(body.length, Number(match[2]));
      offset = end + 1 + body.length;
      assert.equal(bodies[offset++], 10);
      result.set(header.file, { ...cell, body });
    }
    assert.equal(offset, bodies.length);
    return result;
  },
  physical(root, file) {
    const target = path.join(root, file);
    let stat;
    try {
      stat = fs.lstatSync(target);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") return null;
      throw error;
    }
    assert(stat.isFile() && (stat.mode & 0o777) === 0o644, "one physical regular source: " + file);
    const body = fs.readFileSync(target);
    return { ...bodyCell(body), body };
  },
});
function matches(body: Body | null, cell: Cell | null) {
  if (cell === null) return body === null;
  if (!body) return false;
  const actual = bodyCell(body.body, body.mode);
  return (
    actual.mode === "100644" &&
    JSON.stringify({ mode: body.mode, blob: body.blob, bytes: body.bytes, sha256: body.sha256 }) ===
      JSON.stringify(actual) &&
    JSON.stringify(actual) === JSON.stringify(cell)
  );
}
function sourceCommit(body: Buffer, source: string) {
  assert.equal(object("commit", body), source);
  assert.match(body.toString(), /^tree [a-f0-9]{40}\n/u);
}
function originals() {
  assert.equal(hash(reviewBytes()), reviewSHA256);
  for (const role of ["sdk", "component"] as const) {
    const original = review.original[role];
    for (const file of original.files) {
      const physical = path.join(path.dirname(reviewFile.pathname), original.directory, file.file);
      const stat = fs.lstatSync(physical);
      assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
      assert.deepEqual(bodyCell(fs.readFileSync(physical)), {
        mode: file.mode,
        blob: file.blob,
        bytes: file.bytes,
        sha256: file.sha256,
      });
    }
  }
}
const accepted = new WeakMap<object, { root: string; reader: Reader }>();
export function qualifyDualCurrentFrame(root: string) {
  const reader = actualReader;
  originals();
  assert.equal(reader.replacements(root).length, 0, "replacement context refused");
  const head = reader.head(root);
  assert.match(head, /^[a-f0-9]{40}$/u);
  sourceCommit(reader.commit(root, head), head);
  const beforeCommit = reader.commit(root, review.beforeSource);
  sourceCommit(beforeCommit, review.beforeSource);
  assert.equal(beforeCommit.length, review.beforeCommit.bytes);
  assert.equal(hash(beforeCommit), review.beforeCommit.sha256);
  const before = reader.frame(root, review.beforeSource);
  const projection = projectOwnedDualSourceFrame(
    root,
    head,
    review.beforeSource,
    reader.frame(root, head),
    (file) => reader.physical(root, file),
  );
  const after = projection.raw;
  for (const row of review.beforeRows)
    assert(
      matches(before.get(row.file) ?? null, row.cell),
      "complete actual before source: " + row.file,
    );
  for (const row of review.rows) {
    const raw = after.get(row.file) ?? null;
    const physical = projection.physical(row.file);
    assert(matches(raw, row.final), "complete dual current source: " + row.file);
    assert(matches(physical, row.final), "complete dual physical source: " + row.file);
    assert.deepEqual(physical!.body, raw!.body);
  }
  const receipt = () =>
    Object.freeze({
      head,
      completeCurrentPairs: 111,
      preservedOriginalPairs: 110,
      originalNoDeepPairs: 102,
      originalNamedFamilyBodies: 87,
      acceptanceCredit: "none",
    });
  const result = Object.freeze({
    authority: "complete-reviewed-dual-current-source-frame",
    head,
    completeCombinedPairs: 112,
    owned: receipt(),
    component: receipt(),
    privatePrototype: review.privatePrototype,
    runtimeCredit: false,
  });
  accepted.set(result, { root: path.resolve(root), reader });
  return result;
}
export function admittedReview(root: string, packet: ReturnType<typeof qualifyDualCurrentFrame>) {
  const entry = accepted.get(packet);
  assert(entry && entry.root === path.resolve(root), "same actual source receipt required");
  const current = qualifyDualCurrentFrame(root);
  assert.equal(current.head, packet.head, "captured source owner changed");
  return review;
}
export {
  exerciseDualOriginalFrameControls,
  runDualRetainedReaderLaw,
} from "./warm-type-backed-git-reader-dual-originals.ts";

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import type { Body } from "./warm-type-backed-git-reader-dual-frame.ts";
const sha = (body: Uint8Array) => createHash("sha256").update(body).digest("hex");
const blob = (body: Buffer) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const read = (name: string) => {
  const url = new URL(name, import.meta.url),
    stat = fs.lstatSync(url);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
  return fs.readFileSync(url);
};
const bytes = read("./owned-corsa-dual-source.json");
assert.equal(sha(bytes), "77decdb5e4db83a00b089ae901547eed8c1c14f2a87bb8461135a47cceca8d08");
const data = JSON.parse(bytes.toString());
assert.equal(data.schema, 1);
assert.equal(data.sourceOnly, true);
assert.equal(data.runtimeCredit, false);
assert.equal(data.performanceCredit, false);
const rows = data.parts.flatMap(([name, checksum]: [string, string], i: number) => {
  assert.equal(name, `owned-corsa-dual-source-part-${i}.json`);
  const bytes = read(name);
  assert.equal(sha(bytes), checksum);
  return JSON.parse(bytes.toString());
});
assert.equal(rows.length, 10);
assert.equal(new Set(rows.map(({ file }: { file: string }) => file)).size, 10);
const decode = (cell: {
  rawBase64: string;
  mode: string;
  bytes: number;
  sha256: string;
  blob: string;
}): Body => {
  const body = Buffer.from(cell.rawBase64, "base64");
  assert.equal(body.toString("base64"), cell.rawBase64);
  assert.equal(cell.mode, "100644");
  assert.equal(body.length, cell.bytes);
  assert.equal(sha(body), cell.sha256);
  assert.equal(blob(body), cell.blob);
  return { mode: cell.mode, blob: cell.blob, bytes: cell.bytes, sha256: cell.sha256, body };
};
const pairs = new Map<string, { original: Body; current: Body }>();
for (const row of rows) {
  const pair = { original: decode(row.original), current: decode(row.current) };
  const original = pair.original.body.toString().split(/(?<=\n)/u);
  const restored = pair.current.body.toString().split(/(?<=\n)/u);
  for (const [i, j, k, l, before, after] of row.clauses.toReversed()) {
    assert.equal(original.slice(i, j).join(""), before);
    assert.equal(restored.slice(k, l).join(""), after);
    restored.splice(k, l - k, ...(before ? before.split(/(?<=\n)/u) : []));
  }
  assert.deepEqual(Buffer.from(restored.join("")), pair.original.body);
  pairs.set(row.file, pair);
}
const additional = decode(data.additional);
const copy = (value: Body): Body => ({ ...value, body: Buffer.from(value.body) });

/** All10 actual successors and the registered module precede the unchanged whole112 authority. */
export function projectOwnedDualSourceFrame(
  root: string,
  head: string,
  before: string,
  raw: Map<string, Body>,
  physical: (file: string) => Body | null,
) {
  const old = [...pairs].every(([file, pair]) => raw.get(file)?.body.equals(pair.original.body));
  const current = [...pairs].every(([file, pair]) => raw.get(file)?.body.equals(pair.current.body));
  assert(old || current, "partial, mixed or foreign SDK dual source refuses");
  if (!current) return { raw, physical };
  for (const [file, pair] of pairs) {
    assert.deepEqual(raw.get(file), pair.current, "whole current raw SDK dual source: " + file);
    assert.deepEqual(
      physical(file),
      pair.current,
      "whole current physical SDK dual source: " + file,
    );
  }
  assert.equal(before, data.originalBeforeSource);
  const git = (args: string[]) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: root,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
    });
  const added = data.additionalFile;
  assert.equal(added, "crates/vize_patina/src/linter/script_rules/sfc/registered_sfc.rs");
  assert.equal(git(["ls-tree", "-z", before, "--", added]).length, 0);
  assert.deepEqual(
    git(["ls-tree", "-z", head, "--", added]),
    Buffer.from(`100644 blob ${additional.blob}\t${added}\0`),
  );
  assert.deepEqual(git(["cat-file", "blob", `${head}:${added}`]), additional.body);
  const target = path.join(root, added),
    entry = fs.lstatSync(target);
  assert(entry.isFile() && (entry.mode & 0o777) === 0o644);
  assert.deepEqual(fs.readFileSync(target), additional.body);
  const projected = new Map(raw);
  for (const [file, pair] of pairs) projected.set(file, copy(pair.original));
  return {
    raw: projected,
    physical: (file: string) =>
      pairs.has(file) ? copy(pairs.get(file)!.original) : physical(file),
  };
}

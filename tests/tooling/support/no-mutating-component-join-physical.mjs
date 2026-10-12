import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const joinSHA = (body) => createHash("sha256").update(body).digest("hex");
export const joinObjectID = (type, body) =>
  createHash("sha1").update(`${type} ${body.length}\0`).update(body).digest("hex");
export function joinPath(name) {
  assert(name && !path.isAbsolute(name) && path.normalize(name) === name, name);
  assert(!name.split("/").includes("..") && !name.includes("\0"), name);
  return name;
}
export function joinIdentity(body, row) {
  assert(Buffer.isBuffer(body), row.path);
  assert.equal(row.mode, "100644", row.path);
  assert.equal(body.length, row.bytes, row.path);
  assert.equal(joinSHA(body), row.sha256, row.path);
  assert.equal(joinObjectID("blob", body), row.gitBlob, row.path);
  return body;
}
export function joinPhysical(root, row) {
  const target = path.join(root, joinPath(row.path));
  const stat = fs.lstatSync(target);
  assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o644, row.path);
  assert.equal(fs.realpathSync(target), path.join(fs.realpathSync(root), row.path), row.path);
  return joinIdentity(fs.readFileSync(target), row);
}
export function joinGit(root, args, input) {
  for (const key of [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_INDEX_FILE",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
  ])
    assert.equal(process.env[key], undefined, "literal source refuses Git override: " + key);
  return execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    input,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
  });
}
export function joinRawBody(root, source, row) {
  assert.match(source, /^[a-f0-9]{40}$/u);
  const entry = joinGit(root, ["ls-tree", "-z", source, "--", joinPath(row.path)]);
  assert.deepEqual(entry, Buffer.from("100644 blob " + row.gitBlob + "\t" + row.path + "\0"));
  return joinIdentity(joinGit(root, ["cat-file", "blob", row.gitBlob]), row);
}
export function joinRawBodies(root, source, rows) {
  assert.match(source, /^[a-f0-9]{40}$/u);
  const expected = rows
    .map((row) => "100644 blob " + row.gitBlob + "\t" + row.path + "\0")
    .join("");
  assert.deepEqual(
    joinGit(root, ["ls-tree", "-r", "-z", source, "--", ...rows.map((row) => joinPath(row.path))]),
    Buffer.from(expected),
  );
  const unique = [...new Map(rows.map((row) => [row.gitBlob, row])).values()];
  const packet = joinGit(
    root,
    ["cat-file", "--batch"],
    unique.map((row) => row.gitBlob).join("\n") + "\n",
  );
  const objects = new Map();
  let cursor = 0;
  for (const row of unique) {
    const end = packet.indexOf(10, cursor);
    assert(end >= cursor);
    assert.equal(packet.subarray(cursor, end).toString(), row.gitBlob + " blob " + row.bytes);
    const body = packet.subarray(end + 1, end + 1 + row.bytes);
    joinIdentity(body, row);
    assert.equal(packet[end + 1 + row.bytes], 10);
    objects.set(row.gitBlob, body);
    cursor = end + 2 + row.bytes;
  }
  assert.equal(cursor, packet.length);
  return new Map(rows.map((row) => [row.path, joinIdentity(objects.get(row.gitBlob), row)]));
}
export function assertJoinCommitted(root, rows) {
  const head = joinGit(root, ["rev-parse", "HEAD"]).toString().trim();
  assert.match(head, /^[a-f0-9]{40}$/u);
  assert.equal(joinGit(root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  assert.equal(joinObjectID("commit", joinGit(root, ["cat-file", "commit", head])), head);
  const expected = rows
    .map((row) => "100644 blob " + row.gitBlob + "\t" + row.path + "\0")
    .join("");
  assert.deepEqual(
    joinGit(root, ["ls-tree", "-r", "-z", head, "--", ...rows.map((row) => row.path)]),
    Buffer.from(expected),
    "whole committed joined source",
  );
  return head;
}

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";

export const noMutatingCallerAncestryCapsulePath =
  "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody/current-callers/ancestry-b48/literal-raw-objects.json.gz";
export const noMutatingCallerAncestryCapsuleSHA256 =
  "45c44efa0dd825d8e6cf8d0d9e35f8c1b45b263abbd9463528f5ab0f46b99436";
const installed = new Set();
const head = "2489128d5f8fab1b7442a1e5c09b4e1701cbf185";
const ancestor = "eafe5bed4f602e0787ff9f2f48ec4155d115ab7f";
const incoming = "3dda8f9d71fbd7e701f0805187daff6191865190";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const git = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root, input });
const objectId = (type, body) =>
  createHash("sha1").update(`${type} ${body.length}\0`).update(body).digest("hex");
function commitHeaders(body) {
  const boundary = body.indexOf("\n\n");
  assert(boundary > 0);
  const bytes = body.subarray(0, boundary);
  const text = bytes.toString("utf8");
  assert.deepEqual(Buffer.from(text), bytes);
  const rows = text.split("\n");
  assert.match(rows[0], /^tree [a-f0-9]{40}$/u);
  const parents = rows.filter((row) => row.startsWith("parent "));
  for (const row of parents) assert.match(row, /^parent [a-f0-9]{40}$/u);
  return { tree: rows[0].slice(5), parents: parents.map((row) => row.slice(7)) };
}
function treeEntries(body) {
  const entries = new Map();
  for (let cursor = 0; cursor < body.length;) {
    const space = body.indexOf(32, cursor);
    const zero = body.indexOf(0, space);
    assert(space > cursor && zero > space && zero + 21 <= body.length);
    const mode = body.subarray(cursor, space).toString("ascii");
    const nameBytes = body.subarray(space + 1, zero);
    const name = nameBytes.toString("utf8");
    assert.deepEqual(Buffer.from(name), nameBytes);
    assert(!entries.has(name) && !name.includes("/") && name !== "." && name !== "..");
    assert.match(mode, /^(40000|100644|100755|120000|160000)$/u);
    entries.set(name, { mode, id: body.subarray(zero + 1, zero + 21).toString("hex") });
    cursor = zero + 21;
  }
  return entries;
}

/** Validate raw object identities and their complete rooted path/first-parent geometry. */
export function verifyNoMutatingCallerAncestryCapsule(capsule) {
  assert.equal(capsule.schema, "vize.no-mutating.current-callers.literal-first-parent-capsule");
  assert.equal(capsule.version, 1);
  assert.equal(capsule.head, head);
  assert.equal(capsule.ancestor, ancestor);
  assert.equal(capsule.historicalIncoming, incoming);
  assert.equal(capsule.maximumOriginalCommitCount, 64);
  assert.equal(
    capsule.completeHistoricalPhysicalFrameSHA256,
    "c498e0b14257b95acfe7f4ea4e2471d3e25c6e51d70568604af248f59f279d87",
  );
  assert.deepEqual(capsule.credit, {
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
  });
  const objects = new Map();
  for (const row of capsule.objects) {
    assert.match(row.id, /^[a-f0-9]{40}$/u);
    assert(["commit", "tree", "blob"].includes(row.type));
    assert(!objects.has(row.id));
    const body = Buffer.from(row.base64, "base64");
    assert.equal(body.toString("base64"), row.base64);
    assert.equal(body.length, row.bytes);
    assert.equal(sha(body), row.sha256);
    assert.equal(objectId(row.type, body), row.id);
    objects.set(row.id, { type: row.type, body });
  }
  const used = new Set();
  const take = (id, type) => {
    const object = objects.get(id);
    assert(object && object.type === type, `complete declared ${type}: ${id}`);
    used.add(id);
    return object.body;
  };
  const chain = capsule.firstParentCommits;
  assert.equal(chain.length, 13);
  assert(chain.length <= capsule.maximumOriginalCommitCount);
  assert.equal(new Set(chain).size, chain.length);
  assert.equal(chain[0], head);
  assert.equal(chain.at(-1), ancestor);
  for (const [index, id] of chain.entries()) {
    const headers = commitHeaders(take(id, "commit"));
    if (index < chain.length - 1) assert.equal(headers.parents[0], chain[index + 1]);
  }
  assert.deepEqual(
    capsule.sourceScopes.map(({ source }) => source),
    [ancestor, incoming],
  );
  for (const [index, scope] of capsule.sourceScopes.entries()) {
    const rootTree = commitHeaders(take(scope.source, "commit")).tree;
    assert.equal(scope.rootTree, rootTree);
    assert.equal(scope.files.length, index === 0 ? 111 : 4);
    assert.equal(new Set(scope.files.map(([name]) => name)).size, scope.files.length);
    for (const [name, expected] of scope.files) {
      assert(name && !path.isAbsolute(name) && path.normalize(name) === name);
      assert(!name.split("/").includes(".."));
      let current = rootTree;
      let actual = null;
      for (const [position, part] of name.split("/").entries()) {
        const entry = treeEntries(take(current, "tree")).get(part);
        if (!entry) break;
        if (position === name.split("/").length - 1) {
          assert.equal(entry.mode, "100644");
          take(entry.id, "blob");
          actual = entry.id;
        } else {
          assert.equal(entry.mode, "40000");
          current = entry.id;
        }
      }
      assert.equal(actual, expected, `${scope.source}:${name}`);
    }
  }
  assert.equal(objects.size, 138);
  assert.deepEqual(
    [...used].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0)),
    [...objects.keys()].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0)),
    "no unowned object injection",
  );
  return objects;
}

/** Seed only a newly owned private object database; shared source metadata stays read-only. */
export function seedNoMutatingCallerAncestry(root, owned) {
  for (const name of [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_INDEX_FILE",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
  ])
    assert(!process.env[name], "private ancestry refuses inherited Git path override: " + name);
  const source = path.join(root, noMutatingCallerAncestryCapsulePath);
  const status = fs.lstatSync(source);
  assert(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111));
  assert.equal(
    fs.realpathSync(source),
    path.join(fs.realpathSync(root), noMutatingCallerAncestryCapsulePath),
  );
  const bytes = fs.readFileSync(source);
  assert.equal(sha(bytes), noMutatingCallerAncestryCapsuleSHA256);
  const objects = verifyNoMutatingCallerAncestryCapsule(JSON.parse(gunzipSync(bytes)));
  const physical = fs.realpathSync(owned);
  assert.notEqual(physical, fs.realpathSync(root));
  const gitDir = fs.realpathSync(path.join(owned, ".git"));
  const objectDir = fs.realpathSync(path.join(gitDir, "objects"));
  assert.equal(gitDir, path.join(physical, ".git"));
  assert.equal(objectDir, path.join(physical, ".git/objects"));
  assert.equal(git(owned, ["rev-parse", "--git-common-dir"]).toString().trim(), ".git");
  assert.equal(
    fs.realpathSync(
      path.resolve(owned, git(owned, ["rev-parse", "--git-path", "objects"]).toString().trim()),
    ),
    objectDir,
  );
  const beforeHead = fs.readFileSync(path.join(gitDir, "HEAD"));
  assert([head, ancestor].some((id) => beforeHead.equals(Buffer.from(id + "\n"))));
  const shallow = path.join(gitDir, "shallow");
  const beforeShallow = fs.existsSync(shallow) ? fs.readFileSync(shallow) : null;
  if (!installed.has(physical)) {
    for (const [id, { type, body }] of objects)
      assert.equal(
        git(owned, ["hash-object", "-w", "-t", type, "--stdin"], body).toString().trim(),
        id,
      );
  }
  const actual = git(owned, ["cat-file", "--batch"], [...objects.keys()].join("\n") + "\n");
  let offset = 0;
  for (const [id, { type, body }] of objects) {
    const end = actual.indexOf(10, offset);
    assert(end >= offset);
    assert.equal(actual.subarray(offset, end).toString(), `${id} ${type} ${body.length}`);
    const start = end + 1;
    assert.deepEqual(actual.subarray(start, start + body.length), body);
    assert.equal(actual[start + body.length], 10);
    offset = start + body.length + 1;
  }
  assert.equal(offset, actual.length);
  installed.add(physical);
  assert.deepEqual(fs.readFileSync(path.join(gitDir, "HEAD")), beforeHead);
  assert.deepEqual(fs.existsSync(shallow) ? fs.readFileSync(shallow) : null, beforeShallow);
  assert.deepEqual(fs.readFileSync(source), bytes);
  return {
    sourceOnly: true,
    firstParentCommits: 13,
    additionalIncomingCommits: 1,
    rawObjects: 138,
    native: false,
    performance: false,
  };
}

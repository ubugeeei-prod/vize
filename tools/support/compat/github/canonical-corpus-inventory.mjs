import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { join, posix } from "node:path";
import { walkCommittedGraph } from "./canonical-corpus-logical-walk.mjs";

export const corpusRoot = "tests/_fixtures/_git";
const excluded = new Set(["node_modules", "_git-worktrees"]);
const fullOid = /^[0-9a-f]{40}$/;
const decode = (bytes) => new TextDecoder("utf-8", { fatal: true }).decode(bytes);
export const gitObjectId = (type, bytes) =>
  createHash("sha1").update(`${type} ${bytes.length}\0`).update(bytes).digest("hex");

function treeEntries(bytes) {
  const entries = [];
  let offset = 0;
  const names = new Set();
  while (offset < bytes.length) {
    const space = bytes.indexOf(32, offset);
    const zero = bytes.indexOf(0, space + 1);
    assert(space > offset && zero > space && zero + 21 <= bytes.length, "Invalid Git tree object");
    const mode = bytes.subarray(offset, space).toString("ascii");
    const name = decode(bytes.subarray(space + 1, zero));
    assert(
      ["40000", "100644", "100755", "120000", "160000"].includes(mode),
      "Invalid Git tree mode",
    );
    assert(
      name && ![".", ".."].includes(name) && !name.includes("/") && !names.has(name),
      "Invalid Git tree name",
    );
    names.add(name);
    entries.push({ mode, name, id: bytes.subarray(zero + 1, zero + 21).toString("hex") });
    offset = zero + 21;
  }
  return entries;
}

function objectMap(repository) {
  assert(
    Array.isArray(repository.objects) && repository.objects.length,
    "Missing committed Git objects",
  );
  const objects = new Map();
  for (const object of repository.objects) {
    assert(
      fullOid.test(object.id) && ["commit", "tree", "blob"].includes(object.type),
      "Invalid Git object identity",
    );
    assert(typeof object.bytes === "string", "Missing Git object bytes");
    const bytes = Buffer.from(object.bytes, "base64");
    assert.equal(bytes.toString("base64"), object.bytes, "Invalid Git object encoding");
    assert.equal(gitObjectId(object.type, bytes), object.id, "Committed Git object bytes changed");
    assert(!objects.has(object.id), "Duplicate Git object");
    objects.set(object.id, { ...object, bytes });
  }
  return objects;
}

export function committedGraph(proof, gitlinks) {
  assert.equal(proof.schema, "vize.canonical-committed-inventory", "Foreign committed inventory");
  assert.equal(proof.version, 1, "Unknown committed inventory version");
  assert.deepEqual(
    proof.repositories.map(({ path, sha }) => ({ path, sha })),
    gitlinks,
    "Committed inventory omitted or changed gitlinks",
  );
  const nodes = new Map([["", { mode: "40000", path: "" }]]);
  const children = new Map([["", new Set()]]);
  function add(path, node) {
    assert(!nodes.has(path), "Overlapping committed paths");
    const parent = posix.dirname(path) === "." ? "" : posix.dirname(path);
    if (!nodes.has(parent)) add(parent, { mode: "40000" });
    assert.equal(nodes.get(parent).mode, "40000", "Foreign committed directory owner");
    nodes.set(path, { ...node, path });
    if (node.mode === "40000" || node.mode === "160000") children.set(path, new Set());
    children.get(parent).add(posix.basename(path));
  }
  for (const repository of proof.repositories) {
    assert(
      repository.path.startsWith(`${corpusRoot}/`) && fullOid.test(repository.sha),
      "Unsafe committed repository",
    );
    const prefix = repository.path.slice(corpusRoot.length + 1);
    assert(
      prefix && !prefix.split("/").some((part) => ["", ".", ".."].includes(part)),
      "Unsafe committed path",
    );
    const objects = objectMap(repository);
    const used = new Set();
    const read = (id, type) => {
      const object = objects.get(id);
      assert(object && object.type === type, "Missing committed tree or symlink object");
      used.add(id);
      return object.bytes;
    };
    const commit = read(repository.sha, "commit");
    const match = /^tree ([0-9a-f]{40})\n/.exec(commit.toString("utf8"));
    assert(match, "Invalid committed root tree");
    add(prefix, { mode: "40000", id: match[1] });
    const active = new Set();
    const visit = (path, id) => {
      assert(!active.has(id), "Committed tree cycle");
      active.add(id);
      for (const entry of treeEntries(read(id, "tree"))) {
        const child = `${path}/${entry.name}`;
        const node = { mode: entry.mode, id: entry.id };
        if (entry.mode === "120000") node.target = decode(read(entry.id, "blob"));
        add(child, { ...node, repository: repository.path, revision: repository.sha });
        if (entry.mode === "40000") visit(child, entry.id);
      }
      active.delete(id);
    };
    visit(prefix, match[1]);
    assert.equal(used.size, objects.size, "Foreign unused Git objects");
  }
  return { nodes, children };
}

export function verifyCommittedInventory(proof, gitlinks) {
  const { nodes, children } = committedGraph(proof, gitlinks);
  if (proof.kernelBoundary) return walkCommittedGraph(nodes, children, proof.kernelBoundary).files;
  const resolveNode = (path, active = new Set()) => {
    assert(!path.startsWith("/") && !path.includes("\0"), "Foreign committed symlink path");
    let current = "";
    for (const [index, part] of path.split("/").entries()) {
      if (index > 0)
        assert(
          ["40000", "160000"].includes(nodes.get(current).mode),
          "Invalid committed symlink traversal",
        );
      if (!part || part === ".") continue;
      if (part === "..") {
        assert(current, "Foreign committed symlink path");
        current = posix.dirname(current) === "." ? "" : posix.dirname(current);
        continue;
      }
      const next = current ? `${current}/${part}` : part;
      const node = nodes.get(next);
      assert(node, "Missing committed symlink target");
      if (node.mode === "120000") {
        assert(!active.has(next), `Committed symlink cycle: ${next} -> ${node.target}`);
        active.add(next);
        assert(node.target && !node.target.startsWith("/"), "Foreign committed symlink path");
        current = resolveNode(`${posix.dirname(next)}/${node.target}`, active).path;
        active.delete(next);
      } else current = next;
    }
    return nodes.get(current);
  };
  const files = [];
  const walk = (logical, directory, ancestors = new Set()) => {
    assert(
      !ancestors.has(directory.path),
      `Committed directory cycle: ${logical} -> ${directory.path}`,
    );
    ancestors.add(directory.path);
    for (const name of [...children.get(directory.path)].sort((left, right) =>
      left < right ? -1 : left > right ? 1 : 0,
    )) {
      if (excluded.has(name)) continue;
      const node = resolveNode(directory.path ? `${directory.path}/${name}` : name);
      const path = logical ? `${logical}/${name}` : name;
      if (["40000", "160000"].includes(node.mode)) {
        walk(path, node, ancestors);
      } else if (name.endsWith(".vue")) files.push([path, node.id]);
    }
    ancestors.delete(directory.path);
  };
  walk("", nodes.get(""));
  files.sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0));
  assert(files.length > 0, "Committed canonical Vue inventory is empty");
  return files;
}

function readObjects(repository, requests) {
  const bytes = execFileSync("git", ["-C", repository, "cat-file", "--batch"], {
    input: `${requests.map(([id]) => id).join("\n")}\n`,
    maxBuffer: 256 * 1024 * 1024,
  });
  let offset = 0;
  const objects = [];
  for (const [id, type] of requests) {
    const newline = bytes.indexOf(10, offset);
    const header = /^([0-9a-f]{40}) (commit|tree|blob) (\d+)$/.exec(
      bytes.subarray(offset, newline).toString("ascii"),
    );
    assert(header && header[1] === id && header[2] === type, "Missing pinned Git object");
    const length = Number(header[3]);
    assert(
      Number.isSafeInteger(length) && length >= 0 && newline + length + 1 < bytes.length,
      "Invalid Git object size",
    );
    const raw = bytes.subarray(newline + 1, newline + length + 1);
    assert.equal(bytes[newline + length + 1], 10, "Incomplete Git object bytes");
    objects.push({ id, type, bytes: raw.toString("base64") });
    offset = newline + length + 2;
  }
  assert.equal(offset, bytes.length, "Unexpected Git object output");
  return objects;
}

export function createCommittedProof(cwd, gitlinks, repositoryFor = (row) => join(cwd, row.path)) {
  const repositories = gitlinks.map((row) => {
    const repository = repositoryFor(row);
    const commit = execFileSync("git", ["-C", repository, "cat-file", "commit", row.sha], {
      maxBuffer: 256 * 1024 * 1024,
    });
    const match = /^tree ([0-9a-f]{40})\n/.exec(commit.toString("utf8"));
    assert(match, "Missing pinned root tree");
    const tree = execFileSync("git", ["-C", repository, "ls-tree", "-trz", row.sha], {
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    });
    const requests = new Map([
      [row.sha, "commit"],
      [match[1], "tree"],
    ]);
    for (const line of tree.split("\0").filter(Boolean)) {
      const entry = /^(\d+) (tree|blob|commit) ([0-9a-f]{40})\t/.exec(line);
      assert(entry, "Invalid pinned tree inventory");
      if (entry[2] === "tree" || entry[1] === "120000") requests.set(entry[3], entry[2]);
    }
    return { ...row, objects: readObjects(repository, [...requests]) };
  });
  return { schema: "vize.canonical-committed-inventory", version: 1, repositories };
}

export function createCommittedInventory(cwd, gitlinks, repositoryFor) {
  const proof = createCommittedProof(cwd, gitlinks, repositoryFor);
  return { proof, files: verifyCommittedInventory(proof, gitlinks) };
}

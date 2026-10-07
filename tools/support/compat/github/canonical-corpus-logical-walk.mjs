import assert from "node:assert/strict";
import { posix } from "node:path";

export const cycleOwner = {
  repository: "tests/_fixtures/_git/jellyfin-vue",
  revision: "6b35d977335cab224c4536d18dc22d15a63d5185",
  path: "jellyfin-vue/packaging/deb/root",
  blob: "c25bddb6dd4666c6eb8cc92e33f1d60f64c3162b",
  target: "../..",
};
export const pathOrder = (left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right));
const directory = (node) => ["40000", "160000"].includes(node.mode);
const parent = (path) => (posix.dirname(path) === "." ? "" : posix.dirname(path));
export const needsNativeWalk = (gitlinks) =>
  gitlinks.some((row) => row.path === cycleOwner.repository && row.sha === cycleOwner.revision);

function knownCycle(node) {
  return (
    node.path === cycleOwner.path &&
    node.repository === cycleOwner.repository &&
    node.revision === cycleOwner.revision &&
    node.id === cycleOwner.blob &&
    node.target === cycleOwner.target
  );
}

// Resolve the WHOLE logical path, counting every alias traversal again.
// Resolving only directory.path would discard the native collector's multiplicity.
export function resolveLogical(nodes, logical, rejectedAt = null) {
  assert(!logical.startsWith("/") && !logical.includes("\0"), "Foreign committed symlink path");
  const pending = logical.split("/");
  let current = "";
  let followed = 0;
  let allowed = false;
  const active = new Set();
  while (pending.length) {
    const part = pending.shift();
    if (typeof part === "object") {
      active.delete(part.release);
      continue;
    }
    assert(directory(nodes.get(current)), "Invalid committed symlink traversal");
    if (!part || part === ".") continue;
    if (part === "..") {
      assert(current, "Foreign committed symlink path");
      current = parent(current);
      continue;
    }
    const next = current ? `${current}/${part}` : part;
    const node = nodes.get(next);
    assert(node, "Missing committed symlink target");
    if (node.mode !== "120000") {
      current = next;
      continue;
    }
    assert(node.target && !node.target.startsWith("/"), "Foreign committed symlink path");
    allowed ||= knownCycle(node);
    assert(!active.has(next), "Committed symlink cycle");
    active.add(next);
    followed += 1;
    if (rejectedAt !== null && followed >= rejectedAt) {
      assert(allowed, "Foreign committed kernel boundary");
      return { mode: "kernel-boundary", followed };
    }
    pending.unshift(...node.target.split("/"), { release: next });
  }
  return { ...nodes.get(current), followed, allowed };
}

// Establish ownership before executing the native collector on physical paths.
// Only the exact committed ancestor link above may revisit a directory.
export function validateLogicalGraph(nodes, children) {
  const visit = (logical, node, ancestors) => {
    assert(!ancestors.has(node.path), "Committed directory cycle");
    const nextAncestors = new Set([...ancestors, node.path]);
    for (const name of children.get(node.path)) {
      if (["node_modules", "_git-worktrees"].includes(name)) continue;
      const path = logical ? `${logical}/${name}` : name;
      const child = resolveLogical(nodes, path);
      if (!directory(child) || ["node_modules", "_git-worktrees"].includes(name)) continue;
      if (nextAncestors.has(child.path)) {
        assert(child.allowed, `Committed directory cycle: ${path} -> ${child.path}`);
      } else visit(path, child, nextAncestors);
    }
  };
  visit("", nodes.get(""), new Set());
}

export function walkCommittedGraph(nodes, children, boundary) {
  assert.equal(boundary.code, "ELOOP", "Unqualified native kernel boundary");
  assert(
    Number.isSafeInteger(boundary.firstRejected) && boundary.firstRejected > 1,
    "Missing observed native symlink boundary",
  );
  assert(
    Array.isArray(boundary.rejected) && boundary.rejected.length > 0,
    "Missing physical kernel boundary paths",
  );
  validateLogicalGraph(nodes, children);
  const files = [];
  const rejected = [];
  let successfulFollows = 0;
  const visit = (logical, node) => {
    for (const name of [...children.get(node.path)].sort(pathOrder)) {
      if (["node_modules", "_git-worktrees"].includes(name)) continue;
      const path = logical ? `${logical}/${name}` : name;
      const child = resolveLogical(nodes, path, boundary.firstRejected);
      if (child.mode === "kernel-boundary") {
        assert.notEqual(posix.extname(name), ".vue", "Unreadable committed Vue input");
        rejected.push({ path, code: "ELOOP", followed: child.followed });
      } else {
        successfulFollows = Math.max(successfulFollows, child.followed);
        if (directory(child)) {
          if (!["node_modules", "_git-worktrees"].includes(name)) visit(path, child);
        } else if (posix.extname(name) === ".vue") files.push([path, child.id]);
      }
    }
  };
  visit("", nodes.get(""));
  assert.equal(successfulFollows + 1, boundary.firstRejected, "Unwitnessed kernel boundary");
  assert.deepEqual(rejected, boundary.rejected, "Committed kernel boundary paths changed");
  assert(files.length > 0, "Committed canonical Vue inventory is empty");
  return { files, rejected };
}

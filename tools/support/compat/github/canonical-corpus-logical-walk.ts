import assert from "node:assert/strict";
import { posix } from "node:path";

import type {
  CorpusNode,
  CorpusNodes,
  CorpusChildren,
  Boundary,
  RejectedPath,
  KernelPrefixWitness,
} from "./canonical-corpus-types.ts";
export type {
  CorpusNode,
  CorpusNodes,
  CorpusChildren,
  Boundary,
  RejectedPath,
  KernelPrefixWitness,
  VueFile,
} from "./canonical-corpus-types.ts";
type ResolvedNode = CorpusNode & { followed: number; allowed: boolean };
type BoundaryNode = { mode: "kernel-boundary"; followed: number };

export const cycleOwner = {
  repository: "tests/_fixtures/_git/jellyfin-vue",
  revision: "6b35d977335cab224c4536d18dc22d15a63d5185",
  path: "jellyfin-vue/packaging/deb/root",
  blob: "c25bddb6dd4666c6eb8cc92e33f1d60f64c3162b",
  target: "../..",
};
export const pathOrder = (left: string, right: string) =>
  Buffer.compare(Buffer.from(left), Buffer.from(right));
const directory = (
  node: CorpusNode | undefined,
): node is CorpusNode & { mode: "40000" | "160000" } => ["40000", "160000"].includes(node!.mode);
const kernelBoundary = (node: ResolvedNode | BoundaryNode): node is BoundaryNode =>
  node.mode === "kernel-boundary";
const parent = (path: string) => (posix.dirname(path) === "." ? "" : posix.dirname(path));
export const needsNativeWalk = (gitlinks: Array<{ path: string; sha: string }>) =>
  gitlinks.some((row) => row.path === cycleOwner.repository && row.sha === cycleOwner.revision);

function knownCycle(node: CorpusNode) {
  return (
    node.path === cycleOwner.path &&
    node.repository === cycleOwner.repository &&
    node.revision === cycleOwner.revision &&
    node.id === cycleOwner.blob &&
    node.target === cycleOwner.target
  );
}

// A boundary prefix is witnessed only when this exact owned ancestor link is
// the terminal lookup; its ../.. target cannot hide an additional finite alias.
export function isKnownCycleLookup(nodes: CorpusNodes, logical: string) {
  const owner = resolveLogical(nodes, parent(logical));
  const selected = owner.path
    ? `${owner.path}/${posix.basename(logical)}`
    : posix.basename(logical);
  const node = nodes.get(selected);
  return node?.mode === "120000" && knownCycle(node);
}

// Called only after stat(logical) returned ELOOP while the traversal is inside
// its already successful physical parent. No inferred minimum or OS limit is
// used: the terminal owned link adds exactly one follow to that actual prefix.
export function kernelPrefixWitness(
  nodes: CorpusNodes,
  logical: string,
  successfulPrefix: string,
): KernelPrefixWitness | undefined {
  if (!isKnownCycleLookup(nodes, logical)) return;
  assert.equal(parent(logical), successfulPrefix, "Foreign successful kernel prefix");
  const preceding = resolveLogical(nodes, successfulPrefix);
  const refused = resolveLogical(nodes, logical);
  assert.equal(
    preceding.followed + 1,
    refused.followed,
    "Kernel prefix is not a single owned link",
  );
  return {
    path: logical,
    followed: refused.followed,
    preceding: { path: successfulPrefix, followed: preceding.followed },
  };
}

export function qualifyKernelBoundary(
  nodes: CorpusNodes,
  rejected: RejectedPath[],
  successfulFollows: number,
  witnesses: KernelPrefixWitness[],
): Boundary {
  assert(rejected.length > 0, "Missing actual native kernel boundary observation");
  assert(witnesses.length > 0, "Missing actual owned kernel prefix witness");
  for (const witness of witnesses) {
    assert.deepEqual(
      kernelPrefixWitness(nodes, witness.path, witness.preceding.path),
      witness,
      "Foreign kernel prefix witness",
    );
    assert(
      rejected.some(
        (row) =>
          row.path === witness.path && row.code === "ELOOP" && row.followed === witness.followed,
      ),
      "Kernel prefix lacks actual refusal",
    );
  }
  const firstRejected = witnesses[0].followed;
  assert(
    witnesses.every((witness) => witness.followed === firstRejected),
    "Inconsistent native kernel prefix boundary",
  );
  assert.equal(
    successfulFollows + 1,
    firstRejected,
    "Native kernel boundary lacks preceding success",
  );
  return { code: "ELOOP", firstRejected, rejected };
}

// Resolve the WHOLE logical path, counting every alias traversal again.
// Resolving only directory.path would discard the native collector's multiplicity.
export function resolveLogical(nodes: CorpusNodes, logical: string): ResolvedNode;
export function resolveLogical(
  nodes: CorpusNodes,
  logical: string,
  rejectedAt: number,
): ResolvedNode | BoundaryNode;
export function resolveLogical(
  nodes: CorpusNodes,
  logical: string,
  rejectedAt: number | null = null,
): ResolvedNode | BoundaryNode {
  assert(!logical.startsWith("/") && !logical.includes("\0"), "Foreign committed symlink path");
  const pending: Array<string | { release: string }> = logical.split("/");
  let current = "";
  let followed = 0;
  let allowed = false;
  const active = new Set<string>();
  while (pending.length) {
    const part = pending.shift()!;
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
  return { ...nodes.get(current)!, followed, allowed };
}

// Establish ownership before executing the native collector on physical paths.
// Only the exact committed ancestor link above may revisit a directory.
export function validateLogicalGraph(nodes: CorpusNodes, children: CorpusChildren) {
  const visit = (logical: string, node: CorpusNode, ancestors: Set<string>) => {
    assert(!ancestors.has(node.path), "Committed directory cycle");
    const nextAncestors = new Set([...ancestors, node.path]);
    for (const name of children.get(node.path)!) {
      if (["node_modules", "_git-worktrees"].includes(name)) continue;
      const path = logical ? `${logical}/${name}` : name;
      const child = resolveLogical(nodes, path);
      if (!directory(child) || ["node_modules", "_git-worktrees"].includes(name)) continue;
      if (nextAncestors.has(child.path)) {
        assert(child.allowed, `Committed directory cycle: ${path} -> ${child.path}`);
      } else visit(path, child, nextAncestors);
    }
  };
  visit("", nodes.get("")!, new Set());
}

export function walkCommittedGraph(
  nodes: CorpusNodes,
  children: CorpusChildren,
  boundary: Boundary,
) {
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
  const files: Array<[string, string]> = [];
  const rejected: RejectedPath[] = [];
  let successfulFollows = 0;
  const visit = (logical: string, node: CorpusNode) => {
    for (const name of [...children.get(node.path)!].sort(pathOrder)) {
      if (["node_modules", "_git-worktrees"].includes(name)) continue;
      const path = logical ? `${logical}/${name}` : name;
      const child = resolveLogical(nodes, path, boundary.firstRejected);
      if (kernelBoundary(child)) {
        assert.notEqual(posix.extname(name), ".vue", "Unreadable committed Vue input");
        // The kernel stopped at the certified prefix, while provenance retains
        // every alias the complete committed input would have followed.
        rejected.push({ path, code: "ELOOP", followed: resolveLogical(nodes, path).followed });
      } else {
        successfulFollows = Math.max(successfulFollows, child.followed);
        if (directory(child)) {
          if (!["node_modules", "_git-worktrees"].includes(name)) visit(path, child);
        } else if (posix.extname(name) === ".vue") files.push([path, child.id!]);
      }
    }
  };
  visit("", nodes.get("")!);
  assert.equal(successfulFollows + 1, boundary.firstRejected, "Unwitnessed kernel boundary");
  assert.deepEqual(rejected, boundary.rejected, "Committed kernel boundary paths changed");
  assert(files.length > 0, "Committed canonical Vue inventory is empty");
  return { files, rejected };
}

// The kernel-limited original walk remains a separate, complete observation.
// Qualify the successor using committed identities, never physical path metadata.
export function finiteCommittedInventory(
  nodes: CorpusNodes,
  children: CorpusChildren,
  boundary: Boundary,
) {
  const original = walkCommittedGraph(nodes, children, boundary).files;
  const files: Array<[string, string]> = [];
  const cut: string[] = [];
  const visit = (logical: string, node: CorpusNode, edges: Set<string>) => {
    for (const name of [...children.get(node.path)!].sort(pathOrder)) {
      if (["node_modules", "_git-worktrees"].includes(name)) continue;
      const path = logical ? `${logical}/${name}` : name;
      const child = resolveLogical(nodes, path);
      if (directory(child)) {
        const edge = JSON.stringify([node.path, child.path]);
        if (edges.has(edge)) {
          assert(child.allowed, "Foreign finite directory edge repetition");
          cut.push(path);
        } else visit(path, child, new Set([...edges, edge]));
      } else if (posix.extname(name) === ".vue") files.push([path, child.id!]);
    }
  };
  visit("", nodes.get("")!, new Set());
  const kept = original.filter(([path]) => !cut.some((prefix) => path.startsWith(`${prefix}/`)));
  assert.deepEqual(files, kept, "Finite walk changed paths beyond ancestor-loop repetitions");
  const sources = (rows: Array<[string, string]>) =>
    new Set(rows.map(([path, blob]) => JSON.stringify([resolveLogical(nodes, path).path, blob])));
  assert.deepEqual(
    sources(files),
    sources(original),
    "Finite walk omitted an original source or blob",
  );
  return { files, original, cut };
}

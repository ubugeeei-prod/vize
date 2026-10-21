import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
  writeFileSync,
  lstatSync,
  readlinkSync,
} from "node:fs";
import { extname, join, relative, sep } from "node:path";
import { gitObjectId } from "./canonical-corpus-inventory.mjs";
import {
  finiteCommittedInventory,
  pathOrder,
  resolveLogical,
  validateLogicalGraph,
  walkCommittedGraph,
} from "./canonical-corpus-logical-walk.ts";

const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const excluded = (name) => ["node_modules", "_git-worktrees"].includes(name);

export function collectFinitePhysical(root, nodes, children, boundary) {
  validateLogicalGraph(nodes, children);
  const files = [];
  const physicalRoot = realpathSync(root);
  const visit = (logical, physical, edges) => {
    for (const name of readdirSync(physical).sort(pathOrder)) {
      if (excluded(name)) continue;
      const path = join(logical, name);
      const selected = join(physical, name);
      if (lstatSync(selected).isSymbolicLink()) {
        const path = relative(physicalRoot, selected).split(sep).join("/");
        const node = nodes.get(path);
        assert(node?.mode === "120000", "Foreign finite physical symlink");
        assert.equal(
          gitObjectId("blob", readlinkSync(selected, { encoding: "buffer" })),
          node.id,
          "Finite physical symlink bytes changed",
        );
      }
      const resolved = realpathSync(selected);
      assert(
        !relative(physicalRoot, resolved).startsWith(`..${sep}`),
        "Foreign finite physical input",
      );
      assert.notEqual(relative(physicalRoot, resolved), "..", "Foreign finite physical input");
      if (statSync(selected).isDirectory()) {
        const edge = JSON.stringify([physical, resolved]);
        if (!edges.has(edge)) visit(path, resolved, new Set([...edges, edge]));
      } else if (extname(name) === ".vue") {
        const bytes = readFileSync(resolved);
        files.push([
          relative(root, path).split(sep).join("/"),
          digest(bytes),
          bytes.length,
          gitObjectId("blob", bytes),
        ]);
      }
    }
  };
  visit(root, physicalRoot, new Set());
  assert.deepEqual(
    files.map(([path, , , blob]) => [path, blob]),
    finiteCommittedInventory(nodes, children, boundary).files,
    "Finite physical whole vector differs from committed paths/bytes",
  );
  return files;
}

export function validateOriginalArchive(directory, receipt, parseNativeVector) {
  const vector = readFileSync(join(directory, "original-kernel-vector.nul"));
  const original = JSON.parse(readFileSync(join(directory, "original-kernel-files.json"), "utf8"));
  assert.equal(digest(vector), receipt.originalVectorSha256, "Original kernel vector changed");
  assert.equal(
    digest(JSON.stringify(original)),
    receipt.originalFilesSha256,
    "Original kernel bytes changed",
  );
  assert.deepEqual(
    parseNativeVector(vector),
    original.map(([path]) => path),
    "Original kernel ordered paths changed",
  );
  return original;
}

export function validateOriginalBytes(original, files, nodes) {
  const retainedSources = new Map(
    files.map((row) => [resolveLogical(nodes, row[0]).path, row.slice(1)]),
  );
  for (const [path, ...bytes] of original) {
    const resolved = resolveLogical(nodes, path).path;
    assert.deepEqual(
      retainedSources.get(resolved),
      bytes,
      "Successor omitted original whole source bytes",
    );
  }
}

export function captureFiniteObservation(root, nodes, children, original, directory) {
  const files = collectFinitePhysical(root, nodes, children, original.boundary);
  const committed = finiteCommittedInventory(nodes, children, original.boundary).files;
  const originalVector = Buffer.from(original.files.map(([path]) => `${path}\0`).join(""));
  writeFileSync(join(directory, "original-kernel-vector.nul"), originalVector);
  writeFileSync(
    join(directory, "original-kernel-files.json"),
    JSON.stringify(original.files) + "\n",
  );
  return { files, committed, originalVector };
}

export function nativeWalkIdentity(receipt) {
  return {
    sourceSha256: receipt.sourceSha256,
    bodySha256: receipt.bodySha256,
    vectorSha256: receipt.vectorSha256,
    boundarySha256: digest(JSON.stringify(receipt.boundary)),
    ...(receipt.traversal === "finite-physical-edges"
      ? {
          originalVectorSha256: receipt.originalVectorSha256,
          originalFilesSha256: receipt.originalFilesSha256,
        }
      : {}),
  };
}

export function validateFinitePaths(paths, nodes, children, boundary) {
  assert.deepEqual(
    paths,
    finiteCommittedInventory(nodes, children, boundary).files.map(([path]) => path),
    "Finite native ordered vector changed committed routes",
  );
  return paths;
}

export function qualifyFiniteSources(original, files, nodes, children, boundary) {
  assert(original, "Missing original kernel archive");
  assert.deepEqual(
    original.map(([path, , , blob]) => [path, blob]),
    walkCommittedGraph(nodes, children, boundary).files,
    "Original whole kernel Git bytes changed",
  );
  validateOriginalBytes(original, files, nodes);
  return finiteCommittedInventory(nodes, children, boundary).files;
}

export function qualifyProbeVectors(artifact, identity, expected, parseNativeVector) {
  const vector = readFileSync(join(artifact, "native-vector.nul"));
  const paths = parseNativeVector(vector);
  const originalVector = readFileSync(join(artifact, "original-kernel-vector.nul"));
  const originalPaths = parseNativeVector(originalVector);
  assert.equal(digest(originalVector), identity.nativeWalk.originalVectorSha256);
  assert.equal(originalPaths.length, 47056);
  assert.equal(originalPaths.filter((path) => path.startsWith("jellyfin-vue/")).length, 5576);
  assert.equal(identity.filesSha256, digest(JSON.stringify(expected)));
  assert.equal(paths.length, identity.files);
  assert.deepEqual(
    paths,
    expected.map(([path]) => path),
  );
  assert.equal(digest(vector), identity.nativeWalk.vectorSha256);
  return { vector, paths };
}

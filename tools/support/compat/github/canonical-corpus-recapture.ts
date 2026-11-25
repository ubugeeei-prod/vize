import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { posix } from "node:path";
import {
  finiteCommittedPlan,
  kernelPrefixWitness,
  pathOrder,
  qualifyKernelBoundary,
  resolveLogical,
  walkCommittedGraph,
} from "./canonical-corpus-logical-walk.ts";
import type {
  Boundary,
  CorpusChildren,
  CorpusNodes,
  KernelPrefixWitness,
  NativeObservation,
  PhysicalEntry,
  VueFile,
} from "./canonical-corpus-types.ts";

const parent = (path: string) => (posix.dirname(path) === "." ? "" : posix.dirname(path));
const excluded = (name: string) => ["node_modules", "_git-worktrees"].includes(name);
const isDirectory = (mode: string) => ["40000", "160000"].includes(mode);
const below = (path: string, cut: string[]) => cut.some((prefix) => path.startsWith(`${prefix}/`));
const fileIdentity = (files: VueFile[]) => files.map(([path, , , blob]) => [path, blob]);

function physicalTree(nodes: CorpusNodes, observation: NativeObservation) {
  assert(Array.isArray(observation.physical), "Missing recapture physical observation");
  const entries = new Map<string, PhysicalEntry>();
  const children = new Map<string, string[]>();
  for (const entry of observation.physical) {
    assert(
      entry.path &&
        !entry.path.startsWith("/") &&
        !entry.path.split("/").some((part) => ["", ".", ".."].includes(part)),
      "Foreign recapture physical path",
    );
    assert(!entries.has(entry.path), "Duplicate recapture physical path");
    assert(Number.isSafeInteger(entry.mode), "Invalid recapture physical mode");
    entries.set(entry.path, entry);
    const owner = parent(entry.path);
    const names = children.get(owner) ?? [];
    names.push(posix.basename(entry.path));
    children.set(owner, names);
    const node = nodes.get(entry.path);
    if (!node) {
      assert.notEqual(entry.mode & 0o170000, 0o120000, "Foreign uncommitted physical symlink");
      continue; // Ordinary Git metadata files/directories cannot provide Vue sources.
    }
    const mode = entry.mode & 0o170000;
    if (isDirectory(node.mode))
      assert.equal(mode, 0o040000, "Committed physical directory changed");
    else if (node.mode === "120000") {
      assert.equal(mode, 0o120000, "Committed physical symlink mode changed");
      assert.equal(typeof entry.targetBase64, "string", "Missing physical symlink target");
      const bytes = Buffer.from(entry.targetBase64!, "base64");
      assert.equal(bytes.toString("base64"), entry.targetBase64, "Invalid physical symlink target");
      assert.equal(
        bytes.toString("utf8"),
        node.target,
        "Committed physical symlink target changed",
      );
      assert.equal(
        createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
        node.id,
        "Committed physical symlink blob changed",
      );
    } else {
      assert.equal(mode, 0o100000, "Committed physical file mode changed");
      assert.equal(
        entry.mode & 0o111 ? "100755" : "100644",
        node.mode,
        "Committed physical executable mode changed",
      );
    }
  }
  for (const node of nodes.values()) {
    if (!node.path || node.path.split("/").some(excluded)) continue;
    assert(entries.has(node.path), `Missing committed physical entry: ${node.path}`);
  }
  for (const entry of entries.values()) {
    const owner = parent(entry.path);
    assert(
      !owner || (entries.has(owner) && (entries.get(owner)!.mode & 0o170000) === 0o040000),
      "Missing recapture physical directory",
    );
  }
  for (const names of children.values()) names.sort(pathOrder);
  return { entries, children };
}

// This qualifies a later observation against an independently strict initial
// capture. It retains each original vector and metadata refusal: only aliases
// strictly below an already repeated physical directory edge may differ.
export function qualifyRecapturedObservation(
  nodes: CorpusNodes,
  children: CorpusChildren,
  baseline: { raw: NativeObservation; boundary: Boundary; files: VueFile[] },
  raw: NativeObservation,
  boundary: Boundary,
): { files: Array<[string, string]>; cut: string[] } {
  const original = walkCommittedGraph(nodes, children, baseline.boundary).files;
  assert.deepEqual(fileIdentity(baseline.raw.files), original, "Initial original vector changed");
  const plan = finiteCommittedPlan(nodes, children);
  assert(
    Array.isArray(baseline.raw.metadata) && Array.isArray(raw.metadata),
    "Missing recapture metadata observation",
  );
  assert.deepEqual(
    raw.metadata.filter((row) => !below(row.path, plan.cut)),
    baseline.raw.metadata.filter((row) => !below(row.path, plan.cut)),
    "Changed metadata is outside repeated finite descendants",
  );
  assert.deepEqual(fileIdentity(baseline.files), plan.files, "Finite actual vector changed");
  assert.deepEqual(raw.physical, baseline.raw.physical, "Recapture physical observation changed");
  assert.equal(boundary.code, baseline.boundary.code, "Recapture boundary code changed");
  assert.equal(
    boundary.firstRejected,
    baseline.boundary.firstRejected,
    "Recapture owned prefix boundary changed",
  );
  const physical = physicalTree(nodes, baseline.raw);
  const sources = new Map<string, VueFile>();
  for (const row of baseline.files) {
    assert(
      Array.isArray(row) &&
        row.length === 4 &&
        /^[0-9a-f]{64}$/.test(row[1]) &&
        Number.isSafeInteger(row[2]) &&
        row[2] >= 0 &&
        /^[0-9a-f]{40}$/.test(row[3]),
      "Invalid finite whole source tuple",
    );
    const source = resolveLogical(nodes, row[0]);
    assert.equal(source.id, row[3], "Finite committed source blob changed");
    const entry = physical.entries.get(source.path);
    assert(entry && (entry.mode & 0o170000) === 0o100000, "Missing finite physical source");
    assert.equal(entry.size, row[2], "Physical source size differs from whole bytes");
    const previous = sources.get(source.path);
    if (previous) assert.deepEqual(previous.slice(1), row.slice(1), "Finite source bytes disagree");
    sources.set(source.path, row);
  }
  const baselineRejected = new Set(baseline.boundary.rejected.map((row) => row.path));
  const baselineError = baseline.raw.metadata?.find((row) => row.error)?.error;
  assert(baselineError, "Missing initial native metadata refusal");
  assert.equal(baselineError.code, "ELOOP", "Foreign initial metadata error");
  assert.equal(baselineError.syscall, "stat", "Foreign initial metadata syscall");
  assert(Number.isSafeInteger(baselineError.errno), "Missing initial metadata errno");
  assert.equal(typeof baselineError.path, "string", "Missing initial metadata error path");
  const firstErrorPath = baseline.raw.metadata!.find((row) => row.error)!.path;
  assert(baselineError.path!.endsWith(`/${firstErrorPath}`), "Foreign initial metadata error path");
  const root = baselineError.path!.slice(0, -firstErrorPath.length);

  const replay = (observation: NativeObservation, observedBoundary: Boundary, initial: boolean) => {
    assert(Array.isArray(observation.metadata), "Missing recapture metadata observation");
    assert.deepEqual(
      observation.rejected,
      observedBoundary.rejected,
      "Raw rejection vector changed",
    );
    const metadata = observation.metadata;
    const paths = new Set<string>();
    const rejected: Boundary["rejected"] = [];
    const witnesses: KernelPrefixWitness[] = [];
    const files: VueFile[] = [];
    const seenSources = new Set<string>();
    let cursor = 0;
    let successfulFollows = 0;
    const visit = (logical: string, directory: string) => {
      for (const name of physical.children.get(directory) ?? []) {
        if (excluded(name)) continue;
        const path = logical ? `${logical}/${name}` : name;
        const selected = directory ? `${directory}/${name}` : name;
        const entry = physical.entries.get(selected)!;
        const row = metadata[cursor++];
        assert(row, `Missing recapture metadata: ${path}`);
        assert.equal(row.path, path, "Recapture metadata order or path changed");
        assert(!paths.has(path), "Duplicate recapture metadata path");
        paths.add(path);
        const committed = nodes.has(selected) ? resolveLogical(nodes, path) : undefined;
        const directoryEntry = committed
          ? isDirectory(committed.mode)
          : (entry.mode & 0o170000) === 0o040000;
        let expectedFollowed: number | undefined;
        if (row.error) {
          assert(committed?.allowed, "Foreign recapture kernel refusal");
          assert.equal(row.error.code, "ELOOP", "Foreign recapture metadata error");
          assert.equal(row.error.syscall, "stat", "Foreign recapture metadata syscall");
          assert.equal(row.error.errno, baselineError.errno, "Recapture metadata errno changed");
          assert.equal(row.error.path, `${root}${path}`, "Recapture metadata error path changed");
          assert.equal(row.isDirectory, undefined, "Refused metadata claims a directory result");
          expectedFollowed = committed.followed;
          if (!initial && !baselineRejected.has(path)) {
            assert(
              committed.followed < baseline.boundary.firstRejected && below(path, plan.cut),
              "New metadata refusal is outside repeated finite descendants",
            );
          }
          rejected.push({ path, code: "ELOOP", followed: committed.followed });
          const witness = kernelPrefixWitness(nodes, path, logical);
          if (witness) witnesses.push(witness);
        } else {
          assert.equal(
            row.isDirectory,
            directoryEntry,
            "Recapture directory classification changed",
          );
          if (committed)
            assert(
              committed.followed < observedBoundary.firstRejected,
              "Metadata succeeded beyond witnessed kernel boundary",
            );
          if (directoryEntry && nodes.has(path)) {
            expectedFollowed = committed!.followed;
            successfulFollows = Math.max(successfulFollows, expectedFollowed);
          }
        }
        if (!row.error && directoryEntry) {
          visit(path, committed?.path ?? selected);
        } else if (posix.extname(name) === ".vue") {
          assert(committed && !directoryEntry, "Foreign recapture Vue source");
          const source = sources.get(committed.path);
          assert(source, "Recapture omitted a unique finite source");
          expectedFollowed = committed.followed;
          successfulFollows = Math.max(successfulFollows, expectedFollowed);
          files.push([path, source[1], source[2], source[3]]);
          seenSources.add(committed.path);
        }
        assert.equal(row.followed, expectedFollowed, "Recapture metadata follow count changed");
      }
    };
    visit("", "");
    assert.equal(cursor, metadata.length, "Extra recapture metadata paths");
    assert.deepEqual(
      files,
      observation.files,
      "Original recapture ordered paths or whole bytes changed",
    );
    assert.deepEqual(rejected, observation.rejected, "Recapture metadata refusals changed");
    assert.equal(
      successfulFollows,
      observation.successfulFollows,
      "Recapture successful follows changed",
    );
    assert.deepEqual(
      qualifyKernelBoundary(nodes, rejected, successfulFollows, witnesses),
      observedBoundary,
      "Recapture witnessed boundary changed",
    );
    assert.deepEqual(seenSources, new Set(sources.keys()), "Recapture lost a unique whole source");
    assert.deepEqual(
      files.filter(([path]) => !below(path, plan.cut)),
      baseline.files,
      "Recapture changed ordered finite paths or whole bytes",
    );
  };
  replay(baseline.raw, baseline.boundary, true);
  replay(raw, boundary, false);
  return plan;
}

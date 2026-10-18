import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  readlinkSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { extname, join, relative, resolve, sep } from "node:path";
import { committedGraph, gitObjectId } from "./canonical-corpus-inventory.mjs";
import {
  needsNativeWalk,
  kernelPrefixWitness,
  qualifyKernelBoundary,
  pathOrder,
  resolveLogical,
  validateLogicalGraph,
  walkCommittedGraph,
} from "./canonical-corpus-logical-walk.ts";

import {
  captureFiniteObservation,
  nativeWalkIdentity,
  validateOriginalArchive,
  qualifyFiniteSources,
  validateFinitePaths,
} from "./canonical-corpus-finite-walk.mjs";
export { collectFinitePhysical } from "./canonical-corpus-finite-walk.mjs";

import type {
  Boundary,
  CorpusChildren,
  CorpusNodes,
  VueFile,
  KernelPrefixWitness,
  CorpusProof,
  CorpusIdentity,
  NativeReceipt,
  NativeWalkIdentity,
  NativeObservation,
  PhysicalEntry,
  MetadataEntry,
} from "./canonical-corpus-types.ts";
export type {
  CorpusProof,
  CorpusIdentity,
  NativeReceipt,
  NativeWalkIdentity,
  NativeObservation,
} from "./canonical-corpus-types.ts";

export const collectorSource = "tests/davinci_test_support/src/corpus.rs";
const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const gitSource = (cwd: string, sha: string, path: string) =>
  execFileSync("git", ["show", `${sha}:${path}`], { cwd });
const excluded = (name: string) => ["node_modules", "_git-worktrees"].includes(name);

export function collectorHarness(bytes: Uint8Array): { body: string; harness: Buffer } {
  const source = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  const bodies = source.match(
    /pub fn collect_vue_files\(root: &Path, out: &mut Vec<PathBuf>\) \{[\s\S]*?\n\}\n/g,
  );
  assert.equal(bodies?.length, 1, "Missing unique original Rust collector body");
  const body = bodies[0];
  const names = ["fail_corpus_io", "corpus_entry_path", "collect_vue_edges"];
  const helpers = names.flatMap((name) => {
    const matches = source.match(new RegExp(`^fn ${name}\\([\\s\\S]*?\\n\\}\\n`, "gm")) ?? [];
    assert(matches.length <= 1, `Missing unique original Rust helper ${name}`);
    return matches;
  });
  const program = [...helpers, body].join("\n");
  for (const name of names)
    assert(
      !new RegExp(`\\b${name}\\(`).test(program) ||
        helpers.some((helper) => helper.startsWith(`fn ${name}(`)),
      `Missing unique original Rust helper ${name}`,
    );
  const prefix = helpers.length ? `${helpers.join("\n")}\n` : "";
  const harness = `use std::{fs, io::{self, Write}, path::{Path, PathBuf}};\n${prefix}${body}\nfn main() {
    let root = std::env::args_os().nth(1).expect("corpus root");
    let root = Path::new(&root);
    let mut files = Vec::new();
    collect_vue_files(root, &mut files);
    let mut output = io::stdout().lock();
    for file in files {
        let path = file.strip_prefix(root).unwrap().to_str().expect("UTF8 corpus path");
        output.write_all(path.as_bytes()).unwrap();
        output.write_all(&[0]).unwrap();
    }
}\n`;
  return { body, harness: Buffer.from(harness) };
}

// This shallow walk authenticates symlinks before native traversal; it never
// follows them or removes logical aliases from the actual collector output.
function validatePhysicalLinks(
  root: string,
  nodes: CorpusNodes,
  observed: PhysicalEntry[] | null = null,
) {
  const visit = (directory: string): void => {
    for (const name of readdirSync(directory)) {
      if (excluded(name)) continue;
      const path = join(directory, name);
      const stat = lstatSync(path);
      const entry: PhysicalEntry = {
        path: relative(root, path).split(sep).join("/"),
        mode: stat.mode,
        size: stat.size,
        mtimeMs: stat.mtimeMs,
        ctimeMs: stat.ctimeMs,
      };
      observed?.push(entry);
      if (stat.isDirectory()) {
        if (!excluded(name)) visit(path);
      } else if (stat.isSymbolicLink()) {
        const logical = relative(root, path).split(sep).join("/");
        const node = nodes.get(logical);
        const bytes = readlinkSync(path, { encoding: "buffer" });
        entry.targetBase64 = bytes.toString("base64");
        assert(node?.mode === "120000", `Foreign physical canonical symlink: ${logical}`);
        assert.equal(gitObjectId("blob", bytes), node.id, "Physical symlink bytes changed");
        assert.equal(bytes.toString("utf8"), node.target, "Physical symlink target changed");
      }
    }
  };
  visit(root);
}

export function collectNativePhysical(
  root: string,
  nodes: CorpusNodes,
  children: CorpusChildren,
  observe: ((raw: NativeObservation) => void) | null = null,
): { files: VueFile[]; boundary: Boundary; committed: Array<[string, string]> } {
  validateLogicalGraph(nodes, children);
  const physical: PhysicalEntry[] | null = observe ? [] : null;
  const metadata: MetadataEntry[] | null = observe ? [] : null;
  const files: VueFile[] = [];
  const rejected: Boundary["rejected"] = [];
  const witnesses: KernelPrefixWitness[] = [];
  let successfulFollows = 0;
  // Retain raw logical metadata calls for the witnessed original kernel boundary.
  // Directory enumeration and whole-byte reads use the already shortened parent,
  // so those syscalls cannot exhaust a different lookup budget on the same alias.
  const visit = (directory: string, physicalDirectory: string): void => {
    for (const name of readdirSync(physicalDirectory).sort(pathOrder)) {
      if (excluded(name)) continue;
      const path = join(directory, name);
      const physicalPath = join(physicalDirectory, name);
      const logical = relative(root, path).split(sep).join("/");
      let isDirectory;
      const observed: MetadataEntry = { path: logical };
      metadata?.push(observed);
      try {
        isDirectory = statSync(path).isDirectory();
        observed.isDirectory = isDirectory;
      } catch (error) {
        observed.error = {
          code: (error as NodeJS.ErrnoException).code,
          errno: (error as NodeJS.ErrnoException).errno,
          syscall: (error as NodeJS.ErrnoException).syscall,
          path: (error as NodeJS.ErrnoException).path,
          message: (error as NodeJS.ErrnoException).message,
        };
        assert.equal(
          (error as NodeJS.ErrnoException).code,
          "ELOOP",
          "Unexpected physical corpus metadata failure",
        );
        const node = resolveLogical(nodes, logical);
        assert(node.allowed, "Foreign physical kernel boundary");
        rejected.push({
          path: logical,
          code: (error as NodeJS.ErrnoException).code!,
          followed: node.followed,
        });
        const witness = kernelPrefixWitness(
          nodes,
          logical,
          relative(root, directory).split(sep).join("/"),
        );
        if (witness) witnesses.push(witness);
        observed.followed = node.followed;
        isDirectory = false;
      }
      if (isDirectory) {
        // Untracked Git metadata has no committed source node and no Vue inputs.
        if (nodes.has(logical)) {
          observed.followed = resolveLogical(nodes, logical).followed;
          successfulFollows = Math.max(successfulFollows, observed.followed);
        }
        if (!excluded(name)) visit(path, realpathSync(physicalPath));
      } else if (extname(name) === ".vue") {
        const bytes = readFileSync(physicalPath); // Any unreadable Vue remains fatal.
        const node = resolveLogical(nodes, logical);
        observed.followed = node.followed;
        successfulFollows = Math.max(successfulFollows, node.followed);
        files.push([logical, digest(bytes), bytes.length, gitObjectId("blob", bytes)]);
      }
    }
  };
  // Persist the unmodified observations even when traversal or the strict boundary
  // comparisons refuse this capture. These rows never authorize a boundary.
  let traversalFailure: { error: unknown } | null = null;
  try {
    validatePhysicalLinks(root, nodes, physical);
    visit(root, realpathSync(root));
  } catch (error) {
    traversalFailure = { error };
  }
  try {
    observe?.({ physical, metadata, files, rejected, successfulFollows });
  } catch (observationError) {
    if (traversalFailure)
      throw new AggregateError(
        [traversalFailure.error, observationError],
        "Native traversal and observation persistence both failed",
      );
    throw observationError;
  }
  if (traversalFailure) throw traversalFailure.error;
  const boundary = qualifyKernelBoundary(nodes, rejected, successfulFollows, witnesses);
  const committed = walkCommittedGraph(nodes, children, boundary).files;
  assert.deepEqual(
    files.map(([path, , , blob]) => [path, blob]),
    committed,
    "Physical inventory differs from complete committed Vue bytes",
  );
  return { files, boundary, committed };
}

export function validateFiniteNativeVector(
  vector: Uint8Array,
  nodes: CorpusNodes,
  children: CorpusChildren,
  boundary: Boundary,
): string[] {
  return validateFinitePaths(parseNativeVector(vector), nodes, children, boundary);
}

export function captureNativeWalk(
  cwd: string,
  identity: CorpusIdentity,
  proof: CorpusProof,
  directory: string,
): { files: VueFile[]; nativeWalk: NativeWalkIdentity } {
  const { nodes, children } = committedGraph(proof, identity.gitlinks);
  validateLogicalGraph(nodes, children);
  const root = resolve(cwd, "tests/_fixtures/_git");
  validatePhysicalLinks(root, nodes);
  const source = gitSource(cwd, identity.sha, collectorSource);
  assert.deepEqual(
    readFileSync(join(cwd, collectorSource)),
    source,
    "Original collector source changed",
  );
  const { body, harness } = collectorHarness(source);
  mkdirSync(directory, { recursive: true });
  writeFileSync(join(directory, "native-collector-source.rs"), source);
  writeFileSync(join(directory, "native-collector.rs"), harness);
  const toolchain = execFileSync("rustc", ["-vV"], { cwd });
  execFileSync(
    "rustc",
    [
      "--edition=2024",
      "--crate-name",
      "canonical_original_collector",
      "-C",
      "debuginfo=0",
      "--remap-path-prefix",
      `${directory}=canonical-observer`,
      join(directory, "native-collector.rs"),
      "-o",
      join(directory, "native-collector"),
    ],
    { cwd },
  );
  const vector = execFileSync(join(directory, "native-collector"), [root], {
    cwd,
    maxBuffer: 64 * 1024 * 1024,
  });
  // Each recapture has a distinct evidence directory: a later refusal must not
  // overwrite the initial successful vector/graph or require a valid receipt.
  const observation = mkdtempSync(join(directory, "native-observation-"));
  for (const [name, bytes] of [
    ["identity.json", JSON.stringify(identity)],
    ["committed-corpus.json", JSON.stringify(proof)],
    ["native-vector.nul", vector],
    ["native-collector-source.rs", source],
    ["native-collector.rs", harness],
    ["native-rustc.txt", toolchain],
    ["native-collector", readFileSync(join(directory, "native-collector"))],
  ] satisfies Array<[string, string | Uint8Array]>)
    writeFileSync(join(observation, name), bytes, { flag: "wx" });
  const original = collectNativePhysical(root, nodes, children, (raw) => {
    writeFileSync(join(observation, "physical.json"), JSON.stringify(raw), { flag: "wx" });
  });
  proof.kernelBoundary = original.boundary;
  proof.traversal = "finite-physical-edges";
  const {
    files,
    committed,
    originalVector,
  }: {
    files: VueFile[];
    committed: Array<[string, string]>;
    originalVector: Buffer;
  } = captureFiniteObservation(root, nodes, children, original, directory);
  const nativePaths = validateFiniteNativeVector(vector, nodes, children, original.boundary);
  assert.deepEqual(
    nativePaths,
    files.map(([path]) => path),
    "Physical inventory differs from complete ordered successor Rust vector",
  );
  assert.deepEqual(
    files.map(([path, , , blob]) => [path, blob]),
    committed,
    "Git inventory differs from complete ordered original Rust vector/bytes",
  );
  writeFileSync(join(directory, "native-vector.nul"), vector);
  writeFileSync(join(directory, "native-rustc.txt"), toolchain);
  const receipt: NativeReceipt = {
    schema: "vize.canonical-native-walk",
    version: 1,
    sha: identity.sha,
    tree: identity.tree,
    runId: identity.runId,
    attempt: identity.attempt,
    platform: process.platform,
    architecture: process.arch,
    sourceSha256: digest(source),
    bodySha256: digest(body),
    harnessSha256: digest(harness),
    binarySha256: digest(readFileSync(join(directory, "native-collector"))),
    toolchainSha256: digest(toolchain),
    vectorSha256: digest(vector),
    files: nativePaths.length,
    boundary: proof.kernelBoundary,
    traversal: proof.traversal,
    originalVectorSha256: digest(originalVector),
    originalFilesSha256: digest(JSON.stringify(original.files)),
  };
  writeFileSync(join(directory, "native-walk.json"), JSON.stringify(receipt) + "\n");
  validateNativeWalk(directory, identity, proof, files, cwd);
  return {
    files,
    nativeWalk: nativeWalkIdentity(receipt),
  };
}

export function parseNativeVector(vector: Uint8Array): string[] {
  assert(vector.length > 0 && vector.at(-1) === 0, "Incomplete original Rust path vector");
  const paths = new TextDecoder("utf-8", { fatal: true }).decode(vector).slice(0, -1).split("\0");
  assert(
    paths.every(
      (path) =>
        path &&
        !path.startsWith("/") &&
        !path.split("/").some((part) => ["", ".", ".."].includes(part)),
    ),
    "Foreign native corpus path",
  );
  assert.equal(new Set(paths).size, paths.length, "Duplicate original Rust logical path");
  return paths;
}

export function validateNativeSource(
  directory: string,
  identity: CorpusIdentity,
  cwd = process.cwd(),
): { receipt: NativeReceipt; paths: string[]; original?: VueFile[] } {
  const receipt: NativeReceipt = JSON.parse(
    readFileSync(join(directory, "native-walk.json"), "utf8"),
  );
  assert.equal(receipt.schema, "vize.canonical-native-walk", "Foreign native walk receipt");
  assert.equal(receipt.version, 1, "Unknown native walk receipt version");
  for (const key of ["sha", "tree", "runId", "attempt"] as const)
    assert.equal(receipt[key], identity[key], `Native walk ${key} changed`);
  const source = gitSource(cwd, identity.sha, collectorSource);
  const { body, harness } = collectorHarness(source);
  assert.equal(
    receipt.traversal,
    /\bcollect_vue_edges\(/.test(body) ? "finite-physical-edges" : undefined,
    "Native traversal does not match exact Rust source",
  );
  assert.deepEqual(
    readFileSync(join(directory, "native-collector-source.rs")),
    source,
    "Archived original Rust source changed",
  );
  assert.deepEqual(
    readFileSync(join(directory, "native-collector.rs")),
    harness,
    "Archived original Rust body/harness changed",
  );
  assert.equal(receipt.sourceSha256, digest(source), "Native collector source identity changed");
  assert.equal(receipt.bodySha256, digest(body), "Original Rust collector body changed");
  assert.equal(receipt.harnessSha256, digest(harness), "Native collector harness changed");
  for (const [name, key] of [
    ["native-collector", "binarySha256"],
    ["native-rustc.txt", "toolchainSha256"],
    ["native-vector.nul", "vectorSha256"],
  ] as const)
    assert.equal(
      digest(readFileSync(join(directory, name))),
      receipt[key],
      `Native ${name} bytes changed`,
    );
  const channel = /^channel\s*=\s*"([^"]+)"/m.exec(
    gitSource(cwd, identity.sha, "rust-toolchain.toml").toString("utf8"),
  )?.[1];
  assert(channel && /^\d+\.\d+\.\d+$/.test(channel), "Unpinned native collector toolchain");
  assert(
    readFileSync(join(directory, "native-rustc.txt"), "utf8").startsWith(`rustc ${channel} `),
    "Foreign native collector Rust toolchain",
  );
  const paths = parseNativeVector(readFileSync(join(directory, "native-vector.nul")));
  assert.equal(receipt.files, paths.length, "Native vector census changed");
  const original: VueFile[] | undefined =
    receipt.traversal === "finite-physical-edges"
      ? validateOriginalArchive(directory, receipt, parseNativeVector)
      : undefined;
  return { receipt, paths, ...(original ? { original } : {}) };
}

export function validateNativeWalk(
  directory: string,
  identity: CorpusIdentity,
  proof: CorpusProof,
  files: VueFile[],
  cwd = process.cwd(),
): void {
  if (!needsNativeWalk(identity.gitlinks) && identity.nativeWalk === undefined) return;
  const { receipt, paths, original } = validateNativeSource(directory, identity, cwd);
  assert.deepEqual(
    receipt.boundary,
    proof.kernelBoundary,
    "Native kernel boundary receipt changed",
  );
  assert.deepEqual(
    paths,
    files.map(([path]) => path),
    "Native ordered path vector changed",
  );
  const { nodes, children } = committedGraph(proof, identity.gitlinks);
  assert.equal(receipt.traversal, proof.traversal, "Native traversal identity changed");
  let committed = walkCommittedGraph(nodes, children, receipt.boundary).files;
  if (proof.traversal === "finite-physical-edges") {
    committed = qualifyFiniteSources(original, files, nodes, children, receipt.boundary);
  } else assert(proof.traversal === undefined, "Unknown native corpus traversal");
  assert.deepEqual(
    files.map(([path, , , blob]) => [path, blob]),
    committed,
    "Native whole-vector Git bytes changed",
  );
  if (identity.nativeWalk !== undefined)
    assert.deepEqual(
      identity.nativeWalk,
      nativeWalkIdentity(receipt),
      "Native walk identity changed",
    );
}

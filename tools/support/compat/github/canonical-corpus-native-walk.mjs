import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { extname, join, relative, resolve, sep } from "node:path";
import { committedGraph, gitObjectId } from "./canonical-corpus-inventory.mjs";
import {
  needsNativeWalk,
  pathOrder,
  resolveLogical,
  validateLogicalGraph,
  walkCommittedGraph,
} from "./canonical-corpus-logical-walk.mjs";

export const collectorSource = "tests/davinci_test_support/src/corpus.rs";
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const gitSource = (cwd, sha, path) => execFileSync("git", ["show", `${sha}:${path}`], { cwd });
const excluded = (name) => ["node_modules", "_git-worktrees"].includes(name);

export function collectorHarness(bytes) {
  const source = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  const bodies = source.match(
    /pub fn collect_vue_files\(root: &Path, out: &mut Vec<PathBuf>\) \{[\s\S]*?\n\}\n/g,
  );
  assert.equal(bodies?.length, 1, "Missing unique original Rust collector body");
  const body = bodies[0];
  const names = ["fail_corpus_io", "corpus_entry_path"];
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
function validatePhysicalLinks(root, nodes) {
  const visit = (directory) => {
    for (const name of readdirSync(directory)) {
      if (excluded(name)) continue;
      const path = join(directory, name);
      const stat = lstatSync(path);
      if (stat.isDirectory()) {
        if (!excluded(name)) visit(path);
      } else if (stat.isSymbolicLink()) {
        const logical = relative(root, path).split(sep).join("/");
        const node = nodes.get(logical);
        const bytes = readlinkSync(path, { encoding: "buffer" });
        assert(node?.mode === "120000", `Foreign physical canonical symlink: ${logical}`);
        assert.equal(gitObjectId("blob", bytes), node.id, "Physical symlink bytes changed");
        assert.equal(bytes.toString("utf8"), node.target, "Physical symlink target changed");
      }
    }
  };
  visit(root);
}

export function collectNativePhysical(root, nodes, children) {
  validateLogicalGraph(nodes, children);
  validatePhysicalLinks(root, nodes);
  const files = [];
  const rejected = [];
  let successfulFollows = 0;
  const visit = (directory) => {
    for (const name of readdirSync(directory).sort(pathOrder)) {
      if (excluded(name)) continue;
      const path = join(directory, name);
      const logical = relative(root, path).split(sep).join("/");
      let isDirectory;
      try {
        isDirectory = statSync(path).isDirectory();
      } catch (error) {
        assert.equal(error.code, "ELOOP", "Unexpected physical corpus metadata failure");
        const node = resolveLogical(nodes, logical);
        assert(node.allowed, "Foreign physical kernel boundary");
        rejected.push({ path: logical, code: error.code, followed: node.followed });
        isDirectory = false;
      }
      if (isDirectory) {
        // Untracked Git metadata has no committed source node and no Vue inputs.
        if (nodes.has(logical))
          successfulFollows = Math.max(successfulFollows, resolveLogical(nodes, logical).followed);
        if (!excluded(name)) visit(path);
      } else if (extname(name) === ".vue") {
        const bytes = readFileSync(path); // Any unreadable Vue remains fatal.
        const node = resolveLogical(nodes, logical);
        successfulFollows = Math.max(successfulFollows, node.followed);
        files.push([logical, digest(bytes), bytes.length, gitObjectId("blob", bytes)]);
      }
    }
  };
  visit(root);
  assert(rejected.length > 0, "Missing actual native kernel boundary observation");
  const firstRejected = rejected[0].followed;
  assert(
    rejected.every((row) => row.followed === firstRejected),
    "Inconsistent native kernel boundary",
  );
  assert.equal(
    successfulFollows + 1,
    firstRejected,
    "Native kernel boundary lacks preceding success",
  );
  const boundary = { code: "ELOOP", firstRejected, rejected };
  const committed = walkCommittedGraph(nodes, children, boundary).files;
  assert.deepEqual(
    files.map(([path, , , blob]) => [path, blob]),
    committed,
    "Physical inventory differs from complete committed Vue bytes",
  );
  return { files, boundary, committed };
}

export function captureNativeWalk(cwd, identity, proof, directory) {
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
  const physical = collectNativePhysical(root, nodes, children);
  proof.kernelBoundary = physical.boundary;
  const committed = physical.committed;
  const nativePaths = parseNativeVector(vector);
  assert.deepEqual(
    nativePaths,
    physical.files.map(([path]) => path),
    "Physical inventory differs from complete ordered original Rust vector",
  );
  assert.deepEqual(
    physical.files.map(([path, , , blob]) => [path, blob]),
    committed,
    "Git inventory differs from complete ordered original Rust vector/bytes",
  );
  writeFileSync(join(directory, "native-vector.nul"), vector);
  writeFileSync(join(directory, "native-rustc.txt"), toolchain);
  const receipt = {
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
  };
  writeFileSync(join(directory, "native-walk.json"), JSON.stringify(receipt) + "\n");
  validateNativeWalk(directory, identity, proof, physical.files, cwd);
  return {
    files: physical.files,
    nativeWalk: {
      sourceSha256: receipt.sourceSha256,
      bodySha256: receipt.bodySha256,
      vectorSha256: receipt.vectorSha256,
      boundarySha256: digest(JSON.stringify(receipt.boundary)),
    },
  };
}

export function parseNativeVector(vector) {
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

export function validateNativeSource(directory, identity, cwd = process.cwd()) {
  const receipt = JSON.parse(readFileSync(join(directory, "native-walk.json"), "utf8"));
  assert.equal(receipt.schema, "vize.canonical-native-walk", "Foreign native walk receipt");
  assert.equal(receipt.version, 1, "Unknown native walk receipt version");
  for (const key of ["sha", "tree", "runId", "attempt"])
    assert.equal(receipt[key], identity[key], `Native walk ${key} changed`);
  const source = gitSource(cwd, identity.sha, collectorSource);
  const { body, harness } = collectorHarness(source);
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
  ])
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
  return { receipt, paths };
}

export function validateNativeWalk(directory, identity, proof, files, cwd = process.cwd()) {
  if (!needsNativeWalk(identity.gitlinks) && identity.nativeWalk === undefined) return;
  const { receipt, paths } = validateNativeSource(directory, identity, cwd);
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
  assert.deepEqual(
    files.map(([path, , , blob]) => [path, blob]),
    walkCommittedGraph(nodes, children, receipt.boundary).files,
    "Native whole-vector Git bytes changed",
  );
  if (identity.nativeWalk !== undefined)
    assert.deepEqual(
      identity.nativeWalk,
      {
        sourceSha256: receipt.sourceSha256,
        bodySha256: receipt.bodySha256,
        vectorSha256: receipt.vectorSha256,
        boundarySha256: digest(JSON.stringify(receipt.boundary)),
      },
      "Native walk identity changed",
    );
}

import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { ensureHostedLiteralSource } from "./warm-type-backed-git-reader-literal-source.ts";
import { gitReaderSetupPairs, rawFirstParentProof } from "./warm-type-backed-git-reader-setup.ts";

type Cell = { mode: string; blob: string; bytes: number; sha256: string };
type Row = {
  file: string;
  originalBeforeSHA256: string | null;
  originalAfterSHA256: string | null;
  before: Cell | null;
  retainedAfter: Cell | null;
  currentAfter: Cell | null;
};
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const digestObject = (kind: string, body: Buffer) =>
  createHash("sha1").update(`${kind} ${body.length}\0`).update(body).digest("hex");
const dataName = "./warm-type-backed-git-reader-owned-frame.json";
function metadata(file: string) {
  const url = new URL(file, import.meta.url);
  const stat = fs.lstatSync(url);
  assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
  return fs.readFileSync(url);
}
const dataBytes = metadata(dataName);
assert.equal(sha(dataBytes), "5a264e4d3e0b5fe97f09592966109d67c2e6ac1a38b9d1f8458f968b7c82963c");
const data = JSON.parse(dataBytes.toString());
assert.equal(data.version, 1);
assert.equal(data.originalSource, "eafe5bed4f602e0787ff9f2f48ec4155d115ab7f");
assert.equal(data.retainedSource, "2489128d5f8fab1b7442a1e5c09b4e1701cbf185");
assert.equal(data.currentContextSource, "4be309ffd3739e90c0b2d7af0b96bec6df9a0cc4");
assert.equal(sha(JSON.stringify(gitReaderSetupPairs)), data.originalSetupPairsSHA256);
function parts(entries: Array<{ file: string; entries: number; sha256: string }>, stem: string) {
  assert.deepEqual(
    entries.map(({ file }) => file),
    Array.from({ length: 10 }, (_, i) => `${stem}-${String(i).padStart(2, "0")}.json`),
  );
  return entries.flatMap(({ file, entries, sha256 }) => {
    const bytes = metadata(file);
    assert.equal(sha(bytes), sha256);
    const values = JSON.parse(bytes.toString());
    assert.equal(values.length, entries);
    return values;
  });
}
const current: Row[] = parts(data.currentBodyParts, "warm-type-backed-git-reader-owned-current");
const retainedStem = "warm-type-backed-git-reader-owned-retained";
const retained: Array<Cell & { file: string }> = parts(data.retainedBodyParts, retainedStem);
assert.equal(current.length, 111);
assert.equal(new Set(current.map(({ file }) => file)).size, 111);
assert.equal(retained.length, 113);
assert.equal(new Set(retained.map(({ file }) => file)).size, 113);
assert.deepEqual(
  current
    .slice(0, 110)
    .map(({ file, originalBeforeSHA256, originalAfterSHA256 }) => [
      file,
      originalBeforeSHA256,
      originalAfterSHA256,
    ]),
  gitReaderSetupPairs,
);
assert.deepEqual(
  data.exactCurrentAfterDeltas.map(({ file }: { file: string }) => file),
  [
    "tests/tooling/support/ascii-symbol-original-callers.ts",
    "tests/tooling/ascii-symbol-original-callers.test.ts",
  ],
);
const git = (root: string, args: string[]) => {
  if (args[0] === "cat-file" && /^(?:commit|-t)$/u.test(args[1]))
    ensureHostedLiteralSource(root, args[2]);
  return execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    maxBuffer: 64 * 1024 * 1024,
  });
};
function existingFirstParents(root: string, ancestor: string, head: string) {
  for (let source = head, count = 0; count < 64; count++) {
    const body = git(root, ["cat-file", "commit", source]);
    if (source === ancestor) return rawFirstParentProof(root, ancestor, head);
    const parent = /^parent ([a-f0-9]{40})$/mu.exec(
      body.subarray(0, body.indexOf("\n\n")).toString(),
    );
    assert(parent, "literal existing first-parent object required");
    source = parent[1];
  }
  assert.fail("raw first-parent ancestry exceeds 64 commits");
}
function privateCommit(root: string, tree: string, parent: string, message: string) {
  return git(root, ["commit-tree", tree, "-p", parent, "-m", message]).toString().trim();
}
function raw(root: string, source: string, file: string) {
  assert.match(source, /^[a-f0-9]{40}$/u);
  const entry = git(root, ["ls-tree", "-z", source, "--", file]);
  if (!entry.length) return null;
  const match = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(entry.toString());
  assert(match && match[2] === file);
  return { mode: "100644", blob: match[1], body: git(root, ["cat-file", "blob", match[1]]) };
}
function equal(value: ReturnType<typeof raw>, cell: Cell | null) {
  if (!cell) return value === null;
  return (
    !!value &&
    value.mode === cell.mode &&
    value.blob === cell.blob &&
    value.body.length === cell.bytes &&
    sha(value.body) === cell.sha256 &&
    digestObject("blob", value.body) === cell.blob
  );
}
function driver(body: Buffer) {
  const law = data.lawDriver;
  const old = Buffer.from(law.rawBase64, "base64");
  assert.equal(old.toString("base64"), law.rawBase64);
  assert.equal(old.length, law.bytes);
  assert.equal(sha(old), law.sha256);
  assert.equal(digestObject("blob", old), law.blob);
  assert.equal(sha(body), law.afterSHA256);
  let transformed = old.toString();
  for (const [before, after] of law.forwardClauses) {
    assert.equal(transformed.split(before).length, 2);
    transformed = transformed.replace(before, after);
  }
  assert.deepEqual(Buffer.from(transformed), body, "whole original NoDeep law transport inverse");
}

export function qualifyOwnedReaderFrame(root: string) {
  const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  assert.equal(git(root, ["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  assert.match(head, /^[a-f0-9]{40}$/u);
  const currentCommit = git(root, ["cat-file", "commit", head]);
  assert.equal(digestObject("commit", currentCommit), head);
  assert.match(currentCommit.toString(), /^tree [a-f0-9]{40}\n/u);
  for (const [index, node] of data.rawIntegrationPath.entries()) {
    const body = git(root, ["cat-file", "commit", node.source]);
    assert.equal(body.length, node.bytes);
    assert.equal(sha(body), node.sha256);
    assert.equal(digestObject("commit", body), node.source);
    const next = data.rawIntegrationPath[index + 1];
    if (next)
      assert(
        body
          .subarray(0, body.indexOf("\n\n"))
          .toString()
          .split("\n")
          .includes("parent " + next.source),
      );
  }
  assert.equal(data.rawIntegrationPath[0].source, data.currentContextSource);
  assert.equal(data.rawIntegrationPath.at(-1).source, data.retainedSource);
  existingFirstParents(root, data.originalSource, data.retainedSource);
  for (const row of current) {
    assert(
      equal(raw(root, data.originalSource, row.file), row.before),
      "complete original before body: " + row.file,
    );
    const after = raw(root, head, row.file);
    assert(equal(after, row.currentAfter), "complete current body: " + row.file);
    const physical = path.join(root, row.file);
    const stat = fs.lstatSync(physical);
    assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
    assert.deepEqual(fs.readFileSync(physical), after!.body);
    if (row.file === data.lawDriver.file) driver(after!.body);
  }
  return {
    authority: "complete-owned-reader-current-source-frame",
    head,
    completeCurrentPairs: 111,
    preservedOriginalPairs: 110,
    originalNoDeepPairs: 102,
    originalNamedFamilyBodies: 87,
    acceptanceCredit: "none",
  };
}

export function runRetainedReaderLaw(root: string) {
  for (const source of [
    data.retainedSource,
    data.originalSource,
    "3dda8f9d71fbd7e701f0805187daff6191865190",
  ])
    assert.equal(git(root, ["cat-file", "-t", source]).toString(), "commit\n");
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-retained-reader-source-"));
  const bodies = retained.map((row) => {
    const value = raw(root, data.retainedSource, row.file);
    assert(equal(value, row));
    return { ...row, body: value!.body };
  });
  try {
    for (const row of bodies) {
      const target = path.join(fixture, row.file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, row.body, { flag: "wx", mode: 0o644 });
      assert.deepEqual(fs.readFileSync(target), row.body);
    }
    git(fixture, ["init", "--quiet"]);
    const common = git(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(fixture, ".git/objects/info/alternates"),
      path.resolve(root, common, "objects") + "\n",
    );
    git(fixture, ["update-ref", "refs/heads/retained", data.retainedSource]);
    git(fixture, ["symbolic-ref", "HEAD", "refs/heads/retained"]);
    const env = { ...process.env };
    for (const key of Object.keys(env))
      if (key.startsWith("GIT_") || key === "NODE_TEST_CONTEXT") delete env[key];
    const result = spawnSync(
      process.execPath,
      ["--test", "--test-reporter=tap", data.lawDriver.file],
      {
        cwd: fixture,
        env,
        encoding: "utf8",
        maxBuffer: 64 * 1024 * 1024,
        timeout: 60_000,
      },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    for (const line of [
      "# tests 5",
      "# pass 5",
      "# fail 0",
      "# cancelled 0",
      "# skipped 0",
      "# todo 0",
    ])
      assert(result.stdout.split("\n").includes(line), result.stdout);
    return {
      source: data.retainedSource,
      completeRetainedBodies: 113,
      passed: 5,
      skipped: 0,
      stdout: result.stdout,
      stderr: result.stderr,
      runtimeCredit: false,
      performanceCredit: false,
    };
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
}

/** Every refusal uses complete real objects or private physical files, never an injected digest. */
export function exerciseOwnedReaderFrameRefusals(root: string) {
  let head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-owned-reader-refusals-"));
  const results: Array<{ name: string; message: string }> = [];
  try {
    for (const row of current) {
      const value = raw(root, head, row.file);
      assert(equal(value, row.currentAfter));
      const target = path.join(fixture, row.file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, value!.body, { flag: "wx", mode: 0o644 });
    }
    git(fixture, ["init", "--quiet"]);
    git(fixture, ["config", "user.name", "Vize source control"]);
    git(fixture, ["config", "user.email", "source-control@example.invalid"]);
    const common = git(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(fixture, ".git/objects/info/alternates"),
      path.resolve(root, common, "objects") + "\n",
    );
    git(fixture, ["update-ref", "refs/heads/current", head]);
    git(fixture, ["symbolic-ref", "HEAD", "refs/heads/current"]);
    git(fixture, ["read-tree", head]);
    const equivalentTree = git(fixture, ["write-tree"]).toString().trim();
    head = privateCommit(
      fixture,
      equivalentTree,
      data.retainedSource,
      "squashed-equivalent source control",
    );
    git(fixture, ["update-ref", "refs/heads/current", head]);
    const squashed = git(fixture, ["cat-file", "commit", head]).toString();
    assert.deepEqual(
      squashed
        .split("\n\n")[0]
        .split("\n")
        .filter((line) => line.startsWith("parent ")),
      ["parent " + data.retainedSource],
    );
    assert.equal(qualifyOwnedReaderFrame(fixture).completeCurrentPairs, 111);
    const refused = (name: string) => {
      assert.throws(
        () => qualifyOwnedReaderFrame(fixture),
        (error: unknown) => {
          assert(error instanceof Error);
          results.push({ name, message: error.message });
          return true;
        },
      );
    };
    const row = current.find(({ file }) => file === data.exactCurrentAfterDeltas[0].file)!;
    const target = path.join(fixture, row.file);
    const body = fs.readFileSync(target);
    fs.writeFileSync(target, Buffer.concat([body, Buffer.from("\nforeign source\n")]));
    refused("physical-body");
    fs.writeFileSync(target, body);
    fs.chmodSync(target, 0o755);
    refused("physical-mode");
    fs.chmodSync(target, 0o644);
    fs.unlinkSync(target);
    refused("physical-missing");
    fs.symlinkSync(path.join(root, row.file), target);
    refused("physical-symlink");
    fs.unlinkSync(target);
    fs.writeFileSync(target, body, { flag: "wx", mode: 0o644 });
    const old = raw(root, data.retainedSource, row.file);
    assert(equal(old, row.retainedAfter));
    const treeCase = (name: string, mode: string, blob: string) => {
      git(fixture, ["read-tree", head]);
      git(fixture, ["update-index", "--cacheinfo", `${mode},${blob},${row.file}`]);
      const tree = git(fixture, ["write-tree"]).toString().trim();
      const commit = privateCommit(fixture, tree, head, name);
      git(fixture, ["update-ref", "refs/heads/current", commit]);
      refused(name);
      git(fixture, ["update-ref", "refs/heads/current", head]);
    };
    treeCase("raw-mixed-old-ascii", "100644", old!.blob);
    treeCase("raw-executable-mode", "100755", row.currentAfter!.blob);
    git(fixture, ["read-tree", head]);
    git(fixture, ["update-index", "--force-remove", data.additionalCurrentFrame]);
    const tree = git(fixture, ["write-tree"]).toString().trim();
    const commit = privateCommit(fixture, tree, head, "raw-missing-frame");
    git(fixture, ["update-ref", "refs/heads/current", commit]);
    refused("raw-missing-frame");
    git(fixture, ["update-ref", "refs/heads/current", head]);
    git(fixture, ["update-ref", "refs/replace/" + data.originalSource, head]);
    refused("replacement-object");
    assert.equal(results.length, 8);
    return {
      completeCurrentPairs: 111,
      squashedEquivalentPairs: 111,
      squashedParent: data.retainedSource,
      squashedCommit: head,
      controls: results,
      runtimeCredit: false,
    };
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
}

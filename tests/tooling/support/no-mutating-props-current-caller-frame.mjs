import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import {
  privateNoMutatingCallerGit,
  copyNoMutatingCallerShallowBoundary,
} from "./no-mutating-props-current-caller-git.mjs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const compareNames = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
const rawGit = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    input,
    maxBuffer: 16 * 1024 * 1024,
  });
const safePath = (name) => {
  assert(name && !path.isAbsolute(name) && path.normalize(name) === name);
  assert(!name.split("/").includes(".."), name);
};
function treeRows(body) {
  const rows = new Map();
  for (const row of body.toString("utf8").split("\0")) {
    if (!row) continue;
    const match = /^([0-9]{6}) (blob|tree|commit) ([a-f0-9]{40})\t(.+)$/u.exec(row);
    assert(match, "complete raw tree entry");
    assert(!rows.has(match[4]), match[4]);
    rows.set(match[4], { mode: match[1], kind: match[2], blob: match[3] });
  }
  assert.equal(Buffer.from(body.toString("utf8")).equals(body), true);
  return rows;
}

export function rawNoMutatingCallerDigest(root, source, file) {
  assert.match(source, /^[a-f0-9]{40}$/u);
  safePath(file);
  const rows = treeRows(rawGit(root, ["ls-tree", "-z", source, "--", file]));
  if (rows.size === 0) return null;
  assert.equal(rows.size, 1);
  const row = rows.get(file);
  assert(row && row.mode === "100644" && row.kind === "blob", file);
  const body = rawGit(root, ["cat-file", "blob", row.blob]);
  assert.equal(
    createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
    row.blob,
  );
  return sha(body);
}

/** Whole source rows are joined to a literal raw commit, not a synthetic missing-key view. */
export function authenticateNoMutatingCallerFrame(root, manifest) {
  const indexPath =
    "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody/current-callers/successor.json";
  const index = fs.readFileSync(path.join(root, indexPath));
  assert.equal(sha(index), "90d02914213ad8346bdd4a71488dd322591d0ab7dcb9588117f5ccba5caff1b9");
  assert.deepEqual(
    manifest,
    JSON.parse(index),
    "only the complete authenticated source declaration",
  );
  for (const name of [indexPath, manifest.historicalFrame.path]) {
    const status = fs.lstatSync(path.join(root, name));
    assert(status.isFile() && !status.isSymbolicLink() && !(status.mode & 0o111));
    assert.equal(fs.realpathSync(path.join(root, name)), path.join(fs.realpathSync(root), name));
  }
  const bytes = fs.readFileSync(path.join(root, manifest.historicalFrame.path));
  assert.equal(sha(bytes), manifest.historicalFrame.sha256);
  const frame = JSON.parse(gunzipSync(bytes));
  assert.equal(frame.schema, "vize.no-mutating.current-callers.historical-source-frame");
  assert.equal(frame.version, 1);
  assert.equal(frame.source, manifest.source);
  assert.deepEqual(frame.credit, {
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
  });
  assert.deepEqual(frame.completeSubtreeScopes, manifest.completeSubtreeScopes);
  const literalGit = privateNoMutatingCallerGit(root, manifest.source);
  const commit = rawGit(literalGit, ["cat-file", "commit", manifest.source]);
  assert.equal(
    createHash("sha1").update(`commit ${commit.length}\0`).update(commit).digest("hex"),
    manifest.source,
  );
  assert.equal(commit.length, frame.sourceCommit.bytes);
  assert.equal(sha(commit), frame.sourceCommit.sha256);
  assert.deepEqual(commit, Buffer.from(frame.sourceCommit.base64, "base64"));
  const sourceTree = treeRows(rawGit(literalGit, ["ls-tree", "-r", "-z", manifest.source]));
  const bodies = new Map();
  assert.equal(frame.files.length, manifest.historicalFrameRows);
  for (const row of frame.files) {
    safePath(row.path);
    assert(!bodies.has(row.path), row.path);
    const tree = sourceTree.get(row.path);
    assert(tree && tree.mode === "100644" && tree.kind === "blob", row.path);
    assert.equal(row.mode, tree.mode);
    assert.equal(row.gitBlob, tree.blob);
    const body = Buffer.from(row.base64, "base64");
    assert.equal(body.toString("base64"), row.base64);
    assert.equal(body.length, row.bytes, row.path);
    assert.equal(sha(body), row.sha256, row.path);
    assert.equal(
      createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      row.gitBlob,
    );
    bodies.set(row.path, body);
  }
  const required = [...sourceTree.keys()]
    .filter((name) => manifest.completeSubtreeScopes.some((prefix) => name.startsWith(prefix)))
    .toSorted(compareNames);
  assert.deepEqual(
    [...bodies.keys()]
      .filter((name) => manifest.completeSubtreeScopes.some((prefix) => name.startsWith(prefix)))
      .toSorted(compareNames),
    required,
  );
  for (const absent of manifest.historicalNullablePaths) {
    safePath(absent);
    assert.equal(sourceTree.has(absent), false, "genuine committed absence: " + absent);
    assert.equal(bodies.has(absent), false);
  }
  for (const law of manifest.laws) {
    const original = bodies.get(law.path);
    assert(original, law.path);
    assert.equal(sha(original), law.sha256);
    assert.deepEqual(original, fs.readFileSync(path.join(root, law.archive)));
  }
  return bodies;
}

/** The isolated HEAD and object store describe the real2489 source; production Git stays untouched. */
export function runNoMutatingLiteralCaller(root, context, law) {
  assert.equal(context.sourceOnly, true);
  assert.equal(context.oldMeasurement, null);
  assert.equal(context.oldSetup, null);
  const manifest = context.manifest;
  assert(manifest.laws.some((row) => row.path === law.path && row.sha256 === law.sha256));
  const bodies = authenticateNoMutatingCallerFrame(root, manifest);
  const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-no-mutating-current-caller-history-"));
  try {
    for (const [name, body] of bodies) {
      const target = path.join(owned, name);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, body, { mode: 0o644, flag: "wx" });
      assert.deepEqual(fs.readFileSync(target), body);
    }
    const literalGit = privateNoMutatingCallerGit(root, manifest.source);
    const common = rawGit(literalGit, ["rev-parse", "--git-common-dir"]).toString().trim();
    const objects = fs.realpathSync(path.resolve(literalGit, common, "objects"));
    assert(!objects.includes("\n"));
    const origin = rawGit(root, ["remote", "get-url", "origin"]).toString().trim();
    assert(origin && !/[\r\n]/u.test(origin));
    fs.mkdirSync(path.join(owned, ".git/objects/info"), { recursive: true });
    fs.mkdirSync(path.join(owned, ".git/refs"));
    fs.writeFileSync(path.join(owned, ".git/objects/info/alternates"), objects + "\n");
    fs.writeFileSync(path.join(owned, ".git/HEAD"), manifest.source + "\n");
    fs.writeFileSync(
      path.join(owned, ".git/config"),
      '[core]\nrepositoryformatversion = 0\nbare = false\n[remote "origin"]\nurl = ' +
        JSON.stringify(origin) +
        "\n",
    );
    copyNoMutatingCallerShallowBoundary(literalGit, owned);
    assert.equal(rawGit(owned, ["rev-parse", "HEAD"]).toString().trim(), manifest.source);
    assert.deepEqual(
      rawGit(owned, ["cat-file", "commit", manifest.source]),
      rawGit(literalGit, ["cat-file", "commit", manifest.source]),
    );
    const env = { ...process.env };
    delete env.NODE_TEST_CONTEXT;
    for (const name of Object.keys(env)) if (name.startsWith("GIT_")) delete env[name];
    const result = spawnSync(process.execPath, ["--test", "--test-reporter=tap", law.path], {
      cwd: owned,
      env,
      encoding: "utf8",
      maxBuffer: 4 * 1024 * 1024,
      timeout: 60_000,
    });
    const packetDirectory = path.join(root, "target/differential/no-mutating-current-callers");
    fs.mkdirSync(packetDirectory, { recursive: true });
    const packet = path.join(packetDirectory, path.basename(law.path) + ".json");
    fs.writeFileSync(
      packet,
      JSON.stringify(
        {
          source: manifest.source,
          originalWholeLawSHA256: law.sha256,
          nodeExecutable: process.execPath,
          nodeVersion: process.version,
          argv: ["--test", "--test-reporter=tap", law.path],
          literalHistoricalHEAD: manifest.source,
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
          stdout: result.stdout,
          stderr: result.stderr,
          stdoutSHA256: sha(result.stdout ?? ""),
          stderrSHA256: sha(result.stderr ?? ""),
          completeSourceFrameRows: bodies.size,
          current23: context.completeCurrentBodies,
          currentOldMeasurement: null,
          currentOldSetup: null,
          sourceOnly: true,
          compiled: false,
          native: false,
          performance: false,
        },
        null,
        2,
      ) + "\n",
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    for (const line of [`# tests ${law.cases}`, `# pass ${law.cases}`, "# fail 0", "# skipped 0"])
      assert(result.stdout.split("\n").includes(line), result.stdout);
    return {
      source: manifest.source,
      originalWholeLawSHA256: law.sha256,
      completeSourceFrameRows: bodies.size,
      historicalPassed: law.cases,
      current23: context.completeCurrentBodies,
      currentOldMeasurement: null,
      currentOldSetup: null,
      packet,
      sourceOnly: true,
      compiled: false,
      native: false,
      performance: false,
    };
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
}

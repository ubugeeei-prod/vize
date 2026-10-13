import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import { provisionJoinColdSources } from "./no-mutating-component-join-cold.mjs";
import { prepareLiteralSource } from "../../performance/support/warm-type-backed-git-reader-nodeep.ts";
import { copyNoMutatingCallerShallowBoundary } from "./no-mutating-props-current-caller-git.mjs";
import { seedNoMutatingCallerAncestry } from "./no-mutating-props-current-caller-ancestry.mjs";
import {
  joinGit,
  joinIdentity,
  joinObjectID,
  joinPath,
  joinPhysical,
  joinRawBodies,
  joinSHA,
} from "./no-mutating-component-join-physical.mjs";

const pools = new Map();
process.once("exit", () => {
  for (const owned of pools.values()) fs.rmSync(owned, { recursive: true, force: true });
});
function privateMetadata(root, source) {
  const key = fs.realpathSync(root) + "\0" + source;
  if (pools.has(key)) return pools.get(key);
  const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-no-mutating-component-literal-"));
  pools.set(key, owned);
  const common = joinGit(root, ["rev-parse", "--git-common-dir"]).toString().trim();
  const objects = fs.realpathSync(path.resolve(root, common, "objects"));
  assert(!/[\r\n]/u.test(objects));
  const origin = joinGit(root, ["remote", "get-url", "origin"]).toString().trim();
  assert(origin && !/[\r\n]/u.test(origin));
  fs.mkdirSync(path.join(owned, ".git/objects/info"), { recursive: true });
  fs.mkdirSync(path.join(owned, ".git/refs"));
  const historical = "2489128d5f8fab1b7442a1e5c09b4e1701cbf185";
  fs.writeFileSync(path.join(owned, ".git/HEAD"), historical + "\n");
  fs.writeFileSync(path.join(owned, ".git/objects/info/alternates"), objects + "\n");
  fs.writeFileSync(
    path.join(owned, ".git/config"),
    '[core]\nrepositoryformatversion = 0\nbare = false\n[maintenance]\nauto = false\n[gc]\nauto = 0\n[remote "origin"]\nurl = ' +
      JSON.stringify(origin) +
      "\n",
  );
  copyNoMutatingCallerShallowBoundary(root, owned);
  prepareLiteralSource(owned, historical);
  seedNoMutatingCallerAncestry(root, owned);
  prepareLiteralSource(owned, source);
  fs.writeFileSync(path.join(owned, ".git/HEAD"), source + "\n");
  assert.equal(joinGit(owned, ["rev-parse", "HEAD"]).toString().trim(), source);
  return owned;
}
export function authenticateJoinLiteralFrame(root, declared) {
  const frame = JSON.parse(gunzipSync(joinPhysical(root, declared)));
  assert.equal(frame.schema, "vize.no-mutating-component.literal-source-frame");
  assert.equal(frame.version, 1);
  assert.equal(frame.source, declared.source);
  assert.equal(frame.files.length, declared.files);
  assert.equal(new Set(frame.files.map((row) => row.path)).size, frame.files.length);
  assert.deepEqual(
    frame.files.map((row) => row.path),
    frame.files.map((row) => row.path).toSorted(),
  );
  assert.deepEqual(frame.credit, {
    sourceOnly: true,
    compiled: false,
    native: false,
    performance: false,
  });
  const owned = privateMetadata(root, frame.source);
  provisionJoinColdSources(owned, frame.requiredGitSources);
  const commit = joinGit(owned, ["cat-file", "commit", frame.source]);
  assert.equal(joinObjectID("commit", commit), frame.source);
  assert.equal(commit.length, frame.commit.bytes);
  assert.equal(joinSHA(commit), frame.commit.sha256);
  assert.deepEqual(commit, Buffer.from(frame.commit.rawBase64, "base64"));
  assert.equal(commit.subarray(0, commit.indexOf(10)).toString(), "tree " + frame.tree);
  const bodies = joinRawBodies(owned, frame.source, frame.files);
  assert.equal(frame.genuineAbsentPaths.length, declared.genuineAbsentPaths);
  assert.equal(new Set(frame.genuineAbsentPaths).size, frame.genuineAbsentPaths.length);
  for (const name of frame.genuineAbsentPaths) {
    joinPath(name);
    assert.equal(bodies.has(name), false);
  }
  assert.equal(
    joinGit(owned, ["ls-tree", "-z", frame.source, "--", ...frame.genuineAbsentPaths]).length,
    0,
  );
  for (const prefix of frame.completeSubtreeScopes) {
    assert(prefix.endsWith("/"));
    joinPath(prefix.slice(0, -1));
    const entries = joinGit(owned, ["ls-tree", "-r", "-z", frame.source, "--", prefix]);
    const actual = entries
      .toString()
      .split("\0")
      .filter(Boolean)
      .map((row) => row.slice(row.indexOf("\t") + 1));
    assert.deepEqual(
      actual,
      frame.files.filter((row) => row.path.startsWith(prefix)).map((row) => row.path),
    );
  }
  if (frame.source === "49d95b8e93a9f5bcb73e0c1c307815d4b3a9d570") {
    const scanner = frame.scannerInputs;
    assert.equal(scanner.source, frame.source);
    assert.equal(scanner.paths.length, 4464);
    assert.equal(joinSHA(bodies.get(scanner.scanner.path)), scanner.scanner.sha256);
    const scopes = scanner.registeredCrates.flatMap((name) =>
      ["Cargo.toml", "src", "tests", "benches"].map((part) => "crates/" + name + "/" + part),
    );
    const entries = joinGit(owned, [
      "ls-tree",
      "-r",
      "-z",
      frame.source,
      "--",
      ...scopes,
      ...scanner.registeredDirectPaths,
    ]);
    const actual = entries
      .toString()
      .split("\0")
      .filter(Boolean)
      .map((entry) => entry.slice(entry.indexOf("\t") + 1))
      .filter((name) => name.endsWith(".rs") || name.endsWith("Cargo.toml"))
      .toSorted();
    assert.deepEqual(actual, scanner.paths, "complete original registered scanner membership");
    for (const name of actual) assert(bodies.has(name), "whole original scanner input: " + name);
  } else {
    assert.equal(frame.scannerInputs, undefined);
  }
  assert.equal(frame.registryCounterparts.length, 20);
  assert.equal(new Set(frame.registryCounterparts.map((row) => row.path)).size, 20);
  for (const row of frame.registryCounterparts) {
    assert.equal(row.source, frame.source);
    if (row.body === null) {
      assert.equal(bodies.has(row.path), false);
      assert(frame.genuineAbsentPaths.includes(row.path));
      assert(!row.declaredSources.includes(frame.source));
    } else {
      joinIdentity(bodies.get(row.path), row.body);
    }
  }
  for (const law of frame.laws) {
    const body = Buffer.from(law.rawBase64, "base64");
    assert.equal(body.toString("base64"), law.rawBase64);
    joinIdentity(body, law);
    assert.deepEqual(bodies.get(law.path), body);
  }
  for (const [name, body] of bodies) {
    const target = path.join(owned, name);
    if (fs.existsSync(target)) assert.deepEqual(fs.readFileSync(target), body);
    else {
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, body, { mode: 0o644, flag: "wx" });
    }
    joinPhysical(
      owned,
      frame.files.find((row) => row.path === name),
    );
  }
  assert.equal(joinGit(owned, ["rev-parse", "HEAD"]).toString().trim(), frame.source);
  return { frame, root: owned };
}

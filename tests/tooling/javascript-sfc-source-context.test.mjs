import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  authenticateJavaScriptSfcSource,
  javascriptSfcSourceReader,
  javascriptSfcSourcePaths,
  javascriptSfcProductionPaths,
  javascriptSfcBodyPairs,
} from "../../tools/support/compat/github/javascript-sfc-source-context.mjs";
import { qualifyJavaScriptSfcHost } from "../performance/support/warm-type-backed-javascript-sfc-host.ts";

const root = path.resolve(import.meta.dirname, "../..");
const sha = (body) => createHash("sha256").update(body).digest("hex");
const bodies = new Map(
  javascriptSfcSourcePaths.map((file) => [file, fs.readFileSync(path.join(root, file))]),
);
const read = (file) => bodies.get(file);
const pairs = new Map(
  javascriptSfcBodyPairs.map(([file, before, after]) => [file, { before, after }]),
);
const digest = (side, file) => pairs.get(file)?.[side] ?? null;

await test("all93 actual producer and caller bodies authenticate before the one historical source-only view", () => {
  const context = authenticateJavaScriptSfcSource(javascriptSfcSourceReader(root));
  assert.deepEqual(context.receipt, {
    authority: "complete-javascript-sfc-source-context",
    bodyPairs: 93,
    baseBodyPairs: 88,
    changedProductionBodies: 29,
    sourceOnly: true,
    runtimeCredit: false,
    original400Credit: false,
    performanceCredit: false,
  });
  const recent = "crates/vize_canon/src/batch/type_checker/tests/recent_issues.rs";
  assert.equal(
    sha(context.historicalAfter(recent)),
    "32e031cecb75c24a5cdf70c2423b7d73504d164781ad54948b33b05cef848b53",
  );
  for (const file of javascriptSfcSourcePaths.filter((file) => file !== recent))
    assert.deepEqual(context.historicalAfter(file), read(file));
  const borrowed = context.historicalAfter(recent);
  borrowed[0] ^= 1;
  assert.notDeepEqual(context.historicalAfter(recent), borrowed);
  assert.throws(() => context.historicalAfter("unknown/source.rs"));
});

await test("each whole current producer corpus and source helper body is required before projection", () => {
  for (const file of javascriptSfcSourcePaths)
    for (const changed of [null, Buffer.concat([read(file), Buffer.from("\n")])])
      assert.throws(() =>
        authenticateJavaScriptSfcSource((name) => (name === file ? changed : read(name))),
      );
});

await test("actual before-after source qualifier retains every pair and rejects partial mixed or foreign scope", () => {
  const result = qualifyJavaScriptSfcHost([...javascriptSfcProductionPaths], new Set(), digest);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 93);
  assert.equal(result.original400Credit, false);
  for (const [file, before, after] of javascriptSfcBodyPairs)
    for (const side of ["before", "after"])
      for (const value of [null, "f".repeat(64), side === "before" ? after : before]) {
        if (value === digest(side, file)) continue;
        assert.equal(
          qualifyJavaScriptSfcHost([...javascriptSfcProductionPaths], new Set(), (s, name) =>
            s === side && name === file ? value : digest(s, name),
          ),
          null,
        );
      }
  for (const files of [
    [],
    javascriptSfcProductionPaths.slice(1),
    [...javascriptSfcProductionPaths, "unknown.rs"],
    [...javascriptSfcProductionPaths, javascriptSfcProductionPaths[0]],
  ])
    assert.equal(qualifyJavaScriptSfcHost(files, new Set(), digest), null);
  assert.equal(
    qualifyJavaScriptSfcHost(
      [...javascriptSfcProductionPaths],
      new Set(javascriptSfcProductionPaths),
      digest,
    ),
    null,
  );
});

await test("real private Git and physical missing mode symlink body replacement controls refuse", () => {
  const privateRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-js-source-control-"));
  const git = (args) =>
    execFileSync("git", ["--no-replace-objects", ...args], { cwd: privateRoot });
  try {
    for (const [file, body] of bodies) {
      const dst = path.join(privateRoot, file);
      fs.mkdirSync(path.dirname(dst), { recursive: true });
      fs.writeFileSync(dst, body);
    }
    git(["init", "--quiet"]);
    git(["add", "."]);
    git([
      "-c",
      "user.name=Source law",
      "-c",
      "user.email=law@invalid.example",
      "commit",
      "--quiet",
      "-m",
      "private source controls",
    ]);
    assert.doesNotThrow(() =>
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(privateRoot, true)),
    );
    const file = javascriptSfcProductionPaths[0],
      dst = path.join(privateRoot, file),
      body = read(file);
    fs.unlinkSync(dst);
    assert.throws(() =>
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(privateRoot, true)),
    );
    fs.writeFileSync(dst, body);
    fs.chmodSync(dst, 0o755);
    assert.throws(() =>
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(privateRoot, true)),
    );
    fs.chmodSync(dst, 0o644);
    fs.unlinkSync(dst);
    fs.symlinkSync(path.join(root, file), dst);
    assert.throws(() =>
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(privateRoot, true)),
    );
    fs.unlinkSync(dst);
    fs.writeFileSync(dst, Buffer.concat([body, Buffer.from("\n")]));
    assert.throws(() =>
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(privateRoot, true)),
    );
    fs.writeFileSync(dst, body);
    const original = git(["rev-parse", "HEAD"]).toString().trim();
    const replacement = git([
      "-c",
      "user.name=Source law",
      "-c",
      "user.email=law@invalid.example",
      "commit-tree",
      `${original}^{tree}`,
      "-m",
      "foreign raw identity",
    ])
      .toString()
      .trim();
    git(["replace", original, replacement]);
    assert.throws(() => javascriptSfcSourceReader(privateRoot, true));
  } finally {
    fs.rmSync(privateRoot, { recursive: true, force: true });
  }
});

await test("five caller inverses recover every old source and packet law without weakening their tails", () => {
  const file = path.join(root, "tests/_fixtures/ci/javascript-sfc-source-callers.json");
  const records = JSON.parse(fs.readFileSync(file));
  const context = authenticateJavaScriptSfcSource(javascriptSfcSourceReader(root));
  assert.equal(records.length, 5);
  for (const row of records) {
    const before = Buffer.from(row.beforeBody, "base64"),
      after = context.retainedCapturedAfter(row.path);
    assert.equal(sha(before), row.beforeSHA256);
    assert.equal(sha(after), row.afterSHA256);
    assert.deepEqual(after, Buffer.from(row.afterBody, "base64"));
    let restored = after.toString();
    for (const [_i, _j, start, end, old, current] of [...row.inverse].reverse()) {
      assert.equal(restored.slice(start, end), current);
      restored = restored.slice(0, start) + old + restored.slice(end);
    }
    assert.deepEqual(Buffer.from(restored), before);
  }
});

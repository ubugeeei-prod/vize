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
  javascriptSfcBodyPairs,
} from "../../tools/support/compat/github/javascript-sfc-source-context.mjs";
import { retainedJavaScriptSfcOptionsAfter } from "../../tools/support/compat/github/javascript-sfc-options-source.mjs";
import { javascriptSfcProductionPaths } from "../../tools/support/compat/github/javascript-sfc-source-context.mjs";
import {
  qualifyJavaScriptSfcHost,
  javascriptSfcStarAliasCorrection,
  javascriptSfcStarAliasMetadataPairs,
} from "../performance/support/warm-type-backed-javascript-sfc-host.ts";

const root = path.resolve(import.meta.dirname, "../..");
const data = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../tools/support/compat/github/javascript-sfc-options-source.json",
      import.meta.url,
    ),
  ),
);
const sha = (body) => createHash("sha256").update(body).digest("hex");
const read = javascriptSfcSourceReader(root, true);
const bodies = new Map(javascriptSfcSourcePaths.map((file) => [file, read(file)]));
void test("complete current93 source retains the exact original88 model without lending native credit", () => {
  const current = authenticateJavaScriptSfcSource(read);
  assert.equal(current.receipt.bodyPairs, 93);
  assert.equal(current.receipt.baseBodyPairs, 88);
  assert.equal(current.receipt.runtimeCredit, false);
  assert.equal(javascriptSfcBodyPairs.length, 93);
  for (const entry of data.existing) {
    assert.deepEqual(
      current.retainedOptionsAfter(entry.path),
      Buffer.from(entry.currentAfter.body, "base64"),
    );
    assert.deepEqual(
      current.retainedCapturedAfter(entry.path),
      Buffer.from(entry.retainedAfter.body, "base64"),
    );
    const borrowed = current.retainedCapturedAfter(entry.path);
    borrowed[0] ^= 1;
    assert.deepEqual(
      current.retainedCapturedAfter(entry.path),
      Buffer.from(entry.retainedAfter.body, "base64"),
    );
  }
  for (const row of data.supplemental) {
    const pair = javascriptSfcBodyPairs.find(([file]) => file === row.path);
    assert.deepEqual(pair, [row.path, row.before?.sha256 ?? null, row.after.sha256]);
    assert.throws(() => current.retainedCapturedAfter(row.path));
  }
  read.confirmSource();
});
void test("each repaired old or missing body and every supplemental body refuses before an original source loan", () => {
  for (const row of [...data.existing, ...data.supplemental]) {
    const values = [null, Buffer.concat([bodies.get(row.path), Buffer.from("\n")])];
    if (row.retainedAfter) values.push(Buffer.from(row.retainedAfter.body, "base64"));
    for (const changed of values)
      assert.throws(() =>
        authenticateJavaScriptSfcSource((file) => (file === row.path ? changed : bodies.get(file))),
      );
  }
  assert.throws(() =>
    retainedJavaScriptSfcOptionsAfter(
      data.existing[0].path,
      new Map([[data.existing[0].path, bodies.get(data.existing[0].path)]]),
    ),
  );
  const foreign = new Map(bodies);
  foreign.set(data.existing[0].path, Buffer.from("foreign"));
  assert.throws(() => retainedJavaScriptSfcOptionsAfter(data.existing[1].path, foreign));
});
void test("genuine private raw and physical modes missing symlinks mutations and replacements refuse", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-js-options-source-"));
  const git = (args, input) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: fixture,
      input,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
    });
  try {
    git(["init", "--quiet"]);
    for (const [file, body] of bodies) {
      const target = path.join(fixture, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, body, { mode: 0o644 });
    }
    git(["add", "--", "."]);
    git([
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "--quiet",
      "-m",
      "complete current93 source",
    ]);
    const head = git(["rev-parse", "HEAD"]).toString().trim();
    assert.equal(
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)).receipt.bodyPairs,
      93,
    );
    for (const row of [...data.existing, ...data.supplemental]) {
      const target = path.join(fixture, row.path),
        body = bodies.get(row.path);
      fs.unlinkSync(target);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.writeFileSync(target, body, { mode: 0o755 });
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.chmodSync(target, 0o644);
      fs.unlinkSync(target);
      fs.symlinkSync(path.join(root, row.path), target);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.unlinkSync(target);
      fs.writeFileSync(target, body, { mode: 0o644 });
      const changed = Buffer.concat([body, Buffer.from("\n")]);
      fs.writeFileSync(target, changed);
      git(["add", "--", row.path]);
      const tree = git(["write-tree"]).toString().trim();
      const unknown = git(
        [
          "-c",
          "user.name=Fixture",
          "-c",
          "user.email=fixture@example.invalid",
          "commit-tree",
          tree,
        ],
        "unknown matching raw/physical source\n",
      )
        .toString()
        .trim();
      git(["update-ref", "HEAD", unknown]);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      git(["update-ref", "HEAD", head]);
      git(["read-tree", head]);
      fs.writeFileSync(target, body);
    }
    git(["update-ref", "refs/replace/" + head, head]);
    assert.throws(() => javascriptSfcSourceReader(fixture, true));
    git(["update-ref", "-d", "refs/replace/" + head]);
    assert.equal(
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)).receipt.bodyPairs,
      93,
    );
    const current = authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true));
    assert.equal(
      sha(current.retainedOptionsAfter(data.existing[0].path)),
      data.existing[0].currentAfter.sha256,
    );
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

void test("actual29 changed crate sources qualify while both passive Options fixtures remain mandatory", () => {
  // Classify the complete authenticated source delta, independently of its declared production list.
  const actualProduction = javascriptSfcBodyPairs
    .filter(([file, before, after]) => file.startsWith("crates/") && before !== after)
    .map(([file]) => file);
  assert.equal(actualProduction.length, 29);
  assert.deepEqual([...javascriptSfcProductionPaths], actualProduction);
  const pairs = new Map(javascriptSfcBodyPairs.map(([file, before]) => [file, before]));
  // These callbacks are source-only controls over pinned before cells and genuine raw/physical after bytes.
  const digest = (side, file) => {
    assert(pairs.has(file), "closed current93 source digest");
    return side === "before" ? pairs.get(file) : sha(bodies.get(file));
  };
  const result = qualifyJavaScriptSfcHost(actualProduction, new Set(), digest);
  assert(result);
  assert.equal(result.completeBodyPairs, 93);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.original400Credit, false);
  assert.equal(result.performanceCredit, false);
  const fixtures = data.supplemental.filter(({ path }) => path.endsWith(".vue.txt"));
  assert.equal(fixtures.length, 2);
  for (const row of fixtures) {
    assert(!actualProduction.includes(row.path));
    assert.equal(pairs.get(row.path), null);
    assert.equal(sha(bodies.get(row.path)), row.after.sha256);
    for (const changed of [null, Buffer.concat([bodies.get(row.path), Buffer.from("\n")])]) {
      assert.throws(() =>
        authenticateJavaScriptSfcSource((file) => (file === row.path ? changed : bodies.get(file))),
      );
      assert.equal(
        qualifyJavaScriptSfcHost(actualProduction, new Set(), (side, file) =>
          side === "after" && file === row.path
            ? changed === null
              ? null
              : sha(changed)
            : digest(side, file),
        ),
        null,
      );
    }
  }
  read.confirmSource();
});

void test("only actual30 plus the whole typed-and-denied Star test pair admits the current source", () => {
  const alias = javascriptSfcStarAliasCorrection;
  const actual = fs.readFileSync(path.join(root, alias.path));
  assert.equal(sha(actual), alias.currentAfter.sha256);
  const production = [...javascriptSfcProductionPaths, alias.path];
  assert.equal(production.length, 30);
  const before = new Map(javascriptSfcBodyPairs.map(([file, value]) => [file, value]));
  const after = new Map(javascriptSfcSourcePaths.map((file) => [file, sha(bodies.get(file))]));
  before.set(alias.path, alias.currentBefore.sha256);
  after.set(alias.path, sha(actual));
  for (const [file, original, expected] of javascriptSfcStarAliasMetadataPairs) {
    before.set(file, original);
    const value = sha(fs.readFileSync(path.join(root, file)));
    assert.equal(value, expected);
    after.set(file, value);
  }
  // Finite source controls use only raw-authenticated before cells and genuine physical after bytes.
  const digest = (side, file) => {
    const values = side === "before" ? before : after;
    assert(values.has(file), "unknown source cannot become an alias correction");
    return values.get(file);
  };
  const admitted = qualifyJavaScriptSfcHost(production, new Set(), digest);
  assert(admitted);
  assert.equal(admitted.completeBodyPairs, 93);
  assert.equal(admitted.starAliasTestPairs, 1);
  assert.equal(admitted.files.length, 30);
  assert.equal(admitted.runtimeCredit, false);
  assert.equal(admitted.original400Credit, false);
  assert.equal(admitted.performanceCredit, false);
  // The genuine old277 whole test is the refused current-source counterexample.
  assert.equal(
    qualifyJavaScriptSfcHost(production, new Set(), (side, file) =>
      side === "after" && file === alias.path ? alias.currentBefore.sha256 : digest(side, file),
    ),
    null,
  );
  for (const file of [alias.path, ...javascriptSfcStarAliasMetadataPairs.map(([file]) => file)])
    for (const side of ["before", "after"])
      for (const value of [
        null,
        "f".repeat(64),
        side === "before" ? after.get(file) : before.get(file),
      ]) {
        if (value === digest(side, file)) continue;
        assert.equal(
          qualifyJavaScriptSfcHost(production, new Set(), (s, name) =>
            s === side && name === file ? value : digest(s, name),
          ),
          null,
        );
      }
  for (const files of [
    [],
    production.slice(1),
    [...production, "unknown.rs"],
    [...production, alias.path],
  ])
    assert.equal(qualifyJavaScriptSfcHost(files, new Set(), digest), null);
  assert.equal(qualifyJavaScriptSfcHost(production, new Set(production), digest), null);
  const original = qualifyJavaScriptSfcHost([...javascriptSfcProductionPaths], new Set(), digest);
  assert(original && !("starAliasTestPairs" in original));
  assert.equal(original.completeBodyPairs, 93);
  read.confirmSource();
});

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  authenticateNoMutatingCurrentCallerSource,
  composeNoMutatingCurrentCallerLaw,
  noMutatingCurrentCallerDirectory,
  noMutatingCurrentCallerManifestSHA256,
} from "./support/no-mutating-props-current-caller-source-custody.mjs";
import { authenticateNoMutatingCallerFrame } from "./support/no-mutating-props-current-caller-frame.mjs";
import { gitReaderSetupPairs } from "../performance/support/warm-type-backed-git-reader-setup.ts";
import {
  privateNoMutatingCallerGit,
  copyNoMutatingCallerShallowBoundary,
} from "./support/no-mutating-props-current-caller-git.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const sha = (body) => createHash("sha256").update(body).digest("hex");
const read = (name) => fs.readFileSync(path.join(root, name));
const manifest = JSON.parse(read(noMutatingCurrentCallerDirectory + "/successor.json"));
const positive = () => authenticateNoMutatingCurrentCallerSource(root);

void test("current complete23 refuses both immutable old tuples and grants only its new source context", () => {
  const result = positive();
  assert.equal(result.completeCurrentBodies, 23);
  assert.equal(new Set(result.paths).size, 23);
  assert.equal(result.oldMeasurement, null);
  assert.equal(result.oldSetup, null);
  assert.equal(result.historicalProjection, false);
  assert.equal(result.sourceOnly, true);
  for (const key of ["compiled", "native", "performance"]) assert.equal(result[key], false);
});

void test("each of the23 actual complete bodies refuses missing, foreign and historical source reads", () => {
  const current = positive();
  let controls = 0;
  for (const file of current.paths) {
    for (const replacement of [null, Buffer.from("foreign complete body\n")]) {
      controls++;
      assert.throws(
        () =>
          authenticateNoMutatingCurrentCallerSource(root, (name) => {
            if (name !== file) return read(name);
            if (replacement === null)
              throw Object.assign(new Error("missing named source"), { code: "ENOENT" });
            return replacement;
          }),
        undefined,
        file,
      );
    }
    assert.equal(positive().completeCurrentBodies, 23);
  }
  assert.equal(controls, 46);
  assert.throws(() =>
    authenticateNoMutatingCurrentCallerSource(root, (name) =>
      name === manifest.rule.path ? read(manifest.rule.before.archive) : read(name),
    ),
  );
  assert.equal(positive().oldMeasurement, null);
});

function sourceFixture() {
  const files = new Set();
  authenticateNoMutatingCurrentCallerSource(root, (name) => {
    files.add(name);
    return read(name);
  });
  for (const [file] of gitReaderSetupPairs)
    if (fs.existsSync(path.join(root, file))) files.add(file);
  const migration = JSON.parse(
    read("tests/performance/support/warm-type-backed-default-migration-manifest.json"),
  );
  for (const [file] of [...migration.files, ...migration.preservedBodies]) {
    if (fs.existsSync(path.join(root, file))) files.add(file);
  }
  const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-no-mutating-source-refusals-"));
  for (const name of files) {
    const target = path.join(owned, name);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, read(name), { mode: 0o644 });
  }
  const git = (...args) => execFileSync("git", ["--no-replace-objects", ...args], { cwd: root });
  const common = git("rev-parse", "--git-common-dir").toString().trim();
  fs.mkdirSync(path.join(owned, ".git/objects/info"), { recursive: true });
  fs.mkdirSync(path.join(owned, ".git/refs"));
  fs.writeFileSync(path.join(owned, ".git/HEAD"), git("rev-parse", "HEAD"));
  fs.writeFileSync(
    path.join(owned, ".git/objects/info/alternates"),
    path.resolve(root, common, "objects") + "\n",
  );
  fs.writeFileSync(
    path.join(owned, ".git/config"),
    '[core]\nrepositoryformatversion = 0\nbare = false\n[remote "origin"]\nurl = ' +
      JSON.stringify(git("remote", "get-url", "origin").toString().trim()) +
      "\n",
  );
  copyNoMutatingCallerShallowBoundary(root, owned);
  return owned;
}

void test("physical omission, executable, symlink and old whole rule substitutes refuse after fresh positives", () => {
  const owned = sourceFixture();
  const target = path.join(owned, manifest.rule.path);
  const original = fs.readFileSync(target);
  const restore = () => {
    fs.rmSync(target, { force: true });
    fs.writeFileSync(target, original, { mode: 0o644 });
  };
  try {
    assert.equal(authenticateNoMutatingCurrentCallerSource(owned).completeCurrentBodies, 23);
    for (const mutate of [
      () => fs.rmSync(target),
      () => fs.chmodSync(target, 0o755),
      () => {
        fs.rmSync(target);
        fs.symlinkSync(path.join(root, manifest.rule.path), target);
      },
      () => fs.writeFileSync(target, read(manifest.rule.before.archive)),
    ]) {
      mutate();
      assert.throws(() => authenticateNoMutatingCurrentCallerSource(owned));
      restore();
      assert.equal(authenticateNoMutatingCurrentCallerSource(owned).oldMeasurement, null);
    }
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
});

void test("all three original callers retain13 cases and the fourth retains its whole7 raw2489 cases", () => {
  positive();
  const frame = authenticateNoMutatingCallerFrame(root, manifest);
  assert.equal(frame.size, 425);
  assert.equal(
    manifest.laws.slice(0, 3).reduce((count, law) => count + law.cases, 0),
    13,
  );
  assert.equal(manifest.laws[3].cases, 7);
  for (const law of manifest.laws) {
    assert.deepEqual(frame.get(law.path), read(law.archive));
    assert.equal(sha(frame.get(law.path)), law.sha256);
    assert.deepEqual(read(law.path), composeNoMutatingCurrentCallerLaw(law.path, law.cases));
  }
});

void test("frame source and whole raw commit substitutions refuse rather than filling missing rows", () => {
  positive();
  for (const change of [
    { source: "0".repeat(40) },
    { historicalFrameRows: 415 },
    { historicalNullablePaths: ["crates/vize_patina/src/rule.rs"] },
    { completeSubtreeScopes: [] },
    { laws: [...manifest.laws, manifest.laws[0]] },
  ])
    assert.throws(() => authenticateNoMutatingCallerFrame(root, { ...manifest, ...change }));
  assert.equal(authenticateNoMutatingCallerFrame(root, manifest).size, 425);
  assert.equal(positive().completeCurrentBodies, 23);
});

void test("the external source law seals executing helper bodies without a self-checksum cycle", () => {
  assert.equal(
    sha(read(noMutatingCurrentCallerDirectory + "/successor.json")),
    noMutatingCurrentCallerManifestSHA256,
  );
  const fixed = [
    [
      "tests/tooling/support/no-mutating-props-current-caller-source-custody.mjs",
      "b27f733bb618bed3a9de302056024ed18e8082d639e0c2857d45ed122b0f4079",
    ],
    [
      "tests/tooling/support/no-mutating-props-current-caller-frame.mjs",
      "8963f748c143c6c051e84ba5fe45b75fa07d09acaa72605c83af721f60087a80",
    ],
    [
      "tests/tooling/support/no-mutating-props-current-caller-git.mjs",
      "d54ce79f0743fdd45b8d007cfd7dde61c0a7a5457bd879001d22b1ac528e3ffd",
    ],
  ];
  for (const [file, expected] of fixed) assert.equal(sha(read(file)), expected);
  assert.throws(() => privateNoMutatingCallerGit(root, "0".repeat(40)), /two declared/u);
  // This external law's full body is independently sealed by the parent source peer.
  // Neither the runtime manifest nor a running helper claims to authenticate itself.
  assert.equal(positive().performance, false);
});

void test("complete363 manifest and all four whole rejected source bodies remain fixed history", () => {
  const context = positive();
  const original = read(manifest.predecessorManifest.archive);
  assert.equal(sha(original), "fa45cf8fa852f0c222af9378a981cb77d51d33f2bf58bad346230e4a00f8f532");
  const before = JSON.parse(original);
  const inverse = structuredClone(manifest);
  for (const key of ["predecessorSource", "predecessorManifest", "predecessorSourceBodies"])
    delete inverse[key];
  inverse.laws = inverse.laws.slice(0, 3);
  inverse.historicalFrame = before.historicalFrame;
  inverse.historicalFrameRows = before.historicalFrameRows;
  assert.deepEqual(inverse, before);
  assert.equal(manifest.predecessorSourceBodies.length, 4);
  for (const expected of manifest.predecessorSourceBodies) {
    const body = read(expected.archive);
    assert.equal(body.length, expected.bytes);
    assert.equal(sha(body), expected.sha256);
    assert.equal(
      createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      expected.gitBlob,
    );
  }
  assert.equal(context.oldMeasurement, null);
  assert.equal(context.oldSetup, null);
});

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
import { privateNoMutatingCallerGit } from "./support/no-mutating-props-current-caller-git.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const sha = (body) => createHash("sha256").update(body).digest("hex");
const read = (name) => fs.readFileSync(path.join(root, name));
const manifest = JSON.parse(read(noMutatingCurrentCallerDirectory + "/successor.json"));
const positive = () => authenticateNoMutatingCurrentCallerSource(root);

test("current complete23 refuses both immutable old tuples and grants only its new source context", () => {
  const result = positive();
  assert.equal(result.completeCurrentBodies, 23);
  assert.equal(new Set(result.paths).size, 23);
  assert.equal(result.oldMeasurement, null);
  assert.equal(result.oldSetup, null);
  assert.equal(result.historicalProjection, false);
  assert.equal(result.sourceOnly, true);
  for (const key of ["compiled", "native", "performance"]) assert.equal(result[key], false);
});

test("each of the23 actual complete bodies refuses missing, foreign and historical source reads", () => {
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
  return owned;
}

test("physical omission, executable, symlink and old whole rule substitutes refuse after fresh positives", () => {
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

test("all three whole original caller modules and13 cases remain exact raw2489 inputs", () => {
  positive();
  const frame = authenticateNoMutatingCallerFrame(root, manifest);
  assert.equal(frame.size, 416);
  assert.equal(
    manifest.laws.reduce((count, law) => count + law.cases, 0),
    13,
  );
  for (const law of manifest.laws) {
    assert.deepEqual(frame.get(law.path), read(law.archive));
    assert.equal(sha(frame.get(law.path)), law.sha256);
    assert.deepEqual(read(law.path), composeNoMutatingCurrentCallerLaw(law.path, law.cases));
  }
});

test("frame source and whole raw commit substitutions refuse rather than filling missing rows", () => {
  positive();
  for (const change of [
    { source: "0".repeat(40) },
    { historicalFrameRows: 415 },
    { historicalNullablePaths: ["crates/vize_patina/src/rule.rs"] },
    { completeSubtreeScopes: [] },
    { laws: [...manifest.laws, manifest.laws[0]] },
  ])
    assert.throws(() => authenticateNoMutatingCallerFrame(root, { ...manifest, ...change }));
  assert.equal(authenticateNoMutatingCallerFrame(root, manifest).size, 416);
  assert.equal(positive().completeCurrentBodies, 23);
});

test("the external source law seals executing helper bodies without a self-checksum cycle", () => {
  assert.equal(
    sha(read(noMutatingCurrentCallerDirectory + "/successor.json")),
    noMutatingCurrentCallerManifestSHA256,
  );
  const fixed = [
    [
      "tests/tooling/support/no-mutating-props-current-caller-source-custody.mjs",
      "facaefef2ff091fe2252110391f4587632dc114055c39ab6186fa46b0a3f400d",
    ],
    [
      "tests/tooling/support/no-mutating-props-current-caller-frame.mjs",
      "c9b1eebfd15810bf5dfad8c90d2580a16da9a79ffa5083b2239d1d2336628add",
    ],
    [
      "tests/tooling/support/no-mutating-props-current-caller-git.mjs",
      "f9afa9088bf11b52332d0c7562f69fb698cc19495a9fcbfbd7c596e5af09a6b1",
    ],
  ];
  for (const [file, expected] of fixed) assert.equal(sha(read(file)), expected);
  assert.throws(() => privateNoMutatingCallerGit(root, "0".repeat(40)), /two declared/u);
  // This external law's full body is independently sealed by the parent source peer.
  // Neither the runtime manifest nor a running helper claims to authenticate itself.
  assert.equal(positive().performance, false);
});

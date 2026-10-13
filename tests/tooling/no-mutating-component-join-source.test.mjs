import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  authenticateNoMutatingComponentJoin,
  authenticateNoMutatingComponentJoinPhysical,
  componentJoinDirectory,
  composeNoMutatingComponentJoinCaller,
  authenticateNoMutatingComponentJoinCommitted,
} from "./support/no-mutating-component-join.mjs";
const root = path.resolve(import.meta.dirname, "../..");
const sha = (body) => createHash("sha256").update(body).digest("hex");
assert.equal(
  sha(fs.readFileSync(path.join(root, "tests/tooling/support/no-mutating-component-join.mjs"))),
  "73734d3cb8e0099dcac960f7926eadd5f499f8b441e3a4d9a013a63e03c07686",
);
import {
  assertJoinCommitted,
  joinGit,
  joinRawBodies,
} from "./support/no-mutating-component-join-physical.mjs";
import { provisionJoinColdSources } from "./support/no-mutating-component-join-cold.mjs";
import { joinObjectID, joinSHA } from "./support/no-mutating-component-join-physical.mjs";
const physical = authenticateNoMutatingComponentJoinPhysical(root);
void test("the coherent physical union and both complete literal source frames stay distinct", () => {
  const joined = authenticateNoMutatingComponentJoin(root);
  assert.equal(joined.completeCurrentUnionBodies, 206);
  assert.equal(joined.completeCurrentClosedBodies, 323);
  assert.deepEqual(
    joined.frames.map((context) => [context.frame.source, context.frame.files.length]),
    [
      ["49d95b8e93a9f5bcb73e0c1c307815d4b3a9d570", 5080],
      ["f20740ef6b0958ddb0bbbd64fd1b38210ea5e207", 683],
    ],
  );
  assert.equal(joined.manifest.completeOriginalNoMutatingCases, 20);
  assert.equal(joined.manifest.completeIncomingCallerCases, 20);
  assert.equal(joined.manifest.completePureHistoricalCases, 126);
  for (const { frame } of joined.frames) {
    const own = frame.requiredGitSources.find((row) => row.source === frame.source);
    assert.deepEqual(own.files, frame.files);
    assert.deepEqual(own.absentPaths, frame.genuineAbsentPaths);
  }
  assert.equal(joined.manifest.sourceRegistry.length, 20);
  assert.equal(joined.manifest.laws.length, 19);
  for (const entry of joined.manifest.sourceRegistry) {
    for (const context of joined.frames) {
      const cell = context.frame.registryCounterparts.find((row) => row.path === entry.path);
      assert(cell);
      if (entry.kind === "live-native-consumer") {
        assert(cell.body);
        assert.deepEqual(entry.declaredSources, []);
        assert(!context.frame.laws.some((row) => row.path === entry.path));
      } else {
        assert.equal(cell.body !== null, entry.declaredSources.includes(context.frame.source));
      }
    }
  }
  assert.equal(joined.oldMigrationMeasurement, null);
  assert.equal(joined.oldSetupMeasurement, null);
  assert.equal(joined.currentOld111Receipt, null);
  assert.equal(joined.native, false);
  assert.equal(joined.performance, false);
  for (const name of joined.manifest.laws)
    assert.deepEqual(joined.bodies.get(name), composeNoMutatingComponentJoinCaller(name));
  assert.throws(() => composeNoMutatingComponentJoinCaller("unreviewed.test.ts"));
});
function capsule() {
  const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-join-source-refusals-"));
  const rows = [...physical.manifest.current, ...physical.manifest.literalFrames];
  rows.push(physical.manifest.externalMaintenanceLaw);
  for (const row of rows) {
    const target = path.join(owned, row.path);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(root, row.path), target);
    fs.chmodSync(target, 0o644);
  }
  const index = componentJoinDirectory + "/successor.json";
  fs.copyFileSync(path.join(root, index), path.join(owned, index));
  fs.chmodSync(path.join(owned, index), 0o644);
  assert.equal(authenticateNoMutatingComponentJoinPhysical(owned).completeCurrentClosedBodies, 323);
  return { owned, rows, index };
}
void test("every complete physical source, helper, caller and before-frame rejects mutation and missing bytes", () => {
  const { owned, rows } = capsule();
  try {
    for (const row of rows) {
      const target = path.join(owned, row.path);
      const bytes = fs.readFileSync(target);
      fs.writeFileSync(target, Buffer.concat([bytes, Buffer.from("\nforeign source\n")]));
      assert.throws(() => authenticateNoMutatingComponentJoinPhysical(owned), row.path);
      fs.writeFileSync(target, bytes);
      fs.unlinkSync(target);
      assert.throws(() => authenticateNoMutatingComponentJoinPhysical(owned), row.path);
      fs.writeFileSync(target, bytes, { mode: 0o644, flag: "wx" });
    }
    assert.equal(
      authenticateNoMutatingComponentJoinPhysical(owned).completeCurrentClosedBodies,
      323,
    );
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
});
void test("executable, symlink, unknown, duplicate, omitted and hybrid source declarations never qualify", () => {
  const { owned, rows, index } = capsule();
  try {
    const row = rows.find((entry) => entry.path === "crates/vize_patina/src/linter/engine/sfc.rs");
    const target = path.join(owned, row.path);
    const bytes = fs.readFileSync(target);
    fs.chmodSync(target, 0o755);
    assert.throws(() => authenticateNoMutatingComponentJoinPhysical(owned));
    fs.chmodSync(target, 0o644);
    fs.unlinkSync(target);
    fs.symlinkSync(path.join(root, row.path), target);
    assert.throws(() => authenticateNoMutatingComponentJoinPhysical(owned));
    fs.unlinkSync(target);
    fs.writeFileSync(target, bytes, { mode: 0o644, flag: "wx" });
    const filename = path.join(owned, index);
    const original = fs.readFileSync(filename);
    const changed = (mutate) => {
      const value = JSON.parse(original);
      mutate(value);
      fs.writeFileSync(filename, JSON.stringify(value));
      assert.throws(() => authenticateNoMutatingComponentJoinPhysical(owned));
      fs.writeFileSync(filename, original);
    };
    changed((value) => value.current.pop());
    changed((value) => value.current.push(value.current[0]));
    changed((value) => (value.current[0].path = "unknown.rs"));
    changed((value) => value.sources.reverse());
    changed((value) => (value.literalFrames[0] = value.literalFrames[1]));
    changed((value) => value.families.currentReader111.pop());
    changed((value) => delete value.externalMaintenanceLaw);
    changed((value) => {
      const row = value.externalMaintenanceLaw;
      value.externalMaintenanceLaw = [row, row];
    });
    changed((value) => (value.externalMaintenanceLaw = value.current[0]));
    fs.unlinkSync(filename);
    assert.throws(() => authenticateNoMutatingComponentJoinPhysical(owned));
    fs.writeFileSync(filename, original, { mode: 0o644, flag: "wx" });
    assert.equal(
      authenticateNoMutatingComponentJoinPhysical(owned).completeCurrentUnionBodies,
      206,
    );
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
});
void test("actual raw HEAD and mutated committed source refuse an otherwise complete physical family", () => {
  const { owned, rows } = capsule();
  try {
    joinGit(owned, ["init", "--quiet"]);
    joinGit(owned, ["add", "--", "."]);
    const commit = () => {
      const tree = joinGit(owned, ["write-tree"]).toString().trim();
      const head = joinGit(
        owned,
        [
          "-c",
          "user.name=Source Control",
          "-c",
          "user.email=source-control@example.invalid",
          "commit-tree",
          tree,
        ],
        "finite whole source control\n",
      )
        .toString()
        .trim();
      fs.writeFileSync(path.join(owned, ".git/HEAD"), head + "\n");
      return head;
    };
    const head = commit();
    const ordered = rows.toSorted((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
    assert.equal(assertJoinCommitted(owned, ordered), head);
    const row = ordered.find(
      (entry) => entry.path === "crates/vize_patina/src/linter/engine/sfc.rs",
    );
    const filename = path.join(owned, row.path);
    const original = fs.readFileSync(filename);
    fs.writeFileSync(
      filename,
      Buffer.concat([original, Buffer.from("\nforeign committed body\n")]),
    );
    joinGit(owned, ["add", "--", row.path]);
    commit();
    fs.writeFileSync(filename, original);
    assert.equal(
      authenticateNoMutatingComponentJoinPhysical(owned).completeCurrentClosedBodies,
      323,
    );
    assert.throws(() => assertJoinCommitted(owned, ordered), /whole committed joined source/u);
    fs.writeFileSync(path.join(owned, ".git/HEAD"), head + "\n");
    assert.equal(assertJoinCommitted(owned, ordered), head);
    const literal = authenticateNoMutatingComponentJoin(root).frames[0];
    assert.equal(literal.frame.source, physical.manifest.sources[0]);
    const files = ["HEAD", "shallow"].map((n) => path.join(literal.root, ".git", n));
    const before = files.map((p) => (fs.existsSync(p) ? fs.readFileSync(p) : null));
    const objects = fs.realpathSync(path.join(literal.root, ".git/objects"));
    fs.mkdirSync(path.join(owned, ".git/objects/info"), { recursive: true });
    fs.writeFileSync(path.join(owned, ".git/objects/info/alternates"), objects + "\n");
    fs.writeFileSync(path.join(owned, ".git/HEAD"), physical.manifest.sources[0] + "\n");
    assert.throws(() => assertJoinCommitted(owned, ordered), /whole committed joined source/u);
    fs.writeFileSync(path.join(owned, ".git/HEAD"), head + "\n");
    assert.equal(assertJoinCommitted(owned, ordered), head);
    assert.throws(() => authenticateNoMutatingComponentJoinCommitted(owned));
    const after = files.map((p) => (fs.existsSync(p) ? fs.readFileSync(p) : null));
    assert.deepEqual(after, before);
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
});
void test("an existing real commit and tree hydrate a missing declared blob while foreign or corrupt objects refuse", () => {
  const area = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-cold-source-"));
  const origin = path.join(area, "origin");
  const owned = path.join(area, "owned");
  try {
    for (const directory of [origin, owned]) {
      fs.mkdirSync(directory);
      joinGit(directory, ["init", "--quiet"]);
    }
    const body = Buffer.from("complete cold source blob\n");
    fs.mkdirSync(path.join(origin, "selected"));
    fs.mkdirSync(path.join(origin, "unrelated"));
    fs.writeFileSync(path.join(origin, "selected/source.txt"), body);
    fs.writeFileSync(path.join(origin, "unrelated/input.txt"), "unrelated source\n");
    joinGit(origin, ["add", "--", "."]);
    const tree = joinGit(origin, ["write-tree"]).toString().trim();
    const source = joinGit(
      origin,
      [
        "-c",
        "user.name=Source Control",
        "-c",
        "user.email=source-control@example.invalid",
        "commit-tree",
        tree,
      ],
      "exact cold origin\n",
    )
      .toString()
      .trim();
    fs.writeFileSync(path.join(origin, ".git/HEAD"), source + "\n");
    const commit = joinGit(origin, ["cat-file", "commit", source]);
    const blob = joinObjectID("blob", body);
    const declaration = {
      source,
      bytes: commit.length,
      sha256: joinSHA(commit),
      completeTree: true,
      absentPaths: ["selected/absent.txt"],
      files: [
        {
          path: "selected/source.txt",
          mode: "100644",
          bytes: body.length,
          sha256: joinSHA(body),
          gitBlob: blob,
        },
      ],
    };
    const childTrees = ["selected", "unrelated"].map((name) =>
      joinGit(origin, ["rev-parse", source + ":" + name])
        .toString()
        .trim(),
    );
    for (const oid of [source, tree, ...childTrees]) {
      const relative = ".git/objects/" + oid.slice(0, 2) + "/" + oid.slice(2);
      fs.mkdirSync(path.dirname(path.join(owned, relative)), { recursive: true });
      fs.copyFileSync(path.join(origin, relative), path.join(owned, relative));
    }
    fs.writeFileSync(path.join(owned, ".git/HEAD"), source + "\n");
    joinGit(owned, ["remote", "add", "origin", origin]);
    assert.equal(
      joinGit(owned, ["cat-file", "--batch-check"], blob + "\n").toString(),
      blob + " missing\n",
    );
    provisionJoinColdSources(owned, [{ ...declaration, files: [] }]);
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    assert.throws(() => joinRawBodies(owned, source, declaration.files));
    provisionJoinColdSources(owned, [declaration]);
    assert.deepEqual(joinGit(owned, ["cat-file", "blob", blob]), body);
    fs.unlinkSync(path.join(owned, ".git/FETCH_HEAD"));
    const wrong = structuredClone(declaration);
    wrong.files[0].gitBlob = tree;
    assert.throws(() => provisionJoinColdSources(owned, [wrong]));
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    const foreign = structuredClone(declaration);
    foreign.files[0].gitBlob = joinObjectID("blob", Buffer.from("foreign absent object"));
    assert.throws(() => provisionJoinColdSources(owned, [foreign]));
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    const falseAbsence = { ...declaration, absentPaths: ["selected/source.txt"] };
    assert.throws(() => provisionJoinColdSources(owned, [falseAbsence]));
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    const malformed = structuredClone(declaration);
    malformed.files[0].gitBlob += " foreign";
    assert.throws(() => provisionJoinColdSources(owned, [malformed]));
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    const corrupt = structuredClone(declaration);
    corrupt.files[0].sha256 = "0".repeat(64);
    assert.throws(() => provisionJoinColdSources(owned, [corrupt]));
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    const loose = path.join(owned, ".git/objects", blob.slice(0, 2), blob.slice(2));
    fs.mkdirSync(path.dirname(loose), { recursive: true });
    const originalObject = fs.readFileSync(loose);
    fs.unlinkSync(loose);
    fs.writeFileSync(loose, "corrupt compressed object", { mode: 0o644, flag: "wx" });
    assert.throws(() => provisionJoinColdSources(owned, [declaration]));
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    fs.unlinkSync(loose);
    fs.writeFileSync(loose, originalObject, { mode: 0o444, flag: "wx" });
    provisionJoinColdSources(owned, [declaration]);
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    const partial = { ...declaration, completeTree: false };
    const treeObject = (name) => {
      const oid = joinGit(origin, ["rev-parse", source + ":" + name])
        .toString()
        .trim();
      return { oid, filename: path.join(owned, ".git/objects", oid.slice(0, 2), oid.slice(2)) };
    };
    const selectedTree = treeObject("selected");
    const unrelatedTree = treeObject("unrelated");
    fs.unlinkSync(selectedTree.filename);
    assert.deepEqual(joinGit(owned, ["cat-file", "blob", blob]), body);
    assert.equal(
      joinGit(owned, ["cat-file", "--batch-check"], selectedTree.oid + "\n").toString(),
      selectedTree.oid + " missing\n",
    );
    provisionJoinColdSources(owned, [partial]);
    assert.equal(joinGit(owned, ["cat-file", "-t", selectedTree.oid]).toString(), "tree\n");
    fs.unlinkSync(path.join(owned, ".git/FETCH_HEAD"));
    const unusedObject = fs.readFileSync(unrelatedTree.filename);
    fs.unlinkSync(unrelatedTree.filename);
    provisionJoinColdSources(owned, [partial]);
    assert.equal(fs.existsSync(path.join(owned, ".git/FETCH_HEAD")), false);
    assert.equal(
      joinGit(owned, ["cat-file", "--batch-check"], unrelatedTree.oid + "\n").toString(),
      unrelatedTree.oid + " missing\n",
    );
    fs.writeFileSync(unrelatedTree.filename, unusedObject, { mode: 0o444, flag: "wx" });
    provisionJoinColdSources(owned, [declaration]);
  } finally {
    fs.rmSync(area, { recursive: true, force: true });
  }
});

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  authenticateStarComponentReadSource,
  prepareRootComponentReadHistory,
  componentReadSourcePaths,
} from "../../tools/support/compat/github/empty-vue-star-component-read-source.mjs";
import { joinGit } from "./support/no-mutating-component-join-physical.mjs";
const owner = path.resolve(import.meta.dirname, "../..");
const directory = "tools/support/compat/github/";
const metadataDirectory =
  "tests/_fixtures/differential/linter/component-positive-template-reads/source-custody/";
const authored = [
  directory + "empty-vue-star-component-read-source.mjs",
  metadataDirectory + "empty-vue-star-component-read-source-0.json",
  metadataDirectory + "empty-vue-star-component-read-source-1.json",
  metadataDirectory + "empty-vue-star-component-read-cells.json",
  directory + "source-counterpart-snapshot.mjs",
  "tests/tooling/empty-vue-star-component-read-source.test.mjs",
  "tests/tooling/misskey-registry-source.test.mjs",
  "tests/tooling/warm-type-backed-empty-vue-star-custody.test.ts",
];
function capsule(run) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-star-positive-read-source-"));
  try {
    joinGit(root, ["init", "--quiet"]);
    const pool = joinGit(owner, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(root, ".git/objects/info/alternates"),
      fs.realpathSync(path.resolve(owner, pool, "objects")) + "\n",
    );
    fs.writeFileSync(path.join(root, ".git/HEAD"), joinGit(owner, ["rev-parse", "HEAD"]));
    for (const file of new Set([...componentReadSourcePaths, ...authored])) {
      const target = path.join(root, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(owner, file), target);
      fs.chmodSync(target, 0o644);
    }
    assert.equal(authenticateStarComponentReadSource(root).closureBodies, 415);
    run(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
void test("actual complete current source authenticates six production changes and the exact retained-law wrapper", () => {
  const result = authenticateStarComponentReadSource(owner);
  assert.equal(result.closureBodies, 415);
  assert.equal(result.changedBodies, 9);
  assert.equal(result.originalStarBodies, 322);
  assert.equal(result.originalJavaScriptBodies, 93);
  assert.equal(result.sourceOnly, true);
  assert.equal(result.nativeCredit, false);
  assert.equal(result.original400Credit, false);
  assert.equal(result.performanceCredit, false);
});
void test("every declared whole current body refuses physical mutation and omission", () => {
  capsule((root) => {
    for (const file of componentReadSourcePaths) {
      const target = path.join(root, file),
        body = fs.readFileSync(target);
      fs.writeFileSync(target, Buffer.concat([body, Buffer.from("\nforeign whole source\n")]));
      assert.throws(() => authenticateStarComponentReadSource(root), file);
      fs.unlinkSync(target);
      assert.throws(() => authenticateStarComponentReadSource(root), file);
      fs.writeFileSync(target, body, { mode: 0o644 });
    }
    assert(authenticateStarComponentReadSource(root));
  });
});
void test("whole old or mixed production bodies cannot stand in for the named current counterpart", () => {
  const data = JSON.parse(
    fs.readFileSync(
      path.join(owner, metadataDirectory + "empty-vue-star-component-read-cells.json"),
    ),
  );
  capsule((root) => {
    for (const row of data.changes) {
      const target = path.join(root, row.path),
        body = fs.readFileSync(target);
      fs.writeFileSync(target, Buffer.from(row.before.body, "base64"));
      assert.throws(() => authenticateStarComponentReadSource(root), row.path);
      fs.writeFileSync(target, body);
    }
  });
});
void test("metadata, executing helpers, modes and ancestor symlinks remain mandatory", () => {
  capsule((root) => {
    for (const file of authored) {
      const target = path.join(root, file),
        body = fs.readFileSync(target);
      fs.writeFileSync(target, Buffer.concat([body, Buffer.from("\nforeign metadata\n")]));
      assert.throws(() => authenticateStarComponentReadSource(root), file);
      fs.writeFileSync(target, body);
    }
    const file = componentReadSourcePaths[0],
      target = path.join(root, file);
    fs.chmodSync(target, 0o755);
    assert.throws(() => authenticateStarComponentReadSource(root));
    fs.chmodSync(target, 0o644);
    const directory = path.dirname(target),
      held = directory + "-held";
    fs.renameSync(directory, held);
    fs.symlinkSync(held, directory);
    assert.throws(() => authenticateStarComponentReadSource(root));
    fs.unlinkSync(directory);
    fs.renameSync(held, directory);
    assert(authenticateStarComponentReadSource(root));
  });
});
void test("foreign current commits and replacement refs refuse before any historical replay", () => {
  capsule((root) => {
    const head = joinGit(root, ["rev-parse", "HEAD"]).toString().trim();
    const base = "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38";
    fs.writeFileSync(path.join(root, ".git/HEAD"), base + "\n");
    assert.throws(() => authenticateStarComponentReadSource(root));
    fs.writeFileSync(path.join(root, ".git/HEAD"), head + "\n");
    joinGit(root, ["update-ref", "refs/replace/" + head, base]);
    assert.throws(() => authenticateStarComponentReadSource(root));
    joinGit(root, ["update-ref", "-d", "refs/replace/" + head]);
    assert(authenticateStarComponentReadSource(root));
  });
});
void test("a complete current-only private repository cannot fetch missing history through forged Actions", () => {
  capsule((root) => {
    fs.rmSync(path.join(root, ".git"), { recursive: true, force: true });
    joinGit(root, ["init", "--quiet"]);
    joinGit(root, ["add", "--", "."]);
    const tree = joinGit(root, ["write-tree"]).toString().trim();
    const head = joinGit(
      root,
      [
        "-c",
        "user.name=Private source control",
        "-c",
        "user.email=source@example.invalid",
        "commit-tree",
        tree,
      ],
      "complete private current source\n",
    )
      .toString()
      .trim();
    fs.writeFileSync(path.join(root, ".git/HEAD"), head + "\n");
    fs.writeFileSync(path.join(root, ".git/shallow"), head + "\n");
    joinGit(root, ["remote", "add", "origin", "https://github.com/ubugeeei-prod/vize.git"]);
    const state = () => ({
      head: fs.readFileSync(path.join(root, ".git/HEAD")),
      shallow: fs.readFileSync(path.join(root, ".git/shallow")),
      objects: joinGit(root, ["count-objects", "-v"]),
    });
    const before = state();
    const keys = ["GITHUB_ACTIONS", "GITHUB_REPOSITORY", "GITHUB_WORKSPACE"];
    const previous = keys.map((key) => process.env[key]);
    try {
      process.env.GITHUB_ACTIONS = "true";
      process.env.GITHUB_REPOSITORY = "ubugeeei-prod/vize";
      process.env.GITHUB_WORKSPACE = owner;
      assert.throws(() => prepareRootComponentReadHistory(root), /AssertionError/u);
      assert.equal(fs.existsSync(path.join(root, ".git/FETCH_HEAD")), false);
      assert.deepEqual(state(), before);
    } finally {
      keys.forEach((key, index) => {
        if (previous[index] === undefined) delete process.env[key];
        else process.env[key] = previous[index];
      });
    }
  });
});

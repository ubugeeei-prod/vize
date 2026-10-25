import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  cssCustodyRevisions,
  fetchCssCustodyRevisions,
  prepareCssCustodyHistory,
} from "../../tools/support/compat/github/prepare-css-custody-history.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const selected = ["tests/tooling/compiler-css-api-custody.test.mjs"];

function fixture() {
  const directory = mkdtempSync(join(tmpdir(), "vize-css-custody-history-"));
  const origin = join(directory, "origin.git");
  const seed = join(directory, "seed");
  const client = join(directory, "client");
  const git = (cwd, args, input) =>
    execFileSync("git", args, {
      cwd,
      input,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
      env: {
        ...process.env,
        GIT_AUTHOR_NAME: "Git transport control",
        GIT_AUTHOR_EMAIL: "git-control@example.invalid",
        GIT_COMMITTER_NAME: "Git transport control",
        GIT_COMMITTER_EMAIL: "git-control@example.invalid",
      },
    }).trim();
  mkdirSync(seed);
  git(directory, ["init", "--bare", origin]);
  git(seed, ["init"]);
  writeFileSync(join(seed, "source.txt"), "original\n");
  git(seed, ["add", "source.txt"]);
  const before = git(seed, ["commit-tree", git(seed, ["write-tree"])], "original\n");
  writeFileSync(join(seed, "source.txt"), "repaired\n");
  git(seed, ["add", "source.txt"]);
  const tree = git(seed, ["write-tree"]);
  const fix = git(seed, ["commit-tree", tree, "-p", before], "repair\n");
  const source = git(seed, ["commit-tree", tree], "squashed main\n");
  git(seed, ["remote", "add", "origin", origin]);
  git(seed, ["push", "origin", `${source}:refs/heads/main`, `${fix}:refs/pull/10696/head`]);
  git(directory, ["clone", "--single-branch", "--branch", "main", `file://${origin}`, client]);
  return { directory, client, before, fix, source, git };
}

await test("unrelated shards do not read fixtures or invoke Git", () => {
  assert.deepEqual(prepareCssCustodyHistory("/missing/repository", ["unrelated.test.mjs"]), {
    revisions: [],
    fetched: [],
  });
});

await test("selected CSS shards retain the exact authenticated producer revisions", () => {
  const fixtureRoot = "tests/_fixtures/differential/compiler";
  const manifest = JSON.parse(readFileSync(join(root, fixtureRoot, "manifest.json"), "utf8"));
  const expected = [
    ...new Set(
      manifest.cssApiCases.flatMap((row) => {
        const authority = JSON.parse(
          readFileSync(join(root, fixtureRoot, row.inputs.root, "authority.json"), "utf8"),
        );
        return [authority.beforeRevision, authority.fixRevision];
      }),
    ),
  ].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
  assert.deepEqual(cssCustodyRevisions(root, selected), expected);
  assert.deepEqual(
    cssCustodyRevisions(root, ["tests/tooling/compiler-css-binding-custody.test.mjs"]),
    expected,
  );
  const temporary = mkdtempSync(join(tmpdir(), "vize-css-custody-absent-"));
  try {
    assert.throws(() => cssCustodyRevisions(temporary, selected));
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

await test("a fresh squash clone recovers original and repaired bytes without changing HEAD", () => {
  const f = fixture();
  try {
    assert.notEqual(
      spawnSync("git", ["cat-file", "-e", `${f.fix}^{commit}`], { cwd: f.client }).status,
      0,
    );
    assert.deepEqual(fetchCssCustodyRevisions(f.client, [f.before, f.fix]), [f.before, f.fix]);
    assert.equal(f.git(f.client, ["show", `${f.before}:source.txt`]), "original");
    assert.equal(f.git(f.client, ["show", `${f.fix}:source.txt`]), "repaired");
    assert.equal(f.git(f.client, ["rev-parse", "HEAD"]), f.source);
    assert.equal(f.git(f.client, ["status", "--porcelain"]), "");
    f.git(f.client, ["remote", "remove", "origin"]);
    assert.deepEqual(fetchCssCustodyRevisions(f.client, [f.fix, f.before, f.fix]), []);
    assert.throws(() => fetchCssCustodyRevisions(f.client, ["0".repeat(40)]));
    assert.equal(f.git(f.client, ["rev-parse", "HEAD"]), f.source);
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});

await test("invalid revisions are rejected before repository or transport access", () => {
  for (const value of ["main", "--upload-pack=bad", "a".repeat(39), null, 1]) {
    assert.throws(
      () => fetchCssCustodyRevisions("/missing/repository", [value]),
      /immutable commit IDs/u,
    );
  }
});

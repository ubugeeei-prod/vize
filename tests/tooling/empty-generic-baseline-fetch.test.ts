import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fetchEmptyGenericBaseline } from "../../tools/support/compat/content-mapper/fetch-empty-generic-baseline.ts";

// Git transport controls, with no language-coverage credit.
function fixture() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-empty-baseline-"));
  const origin = path.join(directory, "origin.git");
  const seed = path.join(directory, "seed");
  const client = path.join(directory, "client");
  const git = (cwd: string, args: string[], input?: string) =>
    execFileSync("git", args, {
      cwd,
      input,
      encoding: "utf8",
      env: {
        ...process.env,
        GIT_AUTHOR_NAME: "Git control fixture",
        GIT_AUTHOR_EMAIL: "git-control@example.invalid",
        GIT_COMMITTER_NAME: "Git control fixture",
        GIT_COMMITTER_EMAIL: "git-control@example.invalid",
      },
    }).trim();
  fs.mkdirSync(seed);
  git(directory, ["init", "--bare", origin]);
  git(seed, ["init"]);
  fs.writeFileSync(path.join(seed, "source.txt"), "before\n");
  git(seed, ["add", "source.txt"]);
  const beforeTree = git(seed, ["write-tree"]);
  const baseline = git(seed, ["commit-tree", beforeTree], "before\n");
  fs.writeFileSync(path.join(seed, "source.txt"), "reviewed parent\n");
  git(seed, ["add", "source.txt"]);
  const afterTree = git(seed, ["write-tree"]);
  const retaining = git(seed, ["commit-tree", afterTree, "-p", baseline], "parent\n");
  // The same reviewed tree was squash-merged without the old ancestry.
  const source = git(seed, ["commit-tree", afterTree], "squashed main\n");
  git(seed, ["remote", "add", "origin", origin]);
  git(seed, ["push", "origin", source + ":refs/heads/main"]);
  git(seed, ["push", "origin", retaining + ":refs/pull/10558/head"]);
  git(directory, ["clone", "--single-branch", "--branch", "main", "file://" + origin, client]);
  const absent = spawnSync("git", ["cat-file", "-e", baseline + "^{commit}"], { cwd: client });
  assert.notEqual(absent.status, 0, "ordinary main clone must lack the before-image");
  return {
    baseline,
    retaining,
    source,
    beforeTree,
    client,
    seed,
    git,
    options: { repository: client, baseline, retaining, source },
    remove: () => fs.rmSync(directory, { recursive: true, force: true }),
  };
}

test("fetch restores the exact before-image without changing candidate HEAD", () => {
  const f = fixture();
  try {
    assert.deepEqual(fetchEmptyGenericBaseline(f.options), {
      schema: "vize.empty-generic-baseline-fetch",
      version: 1,
      source: f.source,
      baseline: f.baseline,
      retaining: f.retaining,
      retainingRef: "refs/pull/10558/head",
      baselineTree: f.beforeTree,
      candidateHeadUnchanged: true,
    });
    assert.equal(f.git(f.client, ["show", f.baseline + ":source.txt"]), "before");
    assert.equal(f.git(f.client, ["rev-parse", "HEAD"]), f.source);
  } finally {
    f.remove();
  }
});

test("a moved PR tip cannot substitute the reviewed retention authority", () => {
  const f = fixture();
  try {
    const replacement = f.git(
      f.seed,
      ["commit-tree", f.beforeTree, "-p", f.retaining],
      "changed tip\n",
    );
    f.git(f.seed, ["push", "origin", replacement + ":refs/pull/10558/head"]);
    assert.throws(() => fetchEmptyGenericBaseline(f.options), {
      message: "Retaining PR head differs from the frozen reviewed parent",
    });
  } finally {
    f.remove();
  }
});

test("candidate-as-baseline, missing PR ref and malformed or forged source fail closed", () => {
  const f = fixture();
  try {
    assert.throws(() => fetchEmptyGenericBaseline({ ...f.options, baseline: f.source }));
    assert.throws(() => fetchEmptyGenericBaseline({ ...f.options, baseline: "main" }), {
      message: "baseline must be a complete immutable commit",
    });
    assert.throws(() => fetchEmptyGenericBaseline({ ...f.options, source: f.retaining }), {
      message: "Candidate HEAD differs from the authenticated source",
    });
    f.git(f.seed, ["push", "origin", ":refs/pull/10558/head"]);
    assert.throws(() => fetchEmptyGenericBaseline(f.options));
    assert.equal(f.git(f.client, ["rev-parse", "HEAD"]), f.source);
  } finally {
    f.remove();
  }
});

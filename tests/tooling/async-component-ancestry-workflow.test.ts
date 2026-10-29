import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { selectFeatureDelta } from "../../tools/support/cross_file/async_component/feature-delta.ts";

function git(cwd: string, ...args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
}

function commit(cwd: string, subject: string): string {
  git(cwd, "add", ".");
  git(cwd, "commit", "-q", "-m", subject);
  return git(cwd, "rev-parse", "HEAD");
}

test("historical proof selects only the bounded feature delta after a main rebase", (context) => {
  const cwd = mkdtempSync(path.join(os.tmpdir(), "vize-async-feature-delta-"));
  context.after(() => rmSync(cwd, { recursive: true, force: true }));
  git(cwd, "init", "-q");
  git(cwd, "config", "user.name", "Vize Test");
  git(cwd, "config", "user.email", "vize-test@example.invalid");
  mkdirSync(path.join(cwd, "src"));
  writeFileSync(path.join(cwd, "src/base.txt"), "base\n");
  const base = commit(cwd, "test: establish base");

  git(cwd, "checkout", "-q", "-b", "feature");
  writeFileSync(path.join(cwd, "src/parent.txt"), "parent\n");
  const historicalParent = commit(cwd, "fix(test): establish historical production");
  git(cwd, "tag", "historical-parent", historicalParent);
  writeFileSync(path.join(cwd, "src/feature.txt"), "feature\n");
  const historicalFeature = commit(cwd, "fix(test): finish bounded feature");
  git(cwd, "tag", "historical-feature", historicalFeature);

  git(cwd, "checkout", "-q", "-b", "current-main", base);
  writeFileSync(path.join(cwd, "src/main.txt"), "unrelated main advance\n");
  commit(cwd, "fix(test): advance main independently");
  git(cwd, "checkout", "-q", "feature");
  git(cwd, "rebase", "-q", "current-main");
  writeFileSync(path.join(cwd, "src/later.txt"), "later repair\n");
  const landedFeature = commit(cwd, "fix(test): add a later repair");

  git(cwd, "checkout", "-q", "current-main");
  git(cwd, "merge", "--squash", "feature");
  const landedMerge = commit(cwd, "fix(test): land the complete feature");
  git(cwd, "checkout", "-q", "-b", "current-head");
  writeFileSync(path.join(cwd, "src/future.txt"), "later main advance\n");
  const head = commit(cwd, "fix(test): advance after the squash landing");

  const selected = selectFeatureDelta({
    cwd,
    historicalParent,
    historicalFeature,
    landedFeature,
    landedMerge,
    head,
    paths: ["src"],
  });
  assert.notEqual(selected.selectedParent, historicalParent);
  assert.notEqual(selected.selectedFeature, historicalFeature);
  assert.equal(selected.landedFeature, landedFeature);
  assert.equal(selected.landedMerge, landedMerge);
  assert.deepEqual(selected.changed, ["src/feature.txt"]);

  const contaminated = git(cwd, "diff", "--name-only", historicalParent, head, "--", "src")
    .split("\n")
    .filter(Boolean);
  assert.deepEqual(contaminated, [
    "src/feature.txt",
    "src/future.txt",
    "src/later.txt",
    "src/main.txt",
  ]);

  git(cwd, "checkout", "-q", "-b", "tampered-landing", landedMerge);
  writeFileSync(path.join(cwd, "src/feature.txt"), "tampered after review\n");
  const tamperedMerge = commit(cwd, "fix(test): tamper with the selected feature path");
  assert.throws(
    () =>
      selectFeatureDelta({
        cwd,
        historicalParent,
        historicalFeature,
        landedFeature,
        landedMerge: tamperedMerge,
        head: tamperedMerge,
        paths: ["src"],
      }),
    /landed squash must preserve every selected feature path byte-for-byte/,
  );
});

test("async ancestry workflow wires the bounded selector into its historical proof", () => {
  const workflow = readFileSync(
    path.join(process.cwd(), ".github/workflows/async-component-ancestry.yml"),
    "utf8",
  );
  assert.match(workflow, /fetch-depth: \$\{\{ matrix\.role == 'candidate' && '0' \|\| '1' \}\}/);
  assert.match(
    workflow,
    /node --test qualification\/tests\/tooling\/async-component-ancestry-workflow\.test\.ts/,
  );
  assert.match(
    workflow,
    /qualification\/tools\/support\/cross_file\/async_component\/feature-delta\.ts/,
  );
  assert.match(workflow, /refs\/pull\/11011\/head/);
  assert.match(workflow, /8a089d97ea3c95a23c588085f47502fa7d288bdf/);
  assert.match(workflow, /deda174ca6da0fc54a4f7483acbd3f71c382095c/);
  assert.match(workflow, /"featureDelta": delta/);
  assert.match(workflow, /\["git", "worktree", "add", "--detach"/);
  assert.match(workflow, /cwd=worktree/);
  assert.match(workflow, /"CARGO_TARGET_DIR": str\(root \/ "target"\)/);
  assert.doesNotMatch(workflow, /pathlib\.Path\(p\)\.write_bytes/);
  assert.doesNotMatch(
    workflow,
    /\["git", "diff", "--name-only", parent, head/,
    "a historical parent-to-head diff admits unrelated main and later feature changes",
  );
});

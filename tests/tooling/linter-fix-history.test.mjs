import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import {
  collectLinterHistory,
  inventoryTsv,
  writeInventory,
} from "../../tools/support/compat/fixtures/linter-fix-history.mjs";

const tool = fileURLToPath(
  new URL("../../tools/support/compat/fixtures/linter-fix-history.mjs", import.meta.url),
);
const scope = "crates/vize_patina";

function fixture(t) {
  const directory = mkdtempSync(resolve(tmpdir(), "vize-linter-history-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const git = (...args) =>
    execFileSync("git", args, {
      cwd: directory,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  git("init", "-b", "main");
  git("config", "user.name", "History Test");
  git("config", "user.email", "history-test@example.invalid");
  git("config", "commit.gpgsign", "false");
  const write = (path, content) => {
    mkdirSync(dirname(resolve(directory, path)), { recursive: true });
    writeFileSync(resolve(directory, path), content);
  };
  const commit = (subject) => {
    git("add", "-A");
    git("commit", "-m", subject);
    return git("rev-parse", "HEAD");
  };
  write(`${scope}/rule.rs`, "initial behavior\n");
  write(`${scope}/snapshots/rule.snap`, "old diagnostic\n");
  const initial = commit("feat(patina): first rule");
  return { directory, git, write, commit, initial };
}

void test("full history retains a discarded side fix and records merge parents separately", (t) => {
  const repo = fixture(t);
  repo.git("checkout", "-b", "side");
  repo.write(`${scope}/rule.rs`, "side behavior\n");
  const side = repo.commit("fix(patina): discarded branch requirement");
  repo.git("checkout", "main");
  repo.write(`${scope}/rule.rs`, "main behavior\n");
  const main = repo.commit("feat(patina): behavior without a fix title");
  assert.throws(() => repo.git("merge", "--no-ff", "side", "-m", "merge side"));
  repo.write(`${scope}/rule.rs`, "main behavior\n");
  const merged = repo.commit("merge side with main behavior");

  const inventory = collectLinterHistory({ cwd: repo.directory });
  assert.equal(inventory.revision, merged);
  assert.deepEqual(
    new Set(inventory.commits.map((row) => row.sha)),
    new Set([repo.initial, main, side]),
  );
  assert.equal(inventory.summary.titleFixCandidates, 1);
  assert.equal(inventory.summary.nonFixTitleCommits, 2);
  assert.equal(inventory.summary.unreviewedCommits, 3);
  assert.equal(inventory.summary.unreviewedMerges, 1);
  assert.deepEqual(inventory.merges[0].parents, [main, side]);
  assert.equal(inventory.merges[0].parentDeltas[0].scopedDiffBytes, 0);
  assert.ok(inventory.merges[0].parentDeltas[1].scopedDiffBytes > 0);
  assert.equal(inventory.summary.acceptedCorpusCases, 0);
  assert.equal(inventory.summary.executedTests, 0);
  assert.equal(inventory.summary.nativePasses, 0);
  assert.ok(
    [...inventory.commits, ...inventory.merges].every((row) => row.reviewState === "unreviewed"),
  );
});

void test("snapshot candidates bind actual Git blobs, including deleted snapshots", (t) => {
  const repo = fixture(t);
  repo.write(`${scope}/snapshots/rule.snap`, "new diagnostic\n");
  repo.write("outside.txt", "whole-commit context\n");
  const changed = repo.commit("fix(patina): update a snapshot candidate");
  repo.git("rm", `${scope}/snapshots/rule.snap`);
  const deleted = repo.commit("test(patina): remove the old expectation");
  repo.write(`${scope}/rule.rs`, "new behavior without any snapshot\n");
  const noSnapshot = repo.commit("fix(patina): no external snapshot witness");

  const inventory = collectLinterHistory({ cwd: repo.directory });
  const initial = inventory.commits.find((row) => row.sha === repo.initial);
  const update = inventory.commits.find((row) => row.sha === changed);
  const removal = inventory.commits.find((row) => row.sha === deleted);
  assert.equal(initial.snapshotReferenceCandidates[0].blobBeforeCommit, null);
  assert.equal(
    update.snapshotReferenceCandidates[0].blobBeforeCommit,
    initial.snapshotReferenceCandidates[0].blobAtCommit,
  );
  assert.notEqual(
    update.snapshotReferenceCandidates[0].blobAtCommit,
    initial.snapshotReferenceCandidates[0].blobAtCommit,
  );
  assert.equal(
    removal.snapshotReferenceCandidates[0].blobBeforeCommit,
    update.snapshotReferenceCandidates[0].blobAtCommit,
  );
  assert.equal(removal.snapshotReferenceCandidates[0].blobAtCommit, null);
  assert.equal(update.snapshotReferenceCandidates[0].blobAtRevision, null);
  assert.equal(update.snapshotReferenceCandidates[0].status, "candidate-only");
  assert.ok(update.touchedFiles.includes("outside.txt"));
  assert.notEqual(update.scopedDiffSha256, initial.scopedDiffSha256);
  assert.deepEqual(
    inventory.commits.find((row) => row.sha === noSnapshot).snapshotReferenceCandidates,
    [],
  );
});

void test("the pinned artifact is reproducible and TSV JSON cells preserve filenames", (t) => {
  const repo = fixture(t);
  const path = `${scope}/odd\tname\nfile.rs`;
  repo.write(path, "behavior\n");
  const sha = repo.commit("fix(patina): unusual path");
  const inventory = collectLinterHistory({ cwd: repo.directory, revision: sha });
  assert.deepEqual(inventory, collectLinterHistory({ cwd: repo.directory, revision: "HEAD" }));
  repo.git("config", "core.quotePath", "false");
  assert.deepEqual(inventory, collectLinterHistory({ cwd: repo.directory, revision: sha }));
  const cells = inventoryTsv(inventory)
    .trimEnd()
    .split("\n")[1]
    .split("\t")
    .map((cell) => JSON.parse(cell));
  assert.ok(cells[5].includes(path));
  const output = resolve(repo.directory, "artifacts");
  writeInventory(inventory, output);
  assert.deepEqual(JSON.parse(readFileSync(resolve(output, "history.json"), "utf8")), inventory);
  assert.equal(readFileSync(resolve(output, "history.tsv"), "utf8"), inventoryTsv(inventory));
  assert.match(
    readFileSync(resolve(output, "merges.tsv"), "utf8"),
    /^sha\tsubject\tparents\treviewState\tparentDeltas\n/,
  );
});

void test("a snapshot rename retains both paths without declaring witness continuity", (t) => {
  const repo = fixture(t);
  const previous = `${scope}/snapshots/rule.snap`;
  const next = `${scope}/snapshots/renamed.snap`;
  repo.git("mv", previous, next);
  const sha = repo.commit("refactor(patina): move a diagnostic snapshot");
  const inventory = collectLinterHistory({ cwd: repo.directory });
  const renamed = inventory.commits.find((row) => row.sha === sha);
  assert.deepEqual(new Set(renamed.touchedFiles), new Set([previous, next]));
  const removed = renamed.snapshotReferenceCandidates.find((row) => row.path === previous);
  const added = renamed.snapshotReferenceCandidates.find((row) => row.path === next);
  assert.equal(removed.blobAtCommit, null);
  assert.equal(added.blobBeforeCommit, null);
  assert.equal(removed.blobBeforeCommit, added.blobAtCommit);
  assert.equal(added.blobAtRevision, added.blobAtCommit);
  assert.equal(renamed.reviewState, "unreviewed");
  assert.equal(inventory.summary.acceptedCorpusCases, 0);
});

void test("incomplete history and invalid revisions fail before artifacts are written", (t) => {
  const repo = fixture(t);
  repo.write(`${scope}/rule.rs`, "second behavior\n");
  repo.commit("fix(patina): second commit");
  const shallow = resolve(repo.directory, "shallow");
  repo.git("clone", "--depth=1", `file://${repo.directory}`, shallow);
  assert.throws(() => collectLinterHistory({ cwd: shallow }), /non-shallow repository/);
  assert.throws(
    () => collectLinterHistory({ cwd: repo.directory, revision: "--all" }),
    /commit revision/,
  );
  assert.throws(() => collectLinterHistory({ cwd: repo.directory, revision: "missing-commit" }));
  const output = resolve(repo.directory, "invalid-output");
  assert.throws(() =>
    execFileSync(process.execPath, [tool, "missing-commit", output], {
      cwd: repo.directory,
      stdio: "pipe",
    }),
  );
  assert.throws(() => readFileSync(resolve(output, "history.json")), /ENOENT/);
});

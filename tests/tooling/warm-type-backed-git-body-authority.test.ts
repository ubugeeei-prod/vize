import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { gitBodyDigest } from "../performance/support/warm-type-backed-path-host.ts";
import {
  gitReaderHarnessPairs,
  gitReaderOriginalSource,
  qualifyGitReaderHarness,
} from "../performance/support/warm-type-backed-git-reader-harness.ts";

const sha256 = (body: string | Buffer) => createHash("sha256").update(body).digest("hex");

function withRepository(run: (root: string, git: (...args: string[]) => string) => void) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-git-body-authority-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  try {
    git("init", "--quiet");
    git("config", "user.name", "Source authority fixture");
    git("config", "user.email", "fixture@example.invalid");
    git("config", "commit.gpgsign", "false");
    git("config", "core.hooksPath", "/dev/null");
    run(root, git);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function commit(root: string, git: (...args: string[]) => string, body: string) {
  fs.writeFileSync(path.join(root, "entry.ts"), body);
  git("add", "entry.ts");
  git("commit", "--quiet", "-m", "source authority fixture");
  return git("rev-parse", "HEAD");
}

test("commit replacements cannot borrow another complete source body", () => {
  withRepository((root, git) => {
    const reviewedBody = "export const value = 1;\n";
    const actualBody = "export const value = 2;\n";
    const reviewed = commit(root, git, reviewedBody);
    const actual = commit(root, git, actualBody);
    assert.equal(gitBodyDigest(root, actual, "entry.ts"), sha256(actualBody));
    git("replace", actual, reviewed);
    assert.equal(git("show", `${actual}:entry.ts`), reviewedBody.trim());
    assert.equal(git("--no-replace-objects", "show", `${actual}:entry.ts`), actualBody.trim());
    assert.equal(gitBodyDigest(root, actual, "entry.ts"), sha256(actualBody));
    assert.notEqual(gitBodyDigest(root, actual, "entry.ts"), sha256(reviewedBody));
  });
});

test("blob replacements cannot borrow bytes after a genuine raw tree lookup", () => {
  withRepository((root, git) => {
    const reviewedBody = "export const value = 1;\n";
    const actualBody = "export const value = 2;\n";
    const reviewed = commit(root, git, reviewedBody);
    const actual = commit(root, git, actualBody);
    const reviewedBlob = git("rev-parse", `${reviewed}:entry.ts`);
    const actualBlob = git("rev-parse", `${actual}:entry.ts`);
    git("replace", actualBlob, reviewedBlob);
    assert.equal(git("cat-file", "blob", actualBlob), reviewedBody.trim());
    assert.equal(git("--no-replace-objects", "cat-file", "blob", actualBlob), actualBody.trim());
    assert.equal(gitBodyDigest(root, actual, "entry.ts"), sha256(actualBody));
    assert.notEqual(gitBodyDigest(root, actual, "entry.ts"), sha256(reviewedBody));
  });
});

const bodyPairs = new Map(
  gitReaderHarnessPairs.map(([file, before, after]) => [file, { before, after }]),
);
const changed = gitReaderHarnessPairs.slice(0, 3).map(([file]) => file);
const bodyDigest = (side: "before" | "after", file: string) => bodyPairs.get(file)![side];

test("the raw reader harness binds all six actual complete body pairs", () => {
  const root = path.resolve(import.meta.dirname, "../..");
  const afterSource = "293ec2db652d33f82fc77cca7c1966d1d602dde7";
  for (const revision of [gitReaderOriginalSource, afterSource]) {
    const present = () =>
      spawnSync("git", ["--no-replace-objects", "cat-file", "-e", `${revision}^{commit}`], {
        cwd: root,
      });
    const original = present();
    assert.equal(original.error, undefined);
    assert.equal(original.signal, null);
    if (original.status !== 0) {
      assert.ok([1, 128].includes(original.status!));
      execFileSync(
        "git",
        ["--no-replace-objects", "fetch", "--no-tags", "--depth=1", "origin", revision],
        { cwd: root },
      );
    }
    const prepared = present();
    assert.equal(prepared.error, undefined);
    assert.equal(prepared.signal, null);
    assert.equal(prepared.status, 0);
  }
  const actual = (side: "before" | "after", file: string) =>
    gitBodyDigest(root, side === "before" ? gitReaderOriginalSource : afterSource, file);
  const result = qualifyGitReaderHarness(changed, new Set(), actual);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, 6);
  assert.deepEqual(result.files, changed);
});

test("every source, absence and preserved whole law body remains mandatory", () => {
  for (const [file] of gitReaderHarnessPairs)
    for (const side of ["before", "after"] as const)
      assert.equal(
        qualifyGitReaderHarness(changed, new Set(), (candidate, name) =>
          candidate === side && name === file ? "f".repeat(64) : bodyDigest(candidate, name),
        ),
        null,
        `${side}:${file}`,
      );
});

test("unknown, omitted and duplicate harness changes cannot borrow this repair", () => {
  assert.equal(qualifyGitReaderHarness([...changed, "unknown.ts"], new Set(), bodyDigest), null);
  assert.equal(qualifyGitReaderHarness([...changed, changed[0]], new Set(), bodyDigest), null);
  for (const file of changed)
    assert.equal(
      qualifyGitReaderHarness(
        changed.filter((name) => name !== file),
        new Set(),
        bodyDigest,
      ),
      null,
      file,
    );
  assert.equal(qualifyGitReaderHarness([], new Set(), bodyDigest), null);
  const inherited = new Set(["already-owned.ts"]);
  assert.ok(qualifyGitReaderHarness([...changed, ...inherited], inherited, bodyDigest));
});

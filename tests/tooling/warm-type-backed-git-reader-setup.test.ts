import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { test } from "node:test";
import {
  gitReaderNoDeepChanges,
  gitReaderNoDeepPairs,
  prepareLiteralSource,
  qualifyGitReaderNoDeepHarness,
} from "../performance/support/warm-type-backed-git-reader-nodeep.ts";
import {
  gitReaderSetupChanges,
  gitReaderSetupPairs,
  gitReaderSetupPrevious,
  gitReaderSetupPreviousDeltas,
  gitReaderSetupSource,
  qualifyGitReaderSetupHarness,
  rawFirstParentProof,
} from "../performance/support/warm-type-backed-git-reader-setup.ts";

const root = path.resolve(import.meta.dirname, "../..");
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
function rawBody(source: string, file: string): Buffer | null {
  assert.match(source, /^[a-f0-9]{40}$/u);
  const git = (args: string[]) =>
    execFileSync("git", ["--no-replace-objects", ...args], { cwd: root });
  const entry = git(["ls-tree", "-z", source, "--", file]);
  if (entry.length === 0) return null;
  const text = entry.toString("utf8");
  assert.ok(Buffer.from(text).equals(entry));
  const match = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(text);
  assert.ok(match, `one regular literal source blob required: ${file}`);
  assert.equal(match[2], file);
  return git(["cat-file", "blob", match[1]]);
}
const actual = (source: string, file: string) => {
  const body = rawBody(source, file);
  return body === null ? null : sha(body);
};
const pairs = new Map(
  gitReaderSetupPairs.map(([file, before, after]) => [file, { before, after }]),
);
const digest = (side: "before" | "after", file: string) => pairs.get(file)![side];

test("the entire original102 still qualifies only its authentic historical source pair", () => {
  prepareLiteralSource(root, gitReaderSetupSource);
  prepareLiteralSource(root, gitReaderSetupPrevious);
  for (const [file, before, after] of gitReaderNoDeepPairs) {
    assert.equal(actual(gitReaderSetupSource, file), before, `original before: ${file}`);
    assert.equal(actual(gitReaderSetupPrevious, file), after, `original after: ${file}`);
  }
  const old = qualifyGitReaderNoDeepHarness(gitReaderNoDeepChanges, new Set(), (side, file) =>
    actual(side === "before" ? gitReaderSetupSource : gitReaderSetupPrevious, file),
  );
  assert.ok(old);
  assert.equal(old.completeBodyPairs, 102);
  assert.equal(old.acceptanceCredit, "none");
  for (const [file, before, previous] of gitReaderSetupPreviousDeltas) {
    assert.equal(actual(gitReaderSetupSource, file), before, `additional original before: ${file}`);
    assert.equal(actual(gitReaderSetupPrevious, file), previous, `complete previous body: ${file}`);
  }
});

test("every complete successor before and after side refuses changed or absent source", () => {
  const complete = qualifyGitReaderSetupHarness(gitReaderSetupChanges, new Set(), digest);
  assert.ok(complete);
  assert.equal(complete.preservedPreviousPairs, 102);
  assert.equal(complete.acceptanceCredit, "none");
  for (const [file] of gitReaderSetupPairs)
    for (const side of ["before", "after"] as const)
      for (const changed of ["f".repeat(64), null]) {
        if (changed === digest(side, file)) continue;
        assert.equal(
          qualifyGitReaderSetupHarness(gitReaderSetupChanges, new Set(), (s, name) =>
            s === side && name === file ? changed : digest(s, name),
          ),
          null,
          `${side}:${file}`,
        );
      }
});

test("partial, duplicate, unknown and inherited family entries cannot borrow the successor", () => {
  assert.equal(qualifyGitReaderSetupHarness([], new Set(), digest), null);
  assert.equal(
    qualifyGitReaderSetupHarness([...gitReaderSetupChanges, "unknown.ts"], new Set(), digest),
    null,
  );
  for (const file of gitReaderSetupChanges) {
    assert.equal(
      qualifyGitReaderSetupHarness(
        gitReaderSetupChanges.filter((name) => name !== file),
        new Set([file]),
        digest,
      ),
      null,
      file,
    );
    assert.equal(
      qualifyGitReaderSetupHarness([...gitReaderSetupChanges, file], new Set(), digest),
      null,
      file,
    );
  }
  const family = "crates/vize_patina/src/linter/script_rules.rs";
  assert.equal(
    qualifyGitReaderSetupHarness([...gitReaderSetupChanges, family], new Set([family]), digest),
    null,
  );
  assert.ok(
    qualifyGitReaderSetupHarness(
      [...gitReaderSetupChanges, "unrelated-qualified.ts"],
      new Set(["unrelated-qualified.ts"]),
      digest,
    ),
  );
});

const rawGit = (cwd: string, args: string[], input?: string) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd, input });

test("raw first parents prove shallow ancestry, exact missing hydration and finite refusals", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reader-ancestry-"));
  const git = (args: string[], input?: string) => rawGit(fixture, args, input);
  let serial = 0;
  try {
    git(["init", "--quiet"]);
    const tree = git(["mktree"], "").toString().trim();
    const commit = (parent?: string) =>
      git(
        [
          "-c",
          "user.name=Fixture",
          "-c",
          "user.email=fixture@example.invalid",
          "commit-tree",
          tree,
          ...(parent ? ["-p", parent] : []),
        ],
        `raw source ${serial++} 日本語\n`,
      )
        .toString()
        .trim();
    const prior = commit();
    const base = commit(prior);
    const middle = commit(base);
    const head = commit(middle);
    const foreign = commit();
    git(["update-ref", "refs/heads/main", head]);
    git(["update-ref", "refs/heads/foreign", foreign]);
    git(["symbolic-ref", "HEAD", "refs/heads/main"]);
    const reader = path.join(fixture, "reader");
    fs.mkdirSync(reader);
    rawGit(reader, ["init", "--quiet"]);
    rawGit(reader, ["remote", "add", "origin", fixture]);
    rawGit(reader, ["fetch", "--no-tags", "--depth=1", "origin", head]);
    prepareLiteralSource(reader, base);
    const missing = (source: string) =>
      rawGit(
        reader,
        ["cat-file", "--batch-check=%(objectname) %(objecttype)"],
        source + "\n",
      ).toString();
    assert.equal(missing(middle), middle + " missing\n");
    assert.throws(() => rawGit(reader, ["merge-base", "--is-ancestor", base, head]), { status: 1 });
    assert.deepEqual(rawFirstParentProof(reader, base, head), [head, middle, base]);
    assert.equal(missing(prior), prior + " missing\n");
    assert.equal(missing(foreign), foreign + " missing\n");
    assert.throws(() => rawGit(reader, ["merge-base", "--is-ancestor", base, head]), { status: 1 });
    const shallow = path.join(fixture, ".git/shallow");
    git(["merge-base", "--is-ancestor", base, head]);
    fs.writeFileSync(shallow, head + "\n");
    const marker = fs.readFileSync(shallow);
    assert.throws(() => git(["merge-base", "--is-ancestor", base, head]), { status: 1 });
    assert.deepEqual(rawFirstParentProof(fixture, base, head), [head, middle, base]);
    assert.deepEqual(fs.readFileSync(shallow), marker);
    assert.throws(() => rawFirstParentProof(fixture, foreign, head), /disconnected/u);
    git(["replace", foreign, head]);
    execFileSync("git", ["merge-base", "--is-ancestor", base, foreign], { cwd: fixture });
    assert.throws(() => rawFirstParentProof(fixture, base, foreign), /disconnected/u);
    let long = base;
    for (let count = 0; count < 63; count++) long = commit(long);
    assert.equal(rawFirstParentProof(fixture, base, long).length, 64);
    assert.throws(() => rawFirstParentProof(fixture, base, commit(long)), /exceeds 64/u);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

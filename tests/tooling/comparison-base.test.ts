import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { test } from "node:test";
import { comparisonBase } from "../../tools/support/compat/github/comparison-base.mjs";
import { changedPaths } from "../../tools/support/compat/github/plan-source-checks.mjs";

const eventBase = "1".repeat(40);
const newBase = "2".repeat(40);
const prHead = "3".repeat(40);
const head = "4".repeat(40);
const input = {
  event: "pull_request",
  eventBase,
  prHead,
  head,
  checkedOutHead: head,
  commit: `tree ${"5".repeat(40)}\nparent ${newBase}\nparent ${prHead}\n\nMerge candidate`,
};

void test("only the verified two-parent tested merge replaces a stale event base", () => {
  assert.deepEqual(comparisonBase(input), {
    base: newBase,
    reason: "verified pull request merge parent",
  });
  for (const candidate of [
    { ...input, checkedOutHead: newBase },
    { ...input, prHead: newBase },
    { ...input, commit: `tree ${newBase}\n\nNonmerge` },
    { ...input, commit: `parent ${newBase}\n\nSingle parent` },
    { ...input, commit: `parent ${newBase}\nparent ${prHead}\nparent ${head}\n\nOctopus` },
    { ...input, commit: `parent malformed\nparent ${prHead}\n\nInvalid header` },
  ]) {
    assert.deepEqual(comparisonBase(candidate), {
      base: eventBase,
      reason: "unverified merge: conservative event base",
    });
  }
  assert.deepEqual(comparisonBase({ ...input, event: "merge_group", prHead: undefined }), {
    base: eventBase,
    reason: "merge group event base",
  });
});

void test("comparison identity rejects incomplete SHAs and unrecognized planning contexts", () => {
  for (const field of ["eventBase", "head", "checkedOutHead", "prHead"]) {
    assert.throws(() => comparisonBase({ ...input, [field]: "short" }), /Expected full/);
    assert.throws(() => comparisonBase({ ...input, [field]: undefined }), /Expected full/);
  }
  assert.throws(() => comparisonBase({ ...input, event: "push" }), /Unexpected comparison/);
});

void test("a real shallow merge checkout excludes newer main changes from the tested comparison", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-comparison-base-"));
  const origin = join(directory, "origin");
  const shallow = join(directory, "shallow");
  const git = (cwd, ...args) =>
    execFileSync("git", args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
  try {
    git(directory, "init", "-q", "-b", "main", origin);
    git(origin, "config", "user.name", "CI Test");
    git(origin, "config", "user.email", "ci@example.invalid");
    writeFileSync(join(origin, "base.md"), "base\n");
    git(origin, "add", ".");
    git(origin, "commit", "-qm", "base");
    const staleBase = git(origin, "rev-parse", "HEAD");
    git(origin, "checkout", "-qb", "feature");
    writeFileSync(join(origin, "feature.mjs"), "export const selected = true;\n");
    git(origin, "add", ".");
    git(origin, "commit", "-qm", "feature");
    const feature = git(origin, "rev-parse", "HEAD");
    git(origin, "checkout", "-q", "main");
    writeFileSync(join(origin, "release.json"), '{"version":"0.429.1"}\n');
    git(origin, "add", ".");
    git(origin, "commit", "-qm", "release advanced after event payload");
    const testedBase = git(origin, "rev-parse", "HEAD");
    git(origin, "merge", "--no-ff", "-qm", "tested merge", "feature");
    const testedMerge = git(origin, "rev-parse", "HEAD");
    git(directory, "clone", "-q", "--depth=1", `file://${origin}`, shallow);
    const missingParent = spawnSync("git", ["cat-file", "-e", `${testedBase}^{commit}`], {
      cwd: shallow,
    });
    assert.notEqual(missingParent.status, 0, "parent must not exist in the shallow checkout");
    const output = join(directory, "outputs.txt");
    const script = resolve("tools/support/compat/github/comparison-base.mjs");
    const result = execFileSync(process.execPath, [script], {
      cwd: shallow,
      encoding: "utf8",
      env: {
        ...process.env,
        GITHUB_EVENT_NAME: "pull_request",
        EVENT_BASE_SHA: staleBase,
        GITHUB_SHA: testedMerge,
        PR_HEAD_SHA: feature,
        GITHUB_OUTPUT: output,
        GITHUB_STEP_SUMMARY: join(directory, "summary.md"),
      },
    }).trim();
    assert.equal(result, testedBase);
    assert.equal(readFileSync(output, "utf8"), `base=${testedBase}\n`);
    git(shallow, "fetch", "--no-tags", "--depth=1", "origin", result);
    assert.deepEqual(changedPaths(result, testedMerge, shallow), ["feature.mjs"]);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

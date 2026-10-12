import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { ensureHostedLiteralSource } from "../performance/support/warm-type-backed-git-reader-literal-source.ts";

const missing = "f".repeat(40);
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
function environment() {
  const env = { ...process.env, GIT_ALLOW_PROTOCOL: "", GIT_NO_LAZY_FETCH: "1" };
  for (const key of Object.keys(env))
    if (key.startsWith("GIT_") && !["GIT_ALLOW_PROTOCOL", "GIT_NO_LAZY_FETCH"].includes(key))
      delete env[key];
  return env;
}
const git = (root: string, args: string[], input?: string) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root, input, env: environment() });
function snapshot(root: string): Array<[string, number, string]> {
  const rows: Array<[string, number, string]> = [];
  const walk = (directory: string) => {
    for (const name of fs.readdirSync(directory).sort()) {
      const file = path.join(directory, name);
      const stat = fs.lstatSync(file);
      if (stat.isDirectory()) walk(file);
      else
        rows.push([
          path.relative(root, file),
          stat.mode,
          stat.isSymbolicLink() ? fs.readlinkSync(file) : digest(fs.readFileSync(file)),
        ]);
    }
  };
  walk(root);
  return rows;
}
function fixture(run: (root: string, source: string) => void) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-literal-git-"));
  const original = { ...process.env };
  try {
    git(root, ["init", "--quiet"]);
    const tree = git(root, ["mktree"], "").toString().trim();
    const source = git(
      root,
      ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit-tree", tree],
      "owned raw source\n",
    )
      .toString()
      .trim();
    git(root, ["update-ref", "refs/heads/fixture", source]);
    git(root, ["symbolic-ref", "HEAD", "refs/heads/fixture"]);
    for (const key of Object.keys(process.env))
      if (key.startsWith("GIT_") || key.startsWith("GITHUB_")) delete process.env[key];
    process.env.GIT_ALLOW_PROTOCOL = "";
    run(root, source);
  } finally {
    for (const key of Object.keys(process.env)) if (!(key in original)) delete process.env[key];
    Object.assign(process.env, original);
    fs.rmSync(root, { recursive: true, force: true });
  }
}
function refusedUnchanged(root: string, source: string) {
  const before = snapshot(root);
  assert.throws(() => ensureHostedLiteralSource(root, source));
  assert.deepEqual(snapshot(root), before, "refusal must precede any private repository mutation");
}

void test("existing raw commit fastpath needs neither Actions nor origin and ignores replacements", () => {
  fixture((root, source) => {
    const tree = git(root, ["mktree"], "").toString().trim();
    const replacement = git(
      root,
      ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit-tree", tree],
      "foreign replacement\n",
    )
      .toString()
      .trim();
    git(root, ["replace", source, replacement]);
    const before = snapshot(root);
    ensureHostedLiteralSource(root, source);
    assert.deepEqual(snapshot(root), before);
    const body = git(root, ["cat-file", "commit", source]);
    assert(body.toString().endsWith("owned raw source\n"));
    assert.equal(
      createHash("sha1").update(`commit ${body.length}\0`).update(body).digest("hex"),
      source,
    );
  });
});

void test("missing, abbreviated, symbolic and noncommit literals refuse before mutation", () => {
  fixture((root) => {
    const blob = git(root, ["hash-object", "-w", "--stdin"], "not a commit\n").toString().trim();
    const tree = git(root, ["mktree"], "").toString().trim();
    for (const value of [missing, "HEAD", "f".repeat(39), "F".repeat(40), blob, tree])
      refusedUnchanged(root, value);
  });
});

void test("private missing source refuses absent and foreign origins with forged Actions environment", () => {
  fixture((root) => {
    process.env.GITHUB_ACTIONS = "true";
    process.env.GITHUB_REPOSITORY = "ubugeeei-prod/vize";
    process.env.GITHUB_WORKSPACE = root;
    refusedUnchanged(root, missing);
    for (const origin of [
      "file:///unavailable/vize.git",
      "https://github.com/foreign/vize.git",
      "https://github.com/ubugeeei-prod/vize.git",
    ]) {
      git(root, ["config", "remote.origin.url", origin]);
      refusedUnchanged(root, missing);
    }
    process.env.GITHUB_REPOSITORY = "foreign/vize";
    refusedUnchanged(root, missing);
    process.env.GITHUB_WORKSPACE = path.dirname(root);
    refusedUnchanged(root, missing);
  });
});

void test("existing worktree source reads remain inert but missing source cannot mutate shared storage", () => {
  fixture((root, source) => {
    const linked = path.join(root, "linked");
    git(root, ["worktree", "add", "--quiet", "--detach", linked, source]);
    assert(fs.lstatSync(path.join(linked, ".git")).isFile());
    const before = snapshot(root);
    ensureHostedLiteralSource(linked, source);
    assert.deepEqual(snapshot(root), before);
    process.env.GITHUB_ACTIONS = "true";
    process.env.GITHUB_REPOSITORY = "ubugeeei-prod/vize";
    process.env.GITHUB_WORKSPACE = linked;
    refusedUnchanged(linked, missing);
    assert.deepEqual(snapshot(root), before);
  });
});

void test("private shared object and dangling alias layouts cannot turn fake Actions into a fetch", () => {
  fixture((root) => {
    process.env.GITHUB_ACTIONS = "true";
    process.env.GITHUB_REPOSITORY = "ubugeeei-prod/vize";
    process.env.GITHUB_WORKSPACE = root;
    const alternates = path.join(root, ".git/objects/info/alternates");
    fs.symlinkSync(path.join(root, "absent"), alternates);
    refusedUnchanged(root, missing);
    fs.unlinkSync(alternates);
    const shared = path.join(root, "shared");
    fs.mkdirSync(shared);
    git(shared, ["init", "--quiet"]);
    fs.writeFileSync(alternates, path.join(shared, ".git/objects") + "\n");
    refusedUnchanged(root, missing);
  });
});

void test("private shallow no-origin checkout keeps existing source and missing refusal entirely inert", () => {
  fixture((root, source) => {
    const shallow = path.join(root, ".git/shallow");
    fs.writeFileSync(shallow, `${source}\n`, { flag: "wx", mode: 0o644 });
    assert.equal(git(root, ["rev-parse", "--is-shallow-repository"]).toString(), "true\n");
    const before = snapshot(root);
    ensureHostedLiteralSource(root, source);
    assert.deepEqual(snapshot(root), before);
    refusedUnchanged(root, missing);
    assert.deepEqual(fs.readFileSync(shallow), Buffer.from(`${source}\n`));
    assert.deepEqual(snapshot(root), before);
  });
});

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { prepareLiteralSource } from "./warm-type-backed-git-reader-nodeep.ts";

const git = (root: string, args: string[], input?: string) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    input,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
  }).toString("utf8");

/** Provision literal commits only in the independently owned Linux Actions checkout. */
export function ensureHostedLiteralSource(root: string, source: string): void {
  assert.match(source, /^[a-f0-9]{40}$/u);
  const lookup = () =>
    git(root, ["cat-file", "--batch-check=%(objectname) %(objecttype)"], `${source}\n`);
  const found = lookup();
  if (found === `${source} commit\n`) return;
  assert.equal(found, `${source} missing\n`, "literal source must be a raw commit");
  assert.equal(process.env.GITHUB_ACTIONS, "true", "missing literal requires Actions");
  assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert(process.env.GITHUB_WORKSPACE, "missing literal requires the owned workspace");
  const actual = fs.realpathSync(root);
  assert.equal(actual, fs.realpathSync(process.env.GITHUB_WORKSPACE));
  for (const key of [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
  ])
    assert.equal(process.env[key], undefined, `no repository override: ${key}`);
  assert.equal(fs.realpathSync(git(root, ["rev-parse", "--show-toplevel"]).trim()), actual);
  const directory = path.join(actual, ".git");
  assert(fs.lstatSync(directory).isDirectory(), "owned .git must be a physical directory");
  assert.equal(fs.realpathSync(directory), directory);
  assert.equal(
    fs.realpathSync(path.resolve(actual, git(root, ["rev-parse", "--git-common-dir"]).trim())),
    directory,
  );
  for (const suffix of ["objects", "objects/info"]) {
    const target = path.join(directory, suffix);
    assert(fs.lstatSync(target).isDirectory() && fs.realpathSync(target) === target);
  }
  assert(!fs.readdirSync(directory).includes("commondir"), "no shared Git directory");
  assert(
    !fs.readdirSync(path.join(directory, "objects/info")).includes("alternates"),
    "no shared object storage",
  );
  const origin = git(root, ["remote", "get-url", "--all", "origin"]);
  assert.match(origin, /^https:\/\/github\.com\/ubugeeei-prod\/vize(?:\.git)?\n$/u);
  assert.equal(git(root, ["config", "--get-all", "remote.origin.url"]), origin);
  assert.equal(process.platform, "linux", "missing literal requires the Linux Actions checkout");
  assert.match(actual, /^\/home\/runner\/(?:_work|work)\/vize\/vize$/u);
  prepareLiteralSource(root, source);
  assert.equal(lookup(), `${source} commit\n`, "complete literal fetch result");
}

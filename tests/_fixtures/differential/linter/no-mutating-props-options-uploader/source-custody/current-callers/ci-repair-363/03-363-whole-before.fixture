import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { prepareLiteralSource } from "../../performance/support/warm-type-backed-git-reader-nodeep.ts";

const pools = new Map();
const rawGit = (root, args) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root });
const approved = new Set([
  "2489128d5f8fab1b7442a1e5c09b4e1701cbf185",
  "eafe5bed4f602e0787ff9f2f48ec4155d115ab7f",
]);
process.once("exit", () => {
  for (const owned of pools.values()) fs.rmSync(owned, { recursive: true, force: true });
});

/** Exact missing-source hydration is confined to private metadata and a private object store. */
export function privateNoMutatingCallerGit(root, source) {
  assert(approved.has(source), "only the two declared complete historical source commits");
  for (const name of [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_OBJECT_DIRECTORY",
    "GIT_INDEX_FILE",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
  ])
    assert(!process.env[name], "private Git metadata refuses inherited path override: " + name);
  const realRoot = fs.realpathSync(root);
  const key = realRoot + "\0" + source;
  if (!pools.has(key)) {
    const common = rawGit(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    const objects = fs.realpathSync(path.resolve(root, common, "objects"));
    assert(!/[\r\n]/u.test(objects));
    const origin = rawGit(root, ["remote", "get-url", "origin"]).toString().trim();
    assert(origin && !/[\r\n]/u.test(origin));
    const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-no-mutating-caller-private-git-"));
    fs.mkdirSync(path.join(owned, ".git/objects/info"), { recursive: true });
    fs.mkdirSync(path.join(owned, ".git/refs"));
    fs.writeFileSync(path.join(owned, ".git/HEAD"), source + "\n");
    fs.writeFileSync(path.join(owned, ".git/objects/info/alternates"), objects + "\n");
    fs.writeFileSync(
      path.join(owned, ".git/config"),
      '[core]\nrepositoryformatversion = 0\nbare = false\n[remote "origin"]\nurl = ' +
        JSON.stringify(origin) +
        "\n",
    );
    pools.set(key, owned);
  }
  const owned = pools.get(key);
  prepareLiteralSource(owned, source);
  assert.equal(rawGit(owned, ["rev-parse", "HEAD"]).toString().trim(), source);
  return owned;
}

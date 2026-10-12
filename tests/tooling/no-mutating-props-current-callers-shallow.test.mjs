import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { copyNoMutatingCallerShallowBoundary } from "./support/no-mutating-props-current-caller-git.mjs";

const git = (cwd, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd, input });
function metadata(owned, source, objects, origin) {
  fs.mkdirSync(path.join(owned, ".git/objects/info"), { recursive: true });
  fs.mkdirSync(path.join(owned, ".git/refs"));
  fs.writeFileSync(path.join(owned, ".git/HEAD"), source + "\n");
  fs.writeFileSync(path.join(owned, ".git/objects/info/alternates"), objects + "\n");
  fs.writeFileSync(
    path.join(owned, ".git/config"),
    '[core]\nrepositoryformatversion=0\nbare=false\n[remote "origin"]\nurl=' +
      JSON.stringify(origin) +
      "\n",
  );
}

void test("real shallow alternates refuse without boundary and the complete two-hop caller fetch preserves shared source", () => {
  const owned = fs.mkdtempSync(path.join(os.tmpdir(), "vize-no-mutating-authentic-shallow-"));
  const origin = path.join(owned, "origin");
  fs.mkdirSync(origin);
  try {
    git(origin, ["init", "--quiet"]);
    const commits = [];
    for (const text of ["original source\n", "second source\n", "shallow source\n"]) {
      const blob = git(origin, ["hash-object", "-w", "--stdin"], text).toString().trim();
      const tree = git(origin, ["mktree"], `100644 blob ${blob}\tsource.rs\n`).toString().trim();
      const args = [
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit-tree",
        tree,
      ];
      if (commits.length) args.push("-p", commits.at(-1));
      commits.push(git(origin, args, text).toString().trim());
    }
    git(origin, ["update-ref", "refs/heads/main", commits[2]]);
    git(origin, ["symbolic-ref", "HEAD", "refs/heads/main"]);
    const url = pathToFileURL(origin).href;
    const shallow = path.join(owned, "shallow");
    git(owned, ["clone", "--quiet", "--no-checkout", "--depth=1", url, shallow]);
    const originalBoundary = fs.readFileSync(path.join(shallow, ".git/shallow"));
    const originalRefs = git(shallow, ["show-ref"]);
    assert.equal(git(shallow, ["rev-parse", "--is-shallow-repository"]).toString().trim(), "true");
    assert.equal(
      spawnSync("git", ["--no-replace-objects", "cat-file", "-e", commits[1]], { cwd: shallow })
        .status,
      1,
    );
    const bad = path.join(owned, "bad");
    fs.mkdirSync(bad);
    metadata(bad, commits[0], path.join(shallow, ".git/objects"), url);
    const failed = spawnSync(
      "git",
      ["--no-replace-objects", "fetch", "--no-tags", "--depth=1", "origin", commits[0]],
      { cwd: bad, encoding: "utf8" },
    );
    assert.notEqual(failed.status, 0);
    assert.match(failed.stderr, /Could not read|bad object|did not send/u);
    const pool = path.join(owned, "pool");
    fs.mkdirSync(pool);
    metadata(pool, commits[0], path.join(shallow, ".git/objects"), url);
    const copied = copyNoMutatingCallerShallowBoundary(shallow, pool);
    assert.deepEqual(copied.boundaries, [commits[2]]);
    git(pool, ["fetch", "--no-tags", "--depth=1", "origin", commits[0]]);
    const raw = git(pool, ["cat-file", "commit", commits[0]]);
    assert.equal(
      createHash("sha1").update(`commit ${raw.length}\0`).update(raw).digest("hex"),
      commits[0],
    );
    const child = path.join(owned, "historical-caller");
    fs.mkdirSync(child);
    metadata(child, commits[0], path.join(pool, ".git/objects"), url);
    copyNoMutatingCallerShallowBoundary(pool, child);
    const module =
      'import assert from "node:assert/strict";\nimport {execFileSync} from "node:child_process";\nimport {createHash} from "node:crypto";\nimport {test} from "node:test";\n' +
      'void test("whole shallow historical caller",()=>{const g=(...args)=>execFileSync("git",["--no-replace-objects",...args]);g("fetch","--no-tags","--depth=1","origin",' +
      JSON.stringify(commits[1]) +
      ');const b=g("cat-file","commit",' +
      JSON.stringify(commits[1]) +
      ');assert.equal(createHash("sha1").update(`commit ${b.length}\\0`).update(b).digest("hex"),' +
      JSON.stringify(commits[1]) +
      ");});\n";
    fs.writeFileSync(path.join(child, "whole-caller.test.mjs"), module);
    const env = { ...process.env };
    delete env.NODE_TEST_CONTEXT;
    const result = spawnSync(
      process.execPath,
      ["--test", "--test-reporter=tap", "whole-caller.test.mjs"],
      { cwd: child, env, encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stdout + result.stderr);
    for (const line of ["# tests 1", "# pass 1", "# fail 0", "# skipped 0"])
      assert(result.stdout.split("\n").includes(line));
    assert.deepEqual(fs.readFileSync(path.join(shallow, ".git/shallow")), originalBoundary);
    assert.deepEqual(git(shallow, ["show-ref"]), originalRefs);
  } finally {
    fs.rmSync(owned, { recursive: true, force: true });
  }
});

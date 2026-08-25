import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  assertHydratedGitlinkFixtures,
  fixtureHydrationFailures,
} from "../../tools/davinci/lib/corpus-hydration.mjs";

function git(cwd: string, args: string[]): string {
  return execFileSync("git", args, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
  }).trim();
}

function createRepository(directory: string, marker: string): string {
  fs.mkdirSync(directory, { recursive: true });
  git(directory, ["init", "-q"]);
  git(directory, ["config", "user.name", "Fixture"]);
  git(directory, ["config", "user.email", "fixture@example.com"]);
  fs.writeFileSync(path.join(directory, "README.md"), `${marker}\n`);
  git(directory, ["add", "README.md"]);
  git(directory, ["commit", "-qm", `fixture ${marker}`]);
  return git(directory, ["rev-parse", "HEAD"]);
}

function pinGitlink(root: string, relPath: string, revision: string): void {
  git(root, ["update-index", "--add", "--cacheinfo", `160000,${revision},${relPath}`]);
}

test("corpus hydration preflight catches missing, empty, and wrong fixture gitlinks", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-corpus-hydration-"));
  try {
    git(root, ["init", "-q"]);
    const okRevision = createRepository(path.join(root, "fixtures", "ok"), "ok");
    const wrongActualRevision = createRepository(path.join(root, "fixtures", "wrong"), "wrong");
    const wrongExpectedRevision = createRepository(path.join(root, "expected-wrong"), "expected");
    fs.mkdirSync(path.join(root, "fixtures", "empty"), { recursive: true });

    pinGitlink(root, "fixtures/ok", okRevision);
    pinGitlink(root, "fixtures/missing", okRevision);
    pinGitlink(root, "fixtures/empty", okRevision);
    pinGitlink(root, "fixtures/wrong", wrongExpectedRevision);

    assert.notEqual(wrongActualRevision, wrongExpectedRevision);
    assert.deepEqual(fixtureHydrationFailures(root, ["fixtures/ok"]), []);

    const failures = fixtureHydrationFailures(root, [
      "fixtures/ok",
      "fixtures/missing",
      "fixtures/empty",
      "fixtures/wrong",
      "fixtures/not-gitlink",
      "../escape",
      "/absolute",
    ]).map((line) => line.replaceAll(/[0-9a-f]{40}/g, "<sha>"));

    assert.deepEqual(failures, [
      "../escape: not a safe relative fixture path",
      "/absolute: not a safe relative fixture path",
      "fixtures/empty: not hydrated (expected <sha>)",
      "fixtures/missing: not hydrated (expected <sha>)",
      "fixtures/not-gitlink: not a pinned gitlink",
      "fixtures/wrong: checked out <sha>, expected <sha>",
    ]);

    assert.throws(
      () => assertHydratedGitlinkFixtures(["fixtures/missing"], root),
      (error) => {
        assert(error instanceof Error);
        assert.match(error.message, /corpus fixture hydration preflight failed:/);
        assert.match(error.message, /git submodule update --init --depth 1 -- fixtures\/missing/);
        return true;
      },
    );
    assert.throws(
      () =>
        assertHydratedGitlinkFixtures(
          ["fixtures/missing", "../escape", "/absolute", "fixtures/not-gitlink"],
          root,
        ),
      (error) => {
        assert(error instanceof Error);
        assert.match(error.message, /git submodule update --init --depth 1 -- fixtures\/missing/);
        assert.doesNotMatch(error.message, /git submodule update[^\n]*\.\.\/escape/);
        assert.doesNotMatch(error.message, /git submodule update[^\n]*\/absolute/);
        assert.doesNotMatch(error.message, /git submodule update[^\n]*fixtures\/not-gitlink/);
        return true;
      },
    );
    assert.throws(
      () => assertHydratedGitlinkFixtures(["fixtures/not-gitlink"], root),
      (error) => {
        assert(error instanceof Error);
        assert.doesNotMatch(error.message, /\nRun:\n/);
        return true;
      },
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

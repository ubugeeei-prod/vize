import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { gitTrackedFiles } from "../../tools/support/git-tracked-files.mjs";

test("the complete Git index remains readable above the default 1 MiB buffer", (t) => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-git-tracked-files-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  function git(args: string[], input?: string) {
    const result = spawnSync("git", args, { cwd: root, encoding: "utf8", input });
    assert.ifError(result.error);
    assert.equal(result.status, 0, result.stderr);
    return result.stdout;
  }
  git(["init", "--quiet"]);
  const blob = git(["hash-object", "-w", "--stdin"], "").trim();
  // Index-only entries exercise a real Git stream without thousands of files.
  const files = Array.from(
    { length: 8192 },
    (_, i) => `fixtures/${String(i).padStart(5, "0")}-${"x".repeat(128)}.txt`,
  );
  files.push("fixtures/newline\nname.txt", "fixtures/日本語😀.txt");
  files.sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b)));
  const outputBytes = Buffer.byteLength(`${files.join("\0")}\0`);
  assert.ok(outputBytes > 1024 * 1024, `real Git stream is ${outputBytes} bytes`);
  git(
    ["update-index", "-z", "--index-info"],
    files.map((file) => `100644 ${blob}\t${file}\0`).join(""),
  );

  assert.deepEqual(gitTrackedFiles(root), files);
  assert.throws(
    () => gitTrackedFiles(root, { maxBuffer: 1024 }),
    (error: unknown) => {
      assert.ok(error instanceof Error);
      assert.match(error.message, /git ls-files failed/);
      assert.match(error.message, /ENOBUFS/);
      assert.equal((error.cause as NodeJS.ErrnoException).code, "ENOBUFS");
      return true;
    },
    "a buffer overflow must never return a partial tracked-file list",
  );
});

test("a failed Git command remains a failed tracked-file inspection", (t) => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-git-tracked-files-not-repo-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  assert.throws(() => gitTrackedFiles(root), /git ls-files failed.*status=128/);
});

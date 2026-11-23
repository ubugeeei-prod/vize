import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gitBodyDigest } from "../performance/support/warm-type-backed-blob-hash.ts";

test("large regular blobs stream every byte from the original Git object", (t) => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-git-blob-"));
  t.after(() => fs.rmSync(cwd, { recursive: true, force: true }));
  const git = (...args: string[]) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  git("init", "--quiet");
  const file = "file with spaces";
  const body = Buffer.alloc(2 * 1024 * 1024 + 7, 0xa5);
  fs.writeFileSync(path.join(cwd, file), body);
  git("add", file);
  git(
    "-c",
    "user.name=Test",
    "-c",
    "user.email=test@example.invalid",
    "-c",
    "commit.gpgsign=false",
    "-c",
    "core.hooksPath=/dev/null",
    "commit",
    "--quiet",
    "-m",
    "fixture",
  );
  const revision = git("rev-parse", "HEAD");
  const object = git("rev-parse", revision + ":" + file);
  const replacement = execFileSync("git", ["hash-object", "-w", "--stdin"], {
    cwd,
    input: "replacement",
    encoding: "utf8",
  }).trim();
  git("replace", object, replacement);
  assert.equal(gitBodyDigest(cwd, revision, file), createHash("sha256").update(body).digest("hex"));
  const worker = fileURLToPath(
    new URL("../performance/support/warm-type-backed-blob-hash.ts", import.meta.url),
  );
  const truncated = spawnSync(process.execPath, [worker, object, String(body.length + 1)], {
    cwd,
    encoding: "utf8",
  });
  assert.equal(truncated.status, 1);
  assert.equal(truncated.stdout, "");
  assert.match(truncated.stderr, /Incomplete Git blob/);
});

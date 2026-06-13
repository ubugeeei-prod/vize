import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";

function runGit(cwd: string, args: string[]): string {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return result.stdout.trim();
}

function writeLines(filePath: string, count: number): void {
  const lines = Array.from({ length: count }, (_, index) => `line ${index + 1}`);
  fs.writeFileSync(filePath, `${lines.join("\n")}\n`);
}

function resolveBaseRef(): string | undefined {
  if (process.env.SOURCE_LENGTH_BASE_REF) {
    return process.env.SOURCE_LENGTH_BASE_REF;
  }
  if (!process.env.GITHUB_BASE_REF) {
    return undefined;
  }

  const result = spawnSync(
    "git",
    ["fetch", "--no-tags", "--depth=1", "origin", process.env.GITHUB_BASE_REF],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return "FETCH_HEAD";
}

test("source length script checks the current checkout", () => {
  const args = ["--check", "--max-lines", "350", "--limit", "5"];
  const baseRef = resolveBaseRef();
  if (baseRef != null) {
    args.push("--base-ref", baseRef);
  }

  const result = runMoonScript("source_file_lengths", args);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /Source files scanned: \d+/);
  assert.match(result.stdout, /Files over 350 lines: \d+/);
});

test("source length script rejects grown over-limit files", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-lengths-"));
  const filePath = path.join(cwd, "large.ts");
  runGit(cwd, ["init", "-q"]);
  writeLines(filePath, 351);
  runGit(cwd, ["add", "large.ts"]);
  runGit(cwd, [
    "-c",
    "user.name=Vize",
    "-c",
    "user.email=vize@example.com",
    "commit",
    "-qm",
    "base",
  ]);
  const baseRef = runGit(cwd, ["rev-parse", "HEAD"]);

  writeLines(filePath, 352);
  const result = runMoonScript(
    "source_file_lengths",
    ["--check", "--base-ref", baseRef, "--max-lines", "350", "--limit", "5"],
    { cwd },
  );

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stdout, /over-limit file grew/);
  assert.match(result.stdout, /large\.ts/);
});

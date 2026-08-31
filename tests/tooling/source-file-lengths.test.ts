import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";

const command = path.join(repoRoot, "tools/commands/ci/source-file-lengths.rs");

function runGit(cwd: string, args: string[]): string {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return result.stdout.trim();
}

function writeLines(filePath: string, count: number): void {
  const lines = Array.from({ length: count }, (_, index) => `line ${index + 1}`);
  fs.writeFileSync(filePath, `${lines.join("\n")}\n`);
}

function runSourceLengthScript(args: string[] = [], cwd = repoRoot) {
  return spawnSync("rust-script", [command, ...args], { cwd, encoding: "utf8" });
}

function resolveBaseRef(cwd = repoRoot, env: NodeJS.ProcessEnv = process.env): string | undefined {
  if (env.SOURCE_LENGTH_BASE_REF) {
    return env.SOURCE_LENGTH_BASE_REF;
  }
  if (!env.GITHUB_BASE_REF) {
    return undefined;
  }

  assert.ok(env.GITHUB_EVENT_PATH, "GITHUB_EVENT_PATH is required for pull-request source checks");
  let event: unknown;
  try {
    event = JSON.parse(fs.readFileSync(env.GITHUB_EVENT_PATH, "utf8"));
  } catch (error) {
    throw new Error(`Failed to read pull-request event ${env.GITHUB_EVENT_PATH}`, {
      cause: error,
    });
  }
  const baseSha = (event as { pull_request?: { base?: { sha?: unknown } } }).pull_request?.base
    ?.sha;
  assert.ok(
    typeof baseSha === "string" && /^[0-9a-f]{40}$/.test(baseSha),
    "pull_request.base.sha must be a full lowercase commit SHA",
  );

  const result = spawnSync("git", ["fetch", "--no-tags", "--depth=1", "origin", baseSha], {
    cwd,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return baseSha;
}

test("source length script checks the current checkout", () => {
  const args = ["--check", "--max-lines", "350", "--limit", "5"];
  const baseRef = resolveBaseRef();
  if (baseRef != null) {
    args.push("--base-ref", baseRef);
  }

  const result = runSourceLengthScript(args);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /Source files scanned: \d+/);
  assert.match(result.stdout, /Files over 350 lines: \d+/);
});

test("source length comparison keeps the event base when the branch advances", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-length-base-"));
  const remote = path.join(root, "remote.git");
  const publisher = path.join(root, "publisher");
  const checkout = path.join(root, "checkout");
  const eventPath = path.join(root, "event.json");
  fs.mkdirSync(remote);
  fs.mkdirSync(publisher);
  fs.mkdirSync(checkout);
  runGit(remote, ["init", "--bare", "-q"]);
  runGit(publisher, ["init", "-q", "--initial-branch=main"]);
  writeLines(path.join(publisher, "large.ts"), 351);
  runGit(publisher, ["add", "large.ts"]);
  runGit(publisher, [
    "-c",
    "user.name=Vize",
    "-c",
    "user.email=vize@example.com",
    "commit",
    "-qm",
    "event base",
  ]);
  const eventBaseSha = runGit(publisher, ["rev-parse", "HEAD"]);
  runGit(publisher, ["remote", "add", "origin", remote]);
  runGit(publisher, ["push", "-q", "-u", "origin", "main"]);

  writeLines(path.join(publisher, "large.ts"), 352);
  runGit(publisher, ["add", "large.ts"]);
  runGit(publisher, [
    "-c",
    "user.name=Vize",
    "-c",
    "user.email=vize@example.com",
    "commit",
    "-qm",
    "advance main",
  ]);
  const advancedSha = runGit(publisher, ["rev-parse", "HEAD"]);
  runGit(publisher, ["push", "-q", "origin", "main"]);

  runGit(checkout, ["init", "-q"]);
  runGit(checkout, ["remote", "add", "origin", remote]);
  fs.writeFileSync(eventPath, JSON.stringify({ pull_request: { base: { sha: eventBaseSha } } }));

  const resolved = resolveBaseRef(checkout, {
    GITHUB_BASE_REF: "main",
    GITHUB_EVENT_PATH: eventPath,
  });
  assert.ok(resolved);
  assert.equal(resolved, eventBaseSha);
  assert.equal(runGit(checkout, ["rev-parse", resolved]), eventBaseSha);
  assert.notEqual(resolved, advancedSha);
});

test("source length comparison preserves an explicit local base override", () => {
  assert.equal(resolveBaseRef(repoRoot, { SOURCE_LENGTH_BASE_REF: "local-base" }), "local-base");
});

test("source length comparison rejects malformed pull-request metadata", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-length-event-"));
  const eventPath = path.join(root, "event.json");
  fs.writeFileSync(eventPath, JSON.stringify({ pull_request: { base: { sha: "main" } } }));

  assert.throws(
    () =>
      resolveBaseRef(root, {
        GITHUB_BASE_REF: "main",
        GITHUB_EVENT_PATH: eventPath,
      }),
    /pull_request\.base\.sha must be a full lowercase commit SHA/,
  );
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
  const result = runSourceLengthScript(
    ["--check", "--base-ref", baseRef, "--max-lines", "350", "--limit", "5"],
    cwd,
  );

  assert.equal(result.status, 1, result.stdout);
  assert.match(result.stdout, /over-limit file grew/);
  assert.match(result.stdout, /large\.ts/);
});

test("source length script accepts unchanged over-limit files", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-lengths-unchanged-"));
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

  const result = runSourceLengthScript(
    ["--check", "--base-ref", baseRef, "--max-lines", "350", "--limit", "5"],
    cwd,
  );

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /No new or grown files exceed 350 lines/);
});

test("source length script ignores package manifests", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-lengths-package-"));
  const packagePath = path.join(cwd, "package.json");
  const nestedPackagePath = path.join(cwd, "packages", "ui", "package.json");
  fs.mkdirSync(path.dirname(nestedPackagePath), { recursive: true });
  runGit(cwd, ["init", "-q"]);
  writeLines(packagePath, 351);
  writeLines(nestedPackagePath, 351);
  runGit(cwd, ["add", "package.json", "packages/ui/package.json"]);
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

  writeLines(packagePath, 352);
  writeLines(nestedPackagePath, 352);
  const result = runSourceLengthScript(
    ["--check", "--base-ref", baseRef, "--max-lines", "350", "--limit", "5"],
    cwd,
  );

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /No new or grown files exceed 350 lines/);
});

test(
  "source length script skips tracked symlink source paths",
  { skip: process.platform === "win32" },
  () => {
    const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-lengths-symlink-"));
    const targetPath = path.join(cwd, "target.ts");
    const linkPath = path.join(cwd, "link.ts");
    runGit(cwd, ["init", "-q"]);
    writeLines(path.join(cwd, "README.md"), 1);
    runGit(cwd, ["add", "README.md"]);
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

    writeLines(targetPath, 351);
    fs.symlinkSync(targetPath, linkPath);
    runGit(cwd, ["add", "link.ts"]);
    const result = runSourceLengthScript(
      ["--check", "--base-ref", baseRef, "--max-lines", "350", "--limit", "5"],
      cwd,
    );

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /No new or grown files exceed 350 lines/);
  },
);

test("source length script compares renamed over-limit files to their base path", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-lengths-rename-"));
  const filePath = path.join(cwd, "large.ts");
  const renamedPath = path.join(cwd, "renamed.ts");
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

  fs.renameSync(filePath, renamedPath);
  runGit(cwd, ["add", "-A"]);
  const result = runSourceLengthScript(
    ["--check", "--base-ref", baseRef, "--max-lines", "350", "--limit", "5"],
    cwd,
  );

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.doesNotMatch(result.stdout, /new file exceeds limit/);
});

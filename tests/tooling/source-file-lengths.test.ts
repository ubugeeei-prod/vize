import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { resolveSourceLengthBase as resolveBaseRef } from "./support/source-length-base.ts";

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

test("source length script checks the current checkout", () => {
  const args = ["--check", "--limit", "5"];
  const baseRef = resolveBaseRef(repoRoot);
  if (baseRef != null) {
    args.push("--base-ref", baseRef);
  }

  const result = runSourceLengthScript(args);

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /Source files scanned: \d+/);
  assert.match(result.stdout, /Files over 500 lines: \d+/);
});

test("the default limit accepts 500 lines and rejects the 501st line", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-length-boundary-"));
  const filePath = path.join(cwd, "module.rs");
  try {
    runGit(cwd, ["init", "-q"]);
    writeLines(filePath, 350);
    runGit(cwd, ["add", "module.rs"]);
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
    writeLines(filePath, 500);
    const accepted = runSourceLengthScript(["--check", "--base-ref", baseRef], cwd);
    assert.equal(accepted.status, 0, accepted.stdout);
    const custom = runSourceLengthScript(
      ["--check", "--base-ref", baseRef, "--max-lines", "450"],
      cwd,
    );
    assert.equal(custom.status, 1, custom.stdout);
    writeLines(filePath, 501);
    const crossed = runSourceLengthScript(["--check", "--base-ref", baseRef], cwd);
    assert.equal(crossed.status, 1, crossed.stdout);
    assert.match(crossed.stdout, /crossed limit/);
    const newFile = path.join(cwd, "new.rs");
    writeLines(newFile, 501);
    runGit(cwd, ["add", "new.rs"]);
    const added = runSourceLengthScript(["--check", "--base-ref", baseRef], cwd);
    assert.equal(added.status, 1, added.stdout);
    assert.match(added.stdout, /new file exceeds limit/);
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
});

test("ordinary Rust child test modules are accepted", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-test-dir-"));
  try {
    runGit(cwd, ["init", "-q"]);
    fs.mkdirSync(path.join(cwd, "legacy"));
    writeLines(path.join(cwd, "legacy.rs"), 3);
    writeLines(path.join(cwd, "legacy/tests.rs"), 3);
    runGit(cwd, ["add", "-A"]);
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
    assert.equal(runSourceLengthScript(["--check", "--base-ref", baseRef], cwd).status, 0);

    for (const name of ["tests.rs", "test.rs"]) {
      fs.mkdirSync(path.join(cwd, "parser"), { recursive: true });
      writeLines(path.join(cwd, "parser.rs"), 3);
      writeLines(path.join(cwd, "parser", name), 3);
      runGit(cwd, ["add", "-A"]);
      const accepted = runSourceLengthScript(["--check", "--base-ref", baseRef], cwd);
      assert.equal(accepted.status, 0, accepted.stdout);
      fs.rmSync(path.join(cwd, "parser"), { recursive: true });
    }

    writeLines(path.join(cwd, "parser_tests.rs"), 3);
    fs.mkdirSync(path.join(cwd, "nested"));
    writeLines(path.join(cwd, "nested/tests.rs"), 3);
    writeLines(path.join(cwd, "nested/cases.rs"), 3);
    runGit(cwd, ["add", "-A"]);
    const accepted = runSourceLengthScript(["--check", "--base-ref", baseRef], cwd);
    assert.equal(accepted.status, 0, accepted.stdout);
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
});

test("Rust path overrides fail even behind inactive cfgs while examples remain data", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-module-layout-"));
  try {
    runGit(cwd, ["init", "-q"]);
    const source = path.join(cwd, "lib.rs");
    fs.writeFileSync(source, "mod parser;\n");
    fs.writeFileSync(path.join(cwd, "parser.rs"), "");
    runGit(cwd, ["add", "-A"]);
    for (const attribute of [
      '#[path = "parser.rs"]',
      '#[cfg(any())]\n#[path = "parser.rs"]',
      '#[cfg_attr(any(), path = "parser.rs")]',
      '#[cfg_attr(any(), cfg_attr(any(), path = "parser.rs"))]',
    ]) {
      fs.writeFileSync(source, `${attribute}\nmod parser;\n`);
      const rejected = runSourceLengthScript(["--check"], cwd);
      assert.equal(rejected.status, 1, `${rejected.stderr}\n${rejected.stdout}`);
      assert.match(rejected.stdout, /lib\.rs:\d+: path attribute/);
    }
    fs.writeFileSync(
      source,
      '// #[path = "example.rs"]\nconst EXAMPLE: &str = r##"#[path = "example.rs"]"##;\nmod parser;\n',
    );
    const accepted = runSourceLengthScript(["--check"], cwd);
    assert.equal(accepted.status, 0, `${accepted.stderr}\n${accepted.stdout}`);
    const commandDir = path.join(cwd, "tools/commands/fixtures");
    fs.mkdirSync(commandDir, { recursive: true });
    fs.writeFileSync(path.join(commandDir, "observer.rs"), '#[path = "parser.rs"]\nmod parser;\n');
    runGit(cwd, ["add", "-A"]);
    const toolOverride = runSourceLengthScript(["--check"], cwd);
    assert.equal(toolOverride.status, 1, `${toolOverride.stderr}\n${toolOverride.stdout}`);
    assert.match(toolOverride.stdout, /tools\/commands\/fixtures\/observer\.rs:1: path attribute/);
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
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
  writeLines(path.join(publisher, "large.ts"), 496);
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

  writeLines(path.join(publisher, "large.ts"), 503);
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
  runGit(checkout, ["fetch", "-q", "origin", "main"]);
  runGit(checkout, ["checkout", "-q", "--detach", "FETCH_HEAD"]);

  for (const kind of ["pull_request", "merge_group"]) {
    fs.writeFileSync(
      eventPath,
      JSON.stringify(
        kind === "merge_group"
          ? { merge_group: { base_sha: eventBaseSha } }
          : { pull_request: { base: { sha: eventBaseSha } } },
      ),
    );
    const resolved = resolveBaseRef(checkout, {
      GITHUB_EVENT_NAME: kind,
      GITHUB_BASE_REF: kind === "pull_request" ? "main" : "",
      GITHUB_EVENT_PATH: eventPath,
    });
    assert.ok(resolved);
    assert.equal(resolved, eventBaseSha);
    assert.equal(runGit(checkout, ["rev-parse", resolved]), eventBaseSha);
    assert.notEqual(resolved, advancedSha);
    const result = runSourceLengthScript(["--check", "--base-ref", resolved], checkout);
    assert.equal(result.status, 1, `${kind}: ${result.stderr}\n${result.stdout}`);
    assert.match(result.stdout, /crossed limit/);
  }
  // A stacked PR checks a verified merge parent even if its event still
  // carries the older base SHA. The parent already contains the longer file.
  fs.writeFileSync(eventPath, JSON.stringify({ pull_request: { base: { sha: eventBaseSha } } }));
  const stackParent = resolveBaseRef(checkout, {
    GITHUB_EVENT_NAME: "pull_request",
    GITHUB_BASE_REF: "stack-parent",
    GITHUB_EVENT_PATH: eventPath,
    SOURCE_LENGTH_BASE_REF: advancedSha,
  });
  assert.equal(stackParent, advancedSha);
  const stacked = runSourceLengthScript(["--check", "--base-ref", stackParent], checkout);
  assert.equal(stacked.status, 0, stacked.stdout);

  const incorrectBase = runSourceLengthScript(["--check", "--base-ref", advancedSha], checkout);
  assert.equal(incorrectBase.status, 0, incorrectBase.stdout);
  fs.rmSync(root, { recursive: true, force: true });
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

test("source length comparison fails closed for malformed merge-group events", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-length-queue-event-"));
  const eventPath = path.join(root, "event.json");
  const env = {
    GITHUB_EVENT_NAME: "merge_group",
    GITHUB_BASE_REF: "",
    GITHUB_EVENT_PATH: eventPath,
  };
  try {
    for (const payload of [{}, { merge_group: {} }, { merge_group: { base_sha: "main" } }]) {
      fs.writeFileSync(eventPath, JSON.stringify(payload));
      assert.throws(
        () => resolveBaseRef(root, env),
        /merge_group\.base_sha must be a full lowercase commit SHA/,
      );
    }
    fs.writeFileSync(eventPath, "null");
    assert.throws(() => resolveBaseRef(root, env), /merge_group event must be an object/);
    fs.writeFileSync(eventPath, "{");
    assert.throws(() => resolveBaseRef(root, env), /Failed to read merge_group event/);
    runGit(root, ["init", "-q"]);
    runGit(root, ["remote", "add", "origin", path.join(root, "missing-remote.git")]);
    fs.writeFileSync(eventPath, JSON.stringify({ merge_group: { base_sha: "a".repeat(40) } }));
    assert.throws(() => resolveBaseRef(root, env), /missing-remote\.git/);
    assert.throws(
      () => resolveBaseRef(root, { GITHUB_EVENT_NAME: "merge_group" }),
      /GITHUB_EVENT_PATH is required/,
    );
    assert.equal(
      resolveBaseRef(root, { ...env, SOURCE_LENGTH_BASE_REF: "local-base" }),
      "local-base",
    );
    assert.equal(resolveBaseRef(root, {}), undefined);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("source length script rejects grown over-limit files", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-lengths-"));
  const filePath = path.join(cwd, "large.ts");
  runGit(cwd, ["init", "-q"]);
  writeLines(filePath, 501);
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

  writeLines(filePath, 502);
  const result = runSourceLengthScript(
    ["--check", "--base-ref", baseRef, "--max-lines", "500", "--limit", "5"],
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
  writeLines(filePath, 501);
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
    ["--check", "--base-ref", baseRef, "--max-lines", "500", "--limit", "5"],
    cwd,
  );

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /No new or grown files exceed 500 lines/);
});

test("source length script ignores package manifests", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-lengths-package-"));
  const packagePath = path.join(cwd, "package.json");
  const nestedPackagePath = path.join(cwd, "packages", "ui", "package.json");
  fs.mkdirSync(path.dirname(nestedPackagePath), { recursive: true });
  runGit(cwd, ["init", "-q"]);
  writeLines(packagePath, 501);
  writeLines(nestedPackagePath, 501);
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

  writeLines(packagePath, 502);
  writeLines(nestedPackagePath, 502);
  const result = runSourceLengthScript(
    ["--check", "--base-ref", baseRef, "--max-lines", "500", "--limit", "5"],
    cwd,
  );

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /No new or grown files exceed 500 lines/);
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

    writeLines(targetPath, 501);
    fs.symlinkSync(targetPath, linkPath);
    runGit(cwd, ["add", "link.ts"]);
    const result = runSourceLengthScript(
      ["--check", "--base-ref", baseRef, "--max-lines", "500", "--limit", "5"],
      cwd,
    );

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /No new or grown files exceed 500 lines/);
  },
);

test("source length script compares renamed over-limit files to their base path", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-source-lengths-rename-"));
  const filePath = path.join(cwd, "large.ts");
  const renamedPath = path.join(cwd, "renamed.ts");
  runGit(cwd, ["init", "-q"]);
  writeLines(filePath, 501);
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
    ["--check", "--base-ref", baseRef, "--max-lines", "500", "--limit", "5"],
    cwd,
  );

  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.doesNotMatch(result.stdout, /new file exceeds limit/);
});

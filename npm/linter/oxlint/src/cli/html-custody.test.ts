import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import type { OxlintHtmlOptions } from "@vizejs/native";
import { assertHtmlCustody, captureHtmlCustody } from "./html-custody.ts";
import type { HtmlContext } from "./html-operation.ts";

function fixture() {
  const temporary = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-git-custody-")));
  const main = path.join(temporary, "main"),
    linked = path.join(temporary, "linked");
  const git = (cwd: string, ...args: string[]) =>
    execFileSync("git", ["-c", "core.hooksPath=" + os.devNull, "-C", cwd, ...args], {
      encoding: "utf8",
      env: {
        ...Object.fromEntries(
          Object.entries(process.env).filter(([key]) => !key.startsWith("GIT_")),
        ),
        GIT_CONFIG_NOSYSTEM: "1",
        GIT_CONFIG_GLOBAL: os.devNull,
      },
    }).trim();
  fs.mkdirSync(main);
  git(main, "init", "-q");
  fs.writeFileSync(path.join(main, ".oxlintrc.json"), "{}\n");
  fs.mkdirSync(path.join(main, "app"));
  fs.writeFileSync(path.join(main, "app/index.html"), "<!doctype html><html></html>\n");
  git(main, "add", "-A");
  git(
    main,
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.invalid",
    "-c",
    "core.hooksPath=" + os.devNull,
    "-c",
    "commit.gpgsign=false",
    "commit",
    "-qm",
    "fixture",
  );
  git(main, "worktree", "add", "-q", "-b", "linked", linked);
  const common = git(main, "rev-parse", "--path-format=absolute", "--git-common-dir");
  const privateGit = git(linked, "rev-parse", "--absolute-git-dir");
  const exclude = git(linked, "rev-parse", "--path-format=absolute", "--git-path", "info/exclude");
  fs.writeFileSync(exclude, "app/Excluded.html\n");
  return {
    temporary,
    main,
    linked,
    common,
    privateGit,
    exclude,
    git,
    cleanup: () => fs.rmSync(temporary, { recursive: true, force: true }),
  };
}
function options(cwd: string): OxlintHtmlOptions {
  const rootJson = path.join(cwd, ".oxlintrc.json");
  return {
    cwd,
    rootJson,
    rootBytes: Array.from(fs.readFileSync(rootJson)),
    literalTarget: "app",
    hostProfile: "1.86.0",
    noIgnore: false,
    cliIgnorePatterns: [],
    customIgnoreFilename: ".eslintignore",
    format: "default",
    presentation: { cwd, graphicalTheme: "plain", links: false, width: 80, stylishNoColor: true },
  };
}

void test("real main and linked Git worktrees retain the independently resolved common exclusion", (t) => {
  const input = fixture();
  t.after(input.cleanup);
  for (const cwd of [input.main, input.linked]) {
    const capture = captureHtmlCustody(options(cwd), new Set([path.join(cwd, "app")]));
    assert.deepEqual(capture.sources.get(input.exclude), fs.readFileSync(input.exclude));
    assert.equal(capture.sources.has(path.join(input.linked, ".git/info/exclude")), false);
    if (cwd === input.linked) {
      for (const file of [path.join(cwd, ".git"), path.join(input.privateGit, "commondir")])
        assert.deepEqual(capture.sources.get(file), fs.readFileSync(file));
    }
  }
});

void test("relative gitfiles and relative or absolute commondir keep Git's actual exclusion identity", (t) => {
  const input = fixture();
  t.after(input.cleanup);
  const marker = path.join(input.linked, ".git"),
    commonFile = path.join(input.privateGit, "commondir");
  fs.writeFileSync(marker, `gitdir: ${path.relative(input.linked, input.privateGit)}\n`);
  for (const common of [path.relative(input.privateGit, input.common), input.common]) {
    fs.writeFileSync(commonFile, common + "\n");
    const actual = input.git(
      input.linked,
      "rev-parse",
      "--path-format=absolute",
      "--git-path",
      "info/exclude",
    );
    assert.equal(fs.realpathSync(actual), input.exclude);
    const capture = captureHtmlCustody(options(input.linked), new Set());
    assert.deepEqual(capture.sources.get(input.exclude), fs.readFileSync(actual));
    assert.deepEqual(capture.sources.get(marker), fs.readFileSync(marker));
    assert.deepEqual(capture.sources.get(commonFile), fs.readFileSync(commonFile));
  }
});

void test("a real separate Git directory without commondir preserves absence and its own exclusion", (t) => {
  const input = fixture();
  t.after(input.cleanup);
  const project = path.join(input.temporary, "separate"),
    metadata = path.join(input.temporary, "metadata");
  fs.mkdirSync(project);
  input.git(project, "init", "-q", "--separate-git-dir", metadata);
  fs.writeFileSync(path.join(project, ".oxlintrc.json"), "{}\n");
  const exclude = input.git(
    project,
    "rev-parse",
    "--path-format=absolute",
    "--git-path",
    "info/exclude",
  );
  const capture = captureHtmlCustody(options(project), new Set());
  assert.deepEqual(capture.sources.get(exclude), fs.readFileSync(exclude));
  assert.equal(capture.sources.has(path.join(metadata, "commondir")), true);
  assert.equal(capture.sources.get(path.join(metadata, "commondir")), null);
});

void test("Git indirection and common exclusions remain fresh authorities after capture", (t) => {
  const input = fixture();
  t.after(input.cleanup);
  const entrypoint = path.join(input.temporary, "provider.mjs"),
    packageFile = path.join(input.temporary, "package.json");
  fs.writeFileSync(entrypoint, "export {};\n");
  fs.writeFileSync(packageFile, "{}\n");
  for (const file of [
    path.join(input.linked, ".git"),
    path.join(input.privateGit, "commondir"),
    input.exclude,
  ]) {
    const original = fs.readFileSync(file),
      request = options(input.linked);
    const context: HtmlContext = {
      environment: {},
      options: request,
      custody: captureHtmlCustody(request, new Set()),
      provider: {
        requestedEntrypoint: entrypoint,
        entrypoint,
        package: packageFile,
        packageBytes: Array.from(fs.readFileSync(packageFile)),
        version: "1.86.0",
        sha256: createHash("sha256").update(fs.readFileSync(entrypoint)).digest("hex"),
      },
    };
    assertHtmlCustody(context);
    fs.appendFileSync(file, "\n");
    assert.throws(() => assertHtmlCustody(context), /original HTML input changed/u);
    fs.writeFileSync(file, original);
    assertHtmlCustody(context);
  }
});

void test("malformed or symbolic Git authorities refuse instead of silently losing exclusions", (t) => {
  const input = fixture();
  t.after(input.cleanup);
  const marker = path.join(input.linked, ".git"),
    original = fs.readFileSync(marker);
  for (const bytes of [
    Buffer.from("not a gitdir marker\n"),
    Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), original]),
  ]) {
    fs.writeFileSync(marker, bytes);
    assert.throws(
      () => captureHtmlCustody(options(input.linked), new Set()),
      /invalid Git directory metadata/u,
    );
  }
  // Both native and JS readers permit Git's terminal CR without stripping
  // any other path bytes; custody still retains the exact original spelling.
  fs.writeFileSync(marker, Buffer.concat([original.subarray(0, -1), Buffer.from("\r")]));
  assert.deepEqual(
    captureHtmlCustody(options(input.linked), new Set()).sources.get(marker),
    fs.readFileSync(marker),
  );
  fs.writeFileSync(marker, original);
  const destination = path.join(input.temporary, "exclude-copy");
  fs.renameSync(input.exclude, destination);
  fs.symlinkSync(destination, input.exclude);
  assert.throws(() => captureHtmlCustody(options(input.linked), new Set()), /regular original/u);
});

void test("unreadable Git exclusion I/O errors retain their original error and fail closed", (t) => {
  const input = fixture();
  t.after(input.cleanup);
  const request = options(input.main);
  const denied = Object.assign(new Error("owned exclusion permission denied"), {
    code: "EACCES",
    path: input.exclude,
  });
  t.mock.method(fs, "readFileSync", (...args: unknown[]) => {
    assert.equal(args[0], input.exclude);
    throw denied;
  });
  assert.throws(
    () => captureHtmlCustody(request, new Set()),
    (error: unknown) => error === denied,
  );
});

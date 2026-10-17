import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { after, before, test } from "node:test";
import type { OxlintHtmlOptions, OxlintHtmlOutcome } from "../../native/index.js";
import { snapshot, plainHostEnvironment, readEvents } from "./test-support/html-cli-oracles.mjs";
import { captureHtmlCustody } from "./cli/html-custody.ts";

const repository = path.resolve(import.meta.dirname, "../../..");
const nativeDirectory = path.join(repository, "npm/native");
const wrapper = path.join(repository, "npm/oxlint/dist/cli.mjs");
const plugin = path.join(repository, "npm/oxlint/dist/index.mjs");
const require = createRequire(import.meta.url);
const capturePath =
  process.env.VIZE_OXLINT_LINKED_WORKTREE_CAPTURE ??
  path.join(repository, "target/oxlint-linked-worktree-9362.json");
const observations: Record<string, unknown>[] = [];
function saveCapture() {
  fs.mkdirSync(path.dirname(capturePath), { recursive: true });
  fs.writeFileSync(
    capturePath,
    JSON.stringify(
      {
        schema: "vize.oxlint.linked-worktree",
        version: 1,
        source: process.env.GITHUB_SHA ?? null,
        observations,
      },
      null,
      2,
    ) + "\n",
  );
}
const html = '<!doctype html>\n<html><body><div v-html="content"></div></body></html>\n';
const expectedHtml = ["app.html", "keep.html", "nested/override.html"];
const commonExclude = "common-only.html\nnested/override.html\n";
let native: { lintOxlintHtml(options: OxlintHtmlOptions): OxlintHtmlOutcome };
let provider: string;
interface NativeCustody {
  schema: string;
  version: number;
  source: { head: string };
  toolchain: unknown;
  binary: { path: string; sha256: string };
  calls: string;
}
interface NativeEvent {
  kind: string;
  pid: number;
  source: unknown;
  binary: string;
  binarySha256: string;
  args?: unknown[];
  result?: unknown;
  outcome?: string;
}
let custody: NativeCustody;
function events(): NativeEvent[] {
  const rows = readEvents(custody.calls) as NativeEvent[];
  const loaded = new Set<number>();
  for (const row of rows) {
    if (row.kind === "load") loaded.add(row.pid);
    if (["call", "html-call"].includes(row.kind))
      assert.ok(loaded.has(row.pid), "physical image load precedes every observed call");
    assert.deepEqual(row.source, custody.source);
    assert.equal(row.binary, custody.binary.path);
    assert.equal(row.binarySha256, custody.binary.sha256);
  }
  return rows;
}
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

before(() => {
  saveCapture();
  assert.equal(process.env.GITHUB_ACTIONS, "true", "run through the authenticated source carrier");
  assert.ok(process.env.VIZE_OXLINT_NATIVE_CUSTODY && process.env.NODE_OPTIONS);
  custody = JSON.parse(
    fs.readFileSync(process.env.VIZE_OXLINT_NATIVE_CUSTODY, "utf8"),
  ) as NativeCustody;
  assert.equal(custody.schema, "vize.oxlint.source-native");
  assert.equal(custody.version, 1);
  const history = require("../../native/scripts/formatter-history-build.mjs") as {
    nativeHistoryReceipt(directory: string): string;
    validateNativeHistoryBuild(directory: string, receipt: unknown): void;
  };
  const receipt = JSON.parse(
    fs.readFileSync(history.nativeHistoryReceipt(nativeDirectory), "utf8"),
  ) as {
    source: { head: string };
    toolchain: unknown;
    frozen: { sha256: string };
  };
  history.validateNativeHistoryBuild(nativeDirectory, receipt);
  assert.deepEqual(custody.source, receipt.source);
  assert.deepEqual(custody.toolchain, receipt.toolchain);
  assert.equal(custody.source.head, process.env.GITHUB_SHA);
  assert.equal(custody.source.head, git(repository, "rev-parse", "HEAD"));
  assert.equal(custody.binary.sha256, receipt.frozen.sha256);
  assert.equal(hash(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
  native = require(path.join(nativeDirectory, "index.js")) as typeof native;
  assert.equal(typeof native.lintOxlintHtml, "function");
  assert.ok(
    events().some((event) => event.kind === "load" && event.pid === process.pid),
    "the driver must load the authenticated image through the active preload",
  );
  assert.ok(
    fs.existsSync(wrapper) && fs.existsSync(plugin),
    "build the genuine source wrapper first",
  );
  const entrypoint =
    process.env.VIZE_OXLINT_TEST_ENTRYPOINT ??
    path.join(path.dirname(require.resolve("oxlint/package.json")), "bin/oxlint");
  provider = fs.realpathSync(entrypoint);
  const metadata = JSON.parse(
    fs.readFileSync(path.resolve(provider, "../../package.json"), "utf8"),
  ) as {
    name: string;
    version: string;
  };
  assert.deepEqual(
    { name: metadata.name, version: metadata.version },
    { name: "oxlint", version: "1.86.0" },
  );
  const handshake = spawnSync(process.execPath, [provider, "--version"], { encoding: "utf8" });
  assert.equal(handshake.status, 0, handshake.stderr);
  assert.equal(handshake.stdout.trim(), "Version: 1.86.0");
});

function gitEnvironment(): NodeJS.ProcessEnv {
  return {
    ...Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith("GIT_"))),
    GIT_TERMINAL_PROMPT: "0",
    GIT_CONFIG_NOSYSTEM: "1",
    GIT_CONFIG_GLOBAL: os.devNull,
  };
}
function git(cwd: string, ...args: string[]): string {
  const result = spawnSync(
    "git",
    ["-c", "core.autocrlf=false", "-c", "core.hooksPath=" + os.devNull, ...args],
    {
      cwd,
      encoding: "utf8",
      env: gitEnvironment(),
    },
  );
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, `${args.join(" ")}\n${result.stdout}\n${result.stderr}`);
  return result.stdout.trim();
}

function write(directory: string, name: string, bytes: string) {
  const file = path.join(directory, name);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, bytes);
}

function initialize(directory: string) {
  fs.mkdirSync(directory, { recursive: true });
  git(directory, "init", "-q");
  write(directory, ".gitignore", "node_modules/\n");
  write(directory, "app.html", html);
  write(
    directory,
    "AppPanel.vue",
    '<script setup>const content = "hello";</script>\n<template><div v-html="content" /></template>\n',
  );
  write(
    directory,
    ".oxlintrc.json",
    JSON.stringify({
      jsPlugins: [plugin],
      categories: { correctness: "off" },
      rules: { "vize/vue/no-v-html": "warn" },
      settings: { vize: { helpLevel: "none" } },
    }) + "\n",
  );
  git(directory, "add", ".");
  git(
    directory,
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.invalid",
    "-c",
    "commit.gpgsign=false",
    "commit",
    "-qm",
    "fixture",
  );
}

function prepare(directory: string) {
  for (const name of [
    "keep.html",
    "common-only.html",
    "nested/blocked.html",
    "nested/override.html",
  ])
    write(directory, name, html);
  write(directory, "nested/.gitignore", "blocked.html\n!override.html\n");
  write(directory, "untouched.txt", "unrelated bytes\r\n");
  const common = git(directory, "rev-parse", "--path-format=absolute", "--git-common-dir");
  write(common, "info/exclude", commonExclude);
  const modules = path.join(directory, "node_modules");
  fs.mkdirSync(modules, { recursive: true });
  fs.symlinkSync(
    path.resolve(provider, "../.."),
    path.join(modules, "oxlint"),
    process.platform === "win32" ? "junction" : "dir",
  );
  const exclusions = spawnSync(
    "git",
    ["check-ignore", "--no-index", "common-only.html", "nested/blocked.html"],
    { cwd: directory, encoding: "utf8", env: gitEnvironment() },
  );
  assert.equal(exclusions.status, 0, exclusions.stderr);
  assert.deepEqual(exclusions.stdout.trim().split("\n").sort(), [
    "common-only.html",
    "nested/blocked.html",
  ]);
  const included = spawnSync("git", ["check-ignore", "--no-index", "nested/override.html"], {
    cwd: directory,
    encoding: "utf8",
    env: gitEnvironment(),
  });
  assert.equal(included.status, 1, included.stderr);
  return common;
}

function execute(temporary: string, cwd: string, label: string) {
  const rootJson = path.join(cwd, ".oxlintrc.json");
  const options: OxlintHtmlOptions = {
    cwd,
    literalTarget: ".",
    rootJson,
    rootBytes: [...fs.readFileSync(rootJson)],
    hostProfile: "1.86.0",
    noIgnore: false,
    cliIgnorePatterns: [],
    customIgnoreFilename: ".eslintignore",
    format: "json",
    presentation: {
      stylishRelative: false,
      cwd,
      graphicalTheme: "plain",
      links: false,
      width: 400,
      stylishNoColor: true,
    },
  };
  const beforeState = snapshot(temporary);
  const record: Record<string, unknown> = {
    cwd,
    label,
    options,
    before: beforeState,
    provider: { path: provider, sha256: hash(fs.readFileSync(provider)) },
    metadata: null,
    native: null,
    cli: null,
    afterNative: null,
    afterCli: null,
    passed: false,
  };
  observations.push(record);
  const save = saveCapture;
  save();
  try {
    const beforeEvents = events().length;
    const custody = captureHtmlCustody(
      options,
      new Set(expectedHtml.map((name) => path.join(cwd, name))),
    );
    const gitFile = path.join(cwd, ".git");
    const metadataFiles = [gitFile];
    if (fs.statSync(gitFile).isFile()) {
      const privateDirectory = git(cwd, "rev-parse", "--absolute-git-dir");
      const commondir = path.join(privateDirectory, "commondir");
      metadataFiles.push(commondir);
    }
    const commonDirectory = git(cwd, "rev-parse", "--path-format=absolute", "--git-common-dir");
    metadataFiles.push(path.join(commonDirectory, "info/exclude"));
    for (const file of metadataFiles) {
      if (fs.existsSync(file) && fs.statSync(file).isDirectory()) continue;
      assert.ok(custody.sources.has(file), `custody must bind Git authority ${file}`);
      assert.deepEqual(
        custody.sources.get(file),
        fs.existsSync(file) ? fs.readFileSync(file) : null,
      );
    }
    record.metadata = metadataFiles.map((file) => [
      file,
      custody.sources.get(file)?.toString("base64") ?? null,
    ]);
    save();
    const outcome = native.lintOxlintHtml(options);
    record.native = outcome;
    record.trace = events().slice(beforeEvents);
    const direct = events()
      .slice(beforeEvents)
      .filter((event) => event.kind === "html-call" && event.pid === process.pid);
    assert.equal(direct.length, 1);
    assert.equal(direct[0].outcome, "return");
    assert.deepEqual(direct[0].args, [options]);
    assert.deepEqual(direct[0].result, JSON.parse(JSON.stringify(outcome)) as unknown);
    record.afterNative = snapshot(temporary);
    save();
    assert.deepEqual(
      record.afterNative,
      beforeState,
      "native operation preserves the entire Git and original tree",
    );
    assert.equal(outcome.refused, undefined);
    assert.ok(outcome.completed);
    const completed = outcome.completed;
    assert.equal(completed.cwd, cwd);
    assert.equal(completed.repository, cwd);
    assert.equal(completed.target, cwd);
    assert.equal(completed.literalTarget, ".");
    assert.equal(completed.rootJson, rootJson);
    assert.deepEqual(
      completed.files.map((row) => row.filename),
      expectedHtml.map((name) => path.join(cwd, name)),
    );
    assert.deepEqual(
      completed.originals.map((row) => row.cwdRelative),
      expectedHtml,
    );
    assert.equal(completed.executedFileCount, expectedHtml.length);
    assert.equal(completed.errors, 0);
    assert.equal(completed.warnings, expectedHtml.length);
    for (const original of completed.originals) {
      assert.equal(original.path, path.join(cwd, original.cwdRelative));
      assert.deepEqual(original.bytes, [...Buffer.from(html)]);
    }
    for (const authority of metadataFiles) {
      if (fs.existsSync(authority) && fs.statSync(authority).isDirectory()) continue;
      const source = completed.sources.find((row) => row.path === authority);
      assert.ok(source, `native result retains ${authority}`);
      assert.deepEqual(
        source.bytes == null ? null : Buffer.from(source.bytes),
        custody.sources.get(authority),
      );
    }
    const environment = {
      ...plainHostEnvironment(process.env),
      VIZE_PREFER_WORKSPACE_BINDING: "1",
    };
    const cliEvents = events().length;
    const result = spawnSync(
      process.execPath,
      [wrapper, ".", "--format", "json", "--config", rootJson],
      { cwd, env: environment, encoding: "utf8", timeout: 60_000, maxBuffer: 8 * 1024 * 1024 },
    );
    record.cli = {
      status: result.status,
      signal: result.signal,
      error: result.error?.message,
      stdout: result.stdout,
      stderr: result.stderr,
    };
    record.trace = events().slice(beforeEvents);
    record.afterCli = snapshot(temporary);
    save();
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.equal(result.stderr, "");
    const observed = events().slice(cliEvents);
    assert.equal(
      observed.filter(
        (event) =>
          event.kind === "html-call" && event.outcome === "return" && event.pid === result.pid,
      ).length,
      1,
    );
    for (const kind of ["load", "host-handshake", "host-child", "call"])
      assert.ok(
        observed.some((event) => event.kind === kind),
        `actual wrapper must retain ${kind}`,
      );
    record.trace = events().slice(beforeEvents);

    const report = JSON.parse(result.stdout) as {
      diagnostics: { filename: string; severity: string }[];
    };
    assert.ok(Array.isArray(report.diagnostics));
    assert.deepEqual(
      report.diagnostics.map((row) => path.relative(cwd, path.resolve(cwd, row.filename))).sort(),
      ["AppPanel.vue", ...expectedHtml].sort(),
    );
    assert.ok(report.diagnostics.every((row) => row.severity === "warning"));
    assert.deepEqual(
      record.afterCli,
      beforeState,
      "real CLI preserves all input, Git metadata and unrelated controls",
    );
    record.passed = true;
    save();
  } catch (error) {
    record.failure =
      error instanceof Error ? { message: error.message, stack: error.stack } : String(error);
    save();
    throw error;
  }
}

for (const representation of ["generated", "absolute", "relative"]) {
  void test(`real Git main and linked worktree retain common exclude authority (${representation})`, () => {
    const temporary = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-linked-html-")));
    try {
      const main = path.join(temporary, "main"),
        linked = path.join(temporary, "linked");
      initialize(main);
      git(main, "worktree", "add", "--detach", "-q", linked);
      const privateDirectory = git(linked, "rev-parse", "--absolute-git-dir");
      const common = git(linked, "rev-parse", "--path-format=absolute", "--git-common-dir");
      assert.ok(fs.statSync(path.join(linked, ".git")).isFile());
      if (representation !== "generated") {
        write(
          linked,
          ".git",
          `gitdir: ${representation === "absolute" ? privateDirectory : path.relative(linked, privateDirectory)}\n`,
        );
        write(
          privateDirectory,
          "commondir",
          `${representation === "absolute" ? common : path.relative(privateDirectory, common)}\n`,
        );
      }
      assert.equal(prepare(main), common);
      assert.equal(prepare(linked), common);
      execute(temporary, main, `${representation}-main`);
      execute(temporary, linked, `${representation}-linked`);
      assert.equal(fs.readFileSync(path.join(common, "info/exclude"), "utf8"), commonExclude);
    } finally {
      fs.rmSync(temporary, { recursive: true, force: true });
    }
  });
}

void test("a real submodule Git file without commondir retains its own exclude authority", () => {
  const temporary = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-submodule-html-")));
  try {
    const origin = path.join(temporary, "origin"),
      parent = path.join(temporary, "parent");
    initialize(origin);
    initialize(parent);
    git(parent, "-c", "protocol.file.allow=always", "submodule", "add", "-q", origin, "module");
    const module = path.join(parent, "module");
    assert.ok(fs.statSync(path.join(module, ".git")).isFile());
    const privateDirectory = git(module, "rev-parse", "--absolute-git-dir");
    assert.equal(fs.existsSync(path.join(privateDirectory, "commondir")), false);
    prepare(module);
    execute(temporary, module, "submodule-no-commondir");
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

after(() => {
  saveCapture();
  assert.equal(observations.length, 7);
  assert.ok(observations.every((record) => record.passed === true));
});

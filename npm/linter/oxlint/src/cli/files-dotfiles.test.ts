import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { collectVueLikeFilesFromTargets } from "./files.ts";

/** Dot-entries must be candidates; `.git` and `node_modules` must stay out. */
void test("a directory target observes dot-files and skips node_modules and .git", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-files-"));
  try {
    for (const file of [
      "src/App.vue",
      ".lintstagedrc.cjs",
      ".husky/pre-commit.js",
      "node_modules/pkg/index.js",
      ".git/hooks/x.js",
    ]) {
      fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
      fs.writeFileSync(path.join(root, file), "");
    }
    const observed = new Set<string>();
    const vue = collectVueLikeFilesFromTargets(root, ["."], (file) => observed.add(file));
    assert.deepEqual(vue, [path.join(root, "src/App.vue")]);
    assert.ok(observed.has(path.join(root, ".lintstagedrc.cjs")));
    assert.ok(observed.has(path.join(root, ".husky/pre-commit.js")));
    for (const excluded of [".git", "node_modules"]) {
      const excludedRoot = path.join(root, excluded);
      assert.ok(!observed.has(excludedRoot), `${excluded} itself is not observed`);
      assert.ok(
        ![...observed].some((file) => file.startsWith(excludedRoot + path.sep)),
        `nothing below ${excluded} is observed`,
      );
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

/** An unreadable directory is skipped instead of aborting the whole walk. */
void test(
  "an unreadable directory does not abort collection",
  { skip: process.platform === "win32" },
  () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-files-"));
    const locked = path.join(root, "locked");
    try {
      fs.mkdirSync(locked);
      fs.writeFileSync(path.join(root, "ok.vue"), "");
      fs.chmodSync(locked, 0o000);
      const vue = collectVueLikeFilesFromTargets(root, ["."]);
      assert.deepEqual(vue, [path.join(root, "ok.vue")]);
    } finally {
      fs.chmodSync(locked, 0o755);
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

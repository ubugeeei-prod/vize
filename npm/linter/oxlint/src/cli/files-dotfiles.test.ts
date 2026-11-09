import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { collectVueLikeFilesFromTargets } from "./files.ts";

void test("directory targets retain Oxlint's dot-file selection", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-files-"));
  try {
    for (const file of [
      "src/App.vue",
      ".lintstagedrc.cjs",
      ".husky/pre-commit.js",
      "node_modules/pkg/index.js",
      ".git/hooks/pre-commit",
    ]) {
      fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
      fs.writeFileSync(path.join(root, file), "");
    }

    const candidates = new Set<string>();
    const vue = collectVueLikeFilesFromTargets(root, ["."], (file) => candidates.add(file));

    assert.deepEqual(vue, [path.join(root, "src/App.vue")]);
    assert.ok(candidates.has(path.join(root, ".lintstagedrc.cjs")));
    assert.ok(candidates.has(path.join(root, ".husky/pre-commit.js")));
    for (const excluded of [".git", "node_modules"]) {
      const prefix = `${path.join(root, excluded)}${path.sep}`;
      assert.ok(![...candidates].some((file) => file.startsWith(prefix)));
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test(
  "unreadable directories do not abort candidate discovery",
  {
    skip:
      process.platform === "win32" || process.getuid?.() === 0
        ? "permission bits do not make this directory unreadable"
        : false,
  },
  () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-files-"));
    const unreadable = path.join(root, "unreadable");
    try {
      fs.mkdirSync(unreadable);
      fs.writeFileSync(path.join(unreadable, "hidden.vue"), "");
      fs.writeFileSync(path.join(root, "visible.vue"), "");
      fs.chmodSync(unreadable, 0o000);

      assert.deepEqual(collectVueLikeFilesFromTargets(root, ["."]), [
        path.join(root, "visible.vue"),
      ]);
    } finally {
      fs.chmodSync(unreadable, 0o755);
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

void test(
  "directory symlink cycles are visited once",
  { skip: process.platform === "win32" ? "directory symlinks require elevated privileges" : false },
  () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-files-"));
    try {
      fs.mkdirSync(path.join(root, "src"));
      fs.writeFileSync(path.join(root, "src/App.vue"), "");
      fs.symlinkSync(root, path.join(root, "src/root"));

      assert.deepEqual(collectVueLikeFilesFromTargets(root, ["."]), [
        path.join(root, "src/App.vue"),
      ]);
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);

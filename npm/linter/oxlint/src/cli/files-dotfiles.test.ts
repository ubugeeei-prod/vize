import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { collectVueLikeFilesFromTargets } from "./files.ts";

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
    assert.ok(![...observed].some((file) => file.includes("node_modules")));
    assert.ok(![...observed].some((file) => file.includes(`${path.sep}.git${path.sep}`)));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

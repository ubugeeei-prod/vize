import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { collectVueInputPaths } from "../../legacy-tools/fixtures/tool-matrix-inputs.mjs";

function writeFile(root: string, relative: string, content = "<template />\n"): void {
  const file = path.join(root, relative);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

test("fixture Vue input collection includes hidden recursive directories", () => {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-inputs-"));
  try {
    writeFile(fixtureDir, "App.vue");
    writeFile(fixtureDir, ".storybook/Preview.vue");
    writeFile(fixtureDir, "docs/.vitepress/components/DownloadPage.vue");
    writeFile(fixtureDir, ".yarn/cache/Ignored.vue");
    writeFile(fixtureDir, "packages/app/node_modules/pkg/Ignored.vue");

    assert.deepEqual(collectVueInputPaths(fixtureDir, ["**/*.vue"]), [
      ".storybook/Preview.vue",
      "App.vue",
      "docs/.vitepress/components/DownloadPage.vue",
    ]);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
  }
});

test("fixture Vue input collection expands hidden dirs under scoped globs", () => {
  const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-fixture-inputs-"));
  try {
    writeFile(fixtureDir, "playground/src/App.vue");
    writeFile(fixtureDir, "playground/src/.vitepress/Doc.vue");
    writeFile(fixtureDir, "docs/.vitepress/components/Page.vue");

    assert.deepEqual(collectVueInputPaths(fixtureDir, ["playground/src/**/*.vue"]), [
      "playground/src/.vitepress/Doc.vue",
      "playground/src/App.vue",
    ]);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
  }
});

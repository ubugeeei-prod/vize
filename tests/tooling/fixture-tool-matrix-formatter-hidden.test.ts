import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { snapshotFormatterInputs } from "../../legacy-tools/fixtures/tool-matrix-formatter.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("formatter input snapshot covers hidden recursive Vue inputs", () => {
  const fixtureDir = fs.mkdtempSync(path.join(root, "tests", "_fixtures", "formatter-snapshot-"));
  const sourcePath = path.join(fixtureDir, "docs", ".vitepress", "components", "DownloadPage.vue");
  fs.mkdirSync(path.dirname(sourcePath), { recursive: true });
  fs.writeFileSync(sourcePath, "<template />\n");
  try {
    const before = snapshotFormatterInputs(fixtureDir, ["**/*.vue"]);
    fs.writeFileSync(sourcePath, "<template><main /></template>\n");
    assert.notEqual(snapshotFormatterInputs(fixtureDir, ["**/*.vue"]), before);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
  }
});

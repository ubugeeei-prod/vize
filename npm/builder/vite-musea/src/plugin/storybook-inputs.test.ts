import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  assertNoUnsupportedStorybookTsxInputs,
  formatUnsupportedStorybookTsxInputError,
  scanStorybookTsxInputs,
} from "./storybook-inputs.js";

void test("scanStorybookTsxInputs finds included TSX stories and honors excludes", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-storybook-tsx-"));
  try {
    const storyPath = path.join(tempDir, "src", "Button.stories.tsx");
    const excludedPath = path.join(tempDir, "node_modules", "pkg", "Ignored.stories.tsx");
    const artPath = path.join(tempDir, "src", "Button.art.vue");

    await fs.promises.mkdir(path.dirname(storyPath), { recursive: true });
    await fs.promises.mkdir(path.dirname(excludedPath), { recursive: true });
    await fs.promises.writeFile(storyPath, "export default {};\n", "utf8");
    await fs.promises.writeFile(excludedPath, "export default {};\n", "utf8");
    await fs.promises.writeFile(artPath, "<art />\n", "utf8");

    assert.deepEqual(
      await scanStorybookTsxInputs(
        tempDir,
        ["src/**/*.art.vue", "src/**/*.stories.tsx", "node_modules/**/*.stories.tsx"],
        ["node_modules/**"],
      ),
      [storyPath],
    );
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("formatUnsupportedStorybookTsxInputError points to migration path", () => {
  const message = formatUnsupportedStorybookTsxInputError("/repo", [
    "/repo/src/Button.stories.tsx",
  ]);

  assert.match(message, /Storybook TSX files matched by include/);
  assert.match(message, /src\/Button\.stories\.tsx/);
  assert.match(message, /\.art\.vue/);
});

void test("assertNoUnsupportedStorybookTsxInputs rejects matched TSX stories", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-storybook-tsx-error-"));
  try {
    const storyPath = path.join(tempDir, "src", "Button.stories.tsx");
    await fs.promises.mkdir(path.dirname(storyPath), { recursive: true });
    await fs.promises.writeFile(storyPath, "export default {};\n", "utf8");

    await assert.rejects(
      assertNoUnsupportedStorybookTsxInputs(tempDir, ["src/**/*.stories.tsx"], []),
      /Storybook TSX files matched by include/,
    );
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

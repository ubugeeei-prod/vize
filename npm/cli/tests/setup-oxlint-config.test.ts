import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { test } from "vite-plus/test";

import { setupProject } from "../src/setup.ts";

test("does not treat Oxlint config filenames that the binary ignores as configured", () => {
  const ignoredFilenames = [
    "oxlint.config.mts",
    "oxlint.config.js",
    "oxlint.config.mjs",
    "oxlint.config.cjs",
    "oxlint.config.cts",
  ] as const;

  for (const filename of ignoredFilenames) {
    const root = fs.mkdtempSync(
      path.join(os.tmpdir(), `vize-setup-ignored-${path.extname(filename).slice(1)}-`),
    );
    try {
      const ignoredConfig = `export default { rules: { "no-console": "error" } };\n`;
      fs.writeFileSync(
        path.join(root, "package.json"),
        `{"name":"ignored-oxlint-config","private":true}\n`,
      );
      fs.writeFileSync(path.join(root, filename), ignoredConfig);

      const result = setupProject({ root, install: false });

      assert.ok(result.createdFiles.includes("oxlint.config.ts"), filename);
      assert.ok(!result.preservedFiles.includes(filename), filename);
      assert.equal(fs.readFileSync(path.join(root, filename), "utf8"), ignoredConfig, filename);
      assert.match(
        fs.readFileSync(path.join(root, "oxlint.config.ts"), "utf8"),
        /from "oxlint-plugin-vize"/u,
        filename,
      );
    } finally {
      fs.rmSync(root, { force: true, recursive: true });
    }
  }
});

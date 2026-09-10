import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { test } from "vite-plus/test";

import { loadConfig } from "../src/config.ts";

test("loads TypeScript config self-imports without a root vize dependency", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-self-import-"));
  fs.writeFileSync(
    path.join(root, "vize.config.ts"),
    `import { defineConfig } from "vize";

export default defineConfig({
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
`,
  );

  const loaded = await loadConfig(root);

  assert.deepEqual(
    loaded?.vite.scanPatterns,
    ["src/**/*.vue"],
    "TypeScript config self-imports should not require a root vize dependency",
  );
});

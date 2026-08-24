import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  collectInspectorSourceFiles,
  createInspectorGraphPayload,
  isInspectorGraphRequest,
} from "./dev-middleware.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const workspaceRoot = path.resolve(__dirname, "../../../..");
const testRoot = path.join(
  workspaceRoot,
  "target",
  "vize-tests",
  "tests",
  "vite-plugin-vize",
  "dev-middleware",
);
fs.mkdirSync(testRoot, { recursive: true });
const root = fs.mkdtempSync(path.join(testRoot, "inspector-"));

fs.mkdirSync(path.join(root, "src"), { recursive: true });
fs.writeFileSync(
  path.join(root, "src", "App.vue"),
  `<script setup>
import UsedChild from "./UsedChild.vue";
import UnusedChild from "./UnusedChild.vue";
import("./lazy");
</script>
<template>
  <UsedChild />
</template>
`,
);
fs.writeFileSync(path.join(root, "src", "UsedChild.vue"), "<template><p>Used</p></template>\n");
fs.writeFileSync(path.join(root, "src", "UnusedChild.vue"), "<template><p>Unused</p></template>\n");
fs.writeFileSync(path.join(root, "src", "lazy.ts"), "export const value = 1;\n");
fs.writeFileSync(path.join(root, "src", "ignored.d.ts"), "export interface Ignored {}\n");

const state = {
  root,
  scanPatterns: ["src/**/*.vue"],
  ignorePatterns: [],
};

assert.equal(isInspectorGraphRequest("/__vize/inspector/graph"), true);
assert.equal(isInspectorGraphRequest("/__vize/inspector/graph?fresh=1"), true);
assert.equal(isInspectorGraphRequest("/__vize/other"), false);

const files = await collectInspectorSourceFiles(state);
assert.deepEqual(
  files.map((file) => file.path),
  ["src/App.vue", "src/UnusedChild.vue", "src/UsedChild.vue", "src/lazy.ts"],
);

const payload = await createInspectorGraphPayload(state);
assert.equal(payload.schema, "vize.inspector.graph");
assert.equal(payload.version, 1);
assert.equal(payload.fileCount, 4);

assert.deepEqual(
  payload.graph.edges.map((edge) => [edge.from, edge.to, edge.kind, edge.specifier]),
  [
    ["src/App.vue", "src/UnusedChild.vue", "import", "./UnusedChild.vue"],
    ["src/App.vue", "src/UsedChild.vue", "component", "./UsedChild.vue"],
    ["src/App.vue", "src/UsedChild.vue", "import", "./UsedChild.vue"],
    ["src/App.vue", "src/lazy.ts", "dynamic-import", "./lazy"],
  ],
);

void test("collectInspectorSourceFiles does not follow planted symlinks out of the project", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-inspector-symlink-"));
  const project = path.join(tempDir, "project");
  const outside = path.join(tempDir, "outside");

  try {
    fs.mkdirSync(path.join(project, "src"), { recursive: true });
    fs.mkdirSync(outside, { recursive: true });
    fs.writeFileSync(path.join(project, "src", "App.vue"), "<template><p>ok</p></template>\n");
    fs.writeFileSync(path.join(outside, "secret.vue"), "<template>secret</template>\n");
    fs.symlinkSync(path.join(outside, "secret.vue"), path.join(project, "src", "Leak.vue"));

    const collected = await collectInspectorSourceFiles({
      root: project,
      scanPatterns: ["src/**/*.vue"],
      ignorePatterns: [],
    });

    assert.deepEqual(
      collected.map((file) => file.path),
      ["src/App.vue"],
    );
    assert.equal(
      collected.some((file) => file.source.includes("secret")),
      false,
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});

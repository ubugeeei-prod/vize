import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
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

fs.writeFileSync(path.join(root, "outside.vue"), "<template><p>Outside</p></template>\n");
fs.symlinkSync(path.join(root, "outside.vue"), path.join(root, "src", "Escaped.vue"));

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

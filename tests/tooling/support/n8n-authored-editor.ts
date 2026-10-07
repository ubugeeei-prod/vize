import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { root, testOutputRoot } from "./lsp/paths.ts";

const fixture = path.join(root, "tests/_fixtures/differential/lsp/n8n-authored-editor");
export const consumer = `<script setup lang="ts">
import N8nBlockUi from './N8nBlockUi';
const visible = true;
</script>
<template><N8nBlockUi :show="visible" /></template>
`;

export function createN8nWorkspace(corsaPath: string) {
  const directory = path.join(testOutputRoot, "n8n-authored-editor");
  fs.mkdirSync(directory, { recursive: true });
  const workspace = fs.mkdtempSync(path.join(directory, "workspace-"));
  const provenance = JSON.parse(fs.readFileSync(path.join(fixture, "provenance.json"), "utf8")) as {
    files: Array<{ fixture: string; sha256: string; bytes: number }>;
  };
  for (const receipt of provenance.files) {
    const bytes = fs.readFileSync(path.join(fixture, receipt.fixture));
    assert.equal(bytes.length, receipt.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), receipt.sha256);
    const relative =
      receipt.fixture === "NullEmptyCellRenderer.vue.txt"
        ? "NullEmptyCellRenderer.vue"
        : path.join("N8nBlockUi", receipt.fixture.replace(/\.txt$/, ""));
    const target = path.join(workspace, relative);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes);
  }
  fs.writeFileSync(path.join(workspace, "App.vue"), consumer);
  const nodeModules = path.join(workspace, "node_modules");
  fs.mkdirSync(nodeModules);
  const vue = [path.join(root, "tests/node_modules/vue"), path.join(root, "node_modules/vue")].find(
    (candidate) => fs.existsSync(candidate),
  );
  assert.ok(vue, "the source Actions lane must install the Vue type dependency");
  fs.symlinkSync(
    vue,
    path.join(nodeModules, "vue"),
    process.platform === "win32" ? "junction" : "dir",
  );
  fs.writeFileSync(
    path.join(workspace, "vize.config.json"),
    JSON.stringify({
      lsp: { editor: true, lint: false, typecheck: true },
      typeChecker: { corsaPath },
    }),
  );
  fs.writeFileSync(
    path.join(workspace, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "bundler",
        strict: true,
        skipLibCheck: true,
        noEmit: true,
        types: [],
      },
      include: ["**/*.vue", "**/*.ts"],
    }),
  );
  return {
    workspace,
    source: fs.readFileSync(path.join(workspace, "NullEmptyCellRenderer.vue"), "utf8"),
  };
}

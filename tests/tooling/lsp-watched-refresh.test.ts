import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { requireTypecheckDependency } from "./support/typecheck-dependency.ts";

function resolveCorsaBinary(): string | undefined {
  return [
    path.join(root, "../corsa-bind/.cache/tsgo"),
    path.join(root, "node_modules/.bin/tsgo"),
    path.join(root, "tests/node_modules/.bin/tsgo"),
  ].find((candidate) => fs.existsSync(candidate));
}

const CHILD_NUMBER = `<script setup lang="ts">
defineProps<{ count: number }>();
</script>
<template><i>{{ count }}</i></template>
`;

const CHILD_STRING = CHILD_NUMBER.replace("count: number", "count: string");

const APP = `<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template>
  <Child :count="1" />
</template>
`;

// A dependency changed outside the editor — a git checkout, a codegen run, a
// delete — must refresh the open importer's diagnostics without any editor
// edit (#3918). The lifecycle mirrors the Volar oracle: clean on open, one
// mismatch after the prop type changes on disk, still broken while the file is
// gone, clean again once it is restored.
test("watched dependency changes refresh the open importer", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveCorsaBinary(),
    "Corsa runtime for the watched-refresh gate",
    "Corsa runtime is unavailable",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-watched-refresh");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  try {
    fs.mkdirSync(path.join(workspaceDir, "node_modules"), { recursive: true });
    fs.symlinkSync(
      path.join(root, "tests/node_modules/vue"),
      path.join(workspaceDir, "node_modules/vue"),
      process.platform === "win32" ? "junction" : "dir",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({ include: ["*.vue"], compilerOptions: { strict: true } }),
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "vize.config.json"),
      JSON.stringify({ typeChecker: { corsaPath } }),
      "utf8",
    );
    const childPath = path.join(workspaceDir, "Child.vue");
    fs.writeFileSync(childPath, CHILD_NUMBER, "utf8");

    await session.initialize(workspaceDir, {
      editor: true,
      typecheck: true,
      lint: false,
      autoInsert: false,
    });
    const appUri = pathToFileURL(path.join(workspaceDir, "App.vue")).href;
    const childUri = pathToFileURL(childPath).href;
    const diagnosticsFor = (uri: string) => (params: unknown) =>
      (params as { uri: string }).uri === uri;
    const counted = (params: unknown) => (params as { diagnostics: unknown[] }).diagnostics.length;
    // `waitForNotification` resolves out of the backlog, so a stale republish
    // could satisfy the delete phase without the deletion ever being observed.
    // Draining to quiescence first forces the next publish to be the delete's.
    const drainAppDiagnostics = async () => {
      for (;;) {
        try {
          await session.waitForNotification(
            "textDocument/publishDiagnostics",
            diagnosticsFor(appUri),
            1000,
          );
        } catch {
          return;
        }
      }
    };

    session.notify("textDocument/didOpen", {
      textDocument: { uri: appUri, languageId: "vue", version: 1, text: APP },
    });
    const opened = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      diagnosticsFor(appUri),
      60000,
    );
    assert.equal(counted(opened), 0, "the importer opens clean");

    // Phase 1: the dependency's prop narrows on disk, no editor edit.
    fs.writeFileSync(childPath, CHILD_STRING, "utf8");
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: childUri, type: 2 }],
    });
    const afterEdit = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && counted(params) > 0,
      60000,
    );
    assert.equal(counted(afterEdit), 1, "the stale binding is reported");

    // Phase 2: the dependency disappears; the importer must stay broken.
    await drainAppDiagnostics();
    fs.rmSync(childPath);
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: childUri, type: 3 }],
    });
    const afterDelete = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && counted(params) > 0,
      60000,
    );
    assert.ok(counted(afterDelete) > 0, "a deleted dependency keeps the importer broken");

    // Phase 3: restored with the matching type; the importer recovers.
    fs.writeFileSync(childPath, CHILD_NUMBER, "utf8");
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: childUri, type: 1 }],
    });
    const afterCreate = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && counted(params) === 0,
      60000,
    );
    assert.equal(counted(afterCreate), 0, "a restored dependency clears the importer");
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

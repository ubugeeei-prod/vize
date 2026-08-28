import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

function resolveCorsaBinary(): string | undefined {
  return resolveTypecheckRuntime(root);
}

const CHILD_NUMBER = `<script setup lang="ts">
defineProps<{ count: number }>();
</script>
<template><i>{{ count }}</i></template>
`;

const CHILD_STRING = CHILD_NUMBER.replace("count: number", "count: string");

const APP = `<script setup lang="ts">
import Child from "./components/Child.vue";
</script>
<template>
  <Child :count="1" />
</template>
`;

// A dependency changed outside the editor — a git checkout, a codegen run, a
// delete — must refresh the open importer's diagnostics without any editor
// edit (#3918). The lifecycle starts with a missing import, heals on a file
// operation create, reports a watched prop change, reports TS2307 on watched
// delete, and heals again on watched recreation.
test("created and watched dependency changes refresh the open importer", async (t) => {
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
    const componentsDir = path.join(workspaceDir, "components");
    const movedComponentsDir = path.join(workspaceDir, "moved-components");
    const childPath = path.join(componentsDir, "Child.vue");
    const renamedChildPath = path.join(componentsDir, "RenamedChild.vue");

    await session.initialize(workspaceDir, {
      editor: true,
      typecheck: true,
      lint: false,
      autoInsert: false,
    });
    const appUri = pathToFileURL(path.join(workspaceDir, "App.vue")).href;
    const childUri = pathToFileURL(childPath).href;
    const renamedChildUri = pathToFileURL(renamedChildPath).href;
    const componentsUri = pathToFileURL(componentsDir).href;
    const movedComponentsUri = pathToFileURL(movedComponentsDir).href;
    const diagnosticsFor = (uri: string) => (params: unknown) =>
      (params as { uri: string }).uri === uri;
    const counted = (params: unknown) => (params as { diagnostics: unknown[] }).diagnostics.length;
    const diagnosticCodes = (params: unknown) =>
      (params as { diagnostics: Array<{ code?: number | string }> }).diagnostics.map(
        (diagnostic) => diagnostic.code,
      );
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
    assert.ok(diagnosticCodes(opened).includes(2307), "the missing dependency starts as TS2307");

    // Phase 1: a file-operation create must heal the already-open importer at
    // the same document version; requiring an importer edit hides stale state.
    fs.mkdirSync(componentsDir);
    fs.writeFileSync(childPath, CHILD_NUMBER, "utf8");
    session.notify("workspace/didCreateFiles", {
      files: [{ uri: childUri }],
    });
    const afterCreate = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && counted(params) === 0,
      60000,
    );
    assert.equal((afterCreate as { version?: number }).version, 1);

    // Phase 2: a rename that removes the imported path must republish TS2307
    // without relying on the client to apply a willRename workspace edit.
    await drainAppDiagnostics();
    fs.renameSync(childPath, renamedChildPath);
    session.notify("workspace/didRenameFiles", {
      files: [{ oldUri: childUri, newUri: renamedChildUri }],
    });
    const afterRename = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && diagnosticCodes(params).includes(2307),
      60000,
    );
    assert.equal((afterRename as { version?: number }).version, 1);

    // Phase 3: reversing the rename recreates the imported path and must heal
    // the same importer version without an editor edit.
    fs.renameSync(renamedChildPath, childPath);
    session.notify("workspace/didRenameFiles", {
      files: [{ oldUri: renamedChildUri, newUri: childUri }],
    });
    const afterReverseRename = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && counted(params) === 0,
      60000,
    );
    assert.equal((afterReverseRename as { version?: number }).version, 1);

    // Phase 4: a directory rename must invalidate nested dependencies, not
    // only files whose URI exactly matches an indexed import.
    await drainAppDiagnostics();
    fs.renameSync(componentsDir, movedComponentsDir);
    session.notify("workspace/didRenameFiles", {
      files: [{ oldUri: componentsUri, newUri: movedComponentsUri }],
    });
    const afterDirectoryRename = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && diagnosticCodes(params).includes(2307),
      60000,
    );
    assert.equal((afterDirectoryRename as { version?: number }).version, 1);

    // Phase 5: restoring the directory must heal the same importer version.
    fs.renameSync(movedComponentsDir, componentsDir);
    session.notify("workspace/didRenameFiles", {
      files: [{ oldUri: movedComponentsUri, newUri: componentsUri }],
    });
    const afterReverseDirectoryRename = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && counted(params) === 0,
      60000,
    );
    assert.equal((afterReverseDirectoryRename as { version?: number }).version, 1);

    // Phase 6: the dependency's prop narrows on disk, no editor edit.
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

    // Phase 7: the dependency disappears; the importer must stay broken.
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
    assert.ok(
      diagnosticCodes(afterDelete).includes(2307),
      `a deleted dependency reports TS2307 instead of a stale overlay diagnostic: ${JSON.stringify(
        afterDelete,
      )}`,
    );

    // Phase 8: restored with the matching type; the importer recovers.
    fs.writeFileSync(childPath, CHILD_NUMBER, "utf8");
    session.notify("workspace/didChangeWatchedFiles", {
      changes: [{ uri: childUri, type: 1 }],
    });
    const afterWatchedRestore = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && counted(params) === 0,
      60000,
    );
    assert.equal(counted(afterWatchedRestore), 0, "a restored dependency clears the importer");

    // Phase 9: a directory file-operation delete must evict every nested Vue
    // overlay and republish the open importer at the same document version.
    await drainAppDiagnostics();
    fs.rmSync(componentsDir, { recursive: true });
    session.notify("workspace/didDeleteFiles", {
      files: [{ uri: componentsUri }],
    });
    const afterDirectoryDelete = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && diagnosticCodes(params).includes(2307),
      60000,
    );
    assert.equal((afterDirectoryDelete as { version?: number }).version, 1);

    // Phase 10: recreating the directory and nested dependency heals without
    // requiring a file-level event or an importer edit.
    fs.mkdirSync(componentsDir);
    fs.writeFileSync(childPath, CHILD_NUMBER, "utf8");
    session.notify("workspace/didCreateFiles", {
      files: [{ uri: componentsUri }],
    });
    const afterDirectoryCreate = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => diagnosticsFor(appUri)(params) && counted(params) === 0,
      60000,
    );
    assert.equal((afterDirectoryCreate as { version?: number }).version, 1);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

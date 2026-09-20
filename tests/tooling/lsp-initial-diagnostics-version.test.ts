import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { workspace } from "./support/upstream/vue-language-tools.ts";

test("initial sync diagnostics publish before terminal versioned diagnostics", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-initial-diagnostics-version");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    fs.writeFileSync(
      path.join(workspaceDir, "vize.config.json"),
      JSON.stringify({
        lsp: { lint: true, typecheck: true },
        typeChecker: {
          corsaPath: "./vize-missing-corsa-for-initial-sync-diagnostics",
        },
      }),
      "utf8",
    );
    await session.initialize(workspaceDir, {
      editor: true,
      lint: true,
      typecheck: true,
    });

    const filePath = path.join(workspaceDir, "InitialSync.vue");
    const uri = pathToFileURL(filePath).href;
    const text = `<template><div /></template>
<style>.root { color: red; }</style>
`;
    fs.writeFileSync(filePath, text, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text },
    });

    const initial = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        (params as PublishDiagnosticsParams).version == null &&
        (params as PublishDiagnosticsParams).diagnostics.length > 0,
      10000,
    )) as PublishDiagnosticsParams;

    assert.equal(initial.version, undefined);
    assert.ok(initial.diagnostics.some((diagnostic) => diagnostic.source === "vize/lint"));

    const terminal = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) && (params as PublishDiagnosticsParams).version === 1,
      10000,
    )) as PublishDiagnosticsParams;

    assert.equal(terminal.version, 1);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("coalesced initial diagnostics retain unsaved dependency types and repairs", async () => {
  const directory = workspace("initial-unsaved-dependency-");
  const session = new LspSession();
  const child = pathToFileURL(path.join(directory, "Child.vue")).href;
  const parent = pathToFileURL(path.join(directory, "Parent.vue")).href;
  const disk = '<script setup lang="ts">defineProps<{ value: string }>();</script><template />';
  const source = `<script setup lang="ts">import Child from './Child.vue';</script>
<template><Child value="text" /></template>`;
  const wait = async (uri: string, version: number): Promise<PublishDiagnosticsParams> =>
    (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === version,
      10_000,
    )) as PublishDiagnosticsParams;
  try {
    fs.writeFileSync(path.join(directory, "Child.vue"), disk);
    fs.writeFileSync(path.join(directory, "Parent.vue"), source);
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          skipLibCheck: true,
          target: "ESNext",
          moduleResolution: "Bundler",
          module: "ESNext",
        },
        include: ["*.vue"],
      }),
    );
    await session.initialize(directory, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: child,
        languageId: "vue",
        version: 1,
        text: disk.replace("string", "number"),
      },
    });
    session.notify("textDocument/didOpen", {
      textDocument: { uri: parent, languageId: "vue", version: 1, text: source },
    });
    assert.deepEqual((await wait(child, 1)).diagnostics, []);
    assert.deepEqual(
      (await wait(parent, 1)).diagnostics.map((d) => Number(d.code)),
      [2322],
    );
    session.notify("textDocument/didChange", {
      textDocument: { uri: child, version: 2 },
      contentChanges: [{ text: disk }],
    });
    assert.deepEqual((await wait(child, 2)).diagnostics, []);
    assert.deepEqual((await wait(parent, 1)).diagnostics, []);
  } finally {
    await session.shutdown();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

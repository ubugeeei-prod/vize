import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

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

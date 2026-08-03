import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspRequestError, LspSession } from "./support/lsp/session.ts";

const SOURCE = `<script setup lang="ts">
const cancellableSymbol = 1
</script>
<template><p>{{ cancellableSymbol }}</p></template>
`;

test("vize lsp cancels an in-flight request and remains usable", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-request-cancellation");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const filePath = path.join(workspaceDir, "Cancellable.vue");
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, SOURCE, "utf8");

  const session = new LspSession();
  let primaryError: unknown;
  try {
    await session.initialize(workspaceDir, { editor: true, lint: false, typecheck: false });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: SOURCE },
    });
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const requestId = session.nextRequestId;
    const cancelled = session.request("workspace/symbol", { query: "cancellableSymbol" });
    session.notify("$/cancelRequest", { id: requestId });

    await assert.rejects(
      cancelled,
      (error) =>
        error instanceof LspRequestError &&
        error.method === "workspace/symbol" &&
        error.code === -32800,
    );

    const retry = (await session.request("workspace/symbol", {
      query: "cancellableSymbol",
    })) as Array<{ name: string }> | null;
    assert.ok(
      retry?.some((symbol) => symbol.name === "cancellableSymbol"),
      JSON.stringify(retry),
    );
  } catch (error) {
    primaryError = error;
    throw error;
  } finally {
    await session.shutdown().catch((error: unknown) => {
      if (primaryError == null) throw error;
    });
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

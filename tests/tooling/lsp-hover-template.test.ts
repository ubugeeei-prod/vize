import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

// Hover over a template interpolation of a script `ref` is vize-native
// analysis (typecheck:false, no corsa). The lsp-smoke suite already covers
// hover inside `.art.vue` variants; this case covers a plain `.vue` template
// interpolation and asserts the "Template binding from script" provenance note
// that the smoke suite does not check.
test("vize lsp hovers template interpolation of a script ref", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-hover-template");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: true,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
import { ref } from 'vue'
const greeting = ref('hello')
</script>

<template>
  <p>{{ greeting }}</p>
</template>
`;
    const filePath = path.join(workspaceDir, "HoverTemplate.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: 1,
        text: source,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    const usageOffset = source.lastIndexOf("greeting") + "greeting".length;

    const hover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: offsetToPosition(source, usageOffset),
    })) as { contents?: unknown } | null;

    const hoverText = hoverToText(hover);
    assert.match(hoverText, /greeting/);
    assert.match(hoverText, /Ref<string>/);
    assert.match(hoverText, /Template binding from script/);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

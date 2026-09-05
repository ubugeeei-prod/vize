import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type SymbolInformation = {
  name: string;
  kind: number;
  containerName?: string;
  location: {
    uri: string;
    range: {
      start: { line: number; character: number };
      end: { line: number; character: number };
    };
  };
};

test("vize lsp workspaceSymbol locates defineEmits event declarations", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-workspace-symbol-emits");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
const emit = defineEmits<{
  (event: "update:modelValue", value: string): void
}>()
</script>

<template>
  <button @click="emit('update:modelValue', 'next')" />
</template>
`;
    const filePath = path.join(workspaceDir, "Emitter.vue");
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

    const symbols = (await session.request("workspace/symbol", {
      query: "update:modelValue",
    })) as SymbolInformation[] | null;
    assert.equal(symbols?.length, 1, JSON.stringify(symbols));

    const [symbol] = symbols;
    assert.equal(symbol.name, "update:modelValue");
    assert.equal(symbol.kind, 24);
    assert.equal(symbol.containerName, "Emitter");
    assert.equal(symbol.location.uri, uri);

    const eventNameOffset = source.indexOf('"update:modelValue"') + 1;
    assert.deepEqual(symbol.location.range.start, offsetToPosition(source, eventNameOffset));
    assert.deepEqual(
      symbol.location.range.end,
      offsetToPosition(source, eventNameOffset + "update:modelValue".length),
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

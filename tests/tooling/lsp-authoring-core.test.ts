import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  completionLabels,
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

test("vize lsp supports production Vue authoring requests in one editor session", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-authoring-core");
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
const count = ref(0)
const items = [1, 2]
</script>

<template>
  <button
    @click="
      () => {
        co
      }
    "
  >
    {{ count }}
  </button>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
`;
    const filePath = path.join(workspaceDir, "AuthoringCore.vue");
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

    const publish = (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) =>
        isDiagnosticsForUri(params, uri) &&
        params.diagnostics.some(
          (diagnostic) =>
            diagnostic.source === "vize/lint" && diagnostic.code === "vue/require-v-for-key",
        ),
    )) as PublishDiagnosticsParams;
    assert.ok(publish.diagnostics.length > 0, JSON.stringify(publish.diagnostics));

    const completionOffset = source.indexOf("        co") + "        co".length;
    const completion = await session.request("textDocument/completion", {
      textDocument: { uri },
      position: offsetToPosition(source, completionOffset),
    });
    const completionItems = completionLabels(completion);
    assert.ok(completionItems.includes("count"), completionItems.join(", "));
    assert.ok(!completionItems.includes("v-if"), completionItems.join(", "));

    const clickOffset = source.indexOf("@click") + "@click".length;
    const clickHover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: offsetToPosition(source, clickOffset),
    })) as { contents?: unknown } | null;
    const clickHoverText = hoverToText(clickHover);
    assert.match(clickHoverText, /Vue event listener/);
    assert.match(clickHoverText, /@click="handler"/);
    assert.match(clickHoverText, /\$event/);
    assert.match(clickHoverText, /\*\*Example\*\*/);
    assert.match(clickHoverText, /```vue/);
    assert.match(clickHoverText, /Vue Event Handling/);

    const countUsageStart = source.lastIndexOf("count }}");
    const countUsagePosition = offsetToPosition(source, countUsageStart + "count".length);

    const hover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: countUsagePosition,
    })) as { contents?: unknown } | null;
    const hoverText = hoverToText(hover);
    assert.match(hoverText, /count/);
    assert.match(hoverText, /Template binding from script|Ref<number>|<script setup>/);
    assert.match(hoverText, /automatically unwrapped|template scope/i);

    const definition = (await session.request("textDocument/definition", {
      textDocument: { uri },
      position: countUsagePosition,
    })) as
      | Array<{ uri: string; range: { start: { line: number; character: number } } }>
      | { uri: string; range: { start: { line: number; character: number } } };
    const declarationOffset = source.indexOf("count = ref");
    const declarationPosition = offsetToPosition(source, declarationOffset);
    const definitionLocation = firstLocation(definition);
    assert.equal(definitionLocation.uri, uri);
    assert.deepEqual(definitionLocation.range.start, declarationPosition);

    const forAliasOffset = source.indexOf('v-for="item in items"') + 'v-for="'.length;
    const forUsageOffset = source.lastIndexOf("{{ item }}") + "{{ ".length + "item".length;
    const forUsagePosition = offsetToPosition(source, forUsageOffset);

    const forHover = (await session.request("textDocument/hover", {
      textDocument: { uri },
      position: forUsagePosition,
    })) as { contents?: unknown } | null;
    const forHoverText = hoverToText(forHover);
    assert.match(forHoverText, /item/);
    assert.match(forHoverText, /v-for scope binding/);
    assert.match(forHoverText, /nearest `v-for`/);

    const forDefinition = await session.request("textDocument/definition", {
      textDocument: { uri },
      position: forUsagePosition,
    });
    const forDefinitionLocation = firstLocation(forDefinition as never);
    assert.equal(forDefinitionLocation.uri, uri);
    assert.deepEqual(forDefinitionLocation.range.start, offsetToPosition(source, forAliasOffset));

    const references = (await session.request("textDocument/references", {
      textDocument: { uri },
      position: countUsagePosition,
      context: {
        includeDeclaration: true,
      },
    })) as Array<{ uri: string; range: { start: { line: number; character: number } } }>;
    assert.ok(
      references.some(
        (reference) =>
          reference.uri === uri &&
          reference.range.start.line === declarationPosition.line &&
          reference.range.start.character === declarationPosition.character,
      ),
      JSON.stringify(references),
    );
    assert.ok(
      references.some(
        (reference) =>
          reference.uri === uri &&
          reference.range.start.line === countUsagePosition.line &&
          reference.range.start.character === countUsagePosition.character - "count".length,
      ),
      JSON.stringify(references),
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

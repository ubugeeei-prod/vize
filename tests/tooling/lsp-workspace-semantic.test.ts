import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type SymbolInformation = {
  name: string;
  kind: number;
  containerName?: string;
  location: { uri: string; range: { start: { line: number; character: number } } };
};

/**
 * workspaceSymbol only indexes documents that are currently open in the
 * editor (the server iterates `state.documents`), and classifies script-setup
 * bindings by a lightweight shape heuristic:
 *   - `const x = ref(0)` / computed/reactive -> VARIABLE (13)
 *   - `function foo()` -> FUNCTION (12)
 *   - the component name derived from the file name -> CLASS (5)
 * An empty result set is returned as `null` (not an empty array).
 */
test("vize lsp workspaceSymbol indexes open .vue docs and classifies binding kinds", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-workspace-symbol");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });

    // Editor.vue holds the script-setup bindings under test. The component
    // name derived from this file is "Editor", which shares no substring with
    // the binding queries, so binding lookups stay unambiguous.
    const editorSource = `<script setup lang="ts">
import { ref } from 'vue'

const myCounter = ref(0)
function handleSubmit() {}
</script>

<template>
  <button @click="handleSubmit">{{ myCounter }}</button>
</template>
`;
    const editorPath = path.join(workspaceDir, "Editor.vue");
    const editorUri = pathToFileURL(editorPath).href;
    fs.writeFileSync(editorPath, editorSource, "utf8");

    // Widget.vue is opened to exercise the component-name (file-name derived)
    // symbol path, classified as CLASS.
    const widgetSource = `<script setup lang="ts">
const internalState = ref(1)
</script>

<template>
  <span>{{ internalState }}</span>
</template>
`;
    const widgetPath = path.join(workspaceDir, "Widget.vue");
    const widgetUri = pathToFileURL(widgetPath).href;
    fs.writeFileSync(widgetPath, widgetSource, "utf8");

    // Before any document is open the server has nothing to index.
    const beforeOpen = await session.request("workspace/symbol", { query: "myCounter" });
    assert.equal(beforeOpen, null);

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: editorUri,
        languageId: "vue",
        version: 1,
        text: editorSource,
      },
    });
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: widgetUri,
        languageId: "vue",
        version: 1,
        text: widgetSource,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, editorUri),
    );
    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, widgetUri),
    );

    // A ref()-initialized binding is classified as VARIABLE (13) with the
    // script-setup container.
    const counterSymbols = (await session.request("workspace/symbol", {
      query: "myCounter",
    })) as SymbolInformation[] | null;
    assert.ok(Array.isArray(counterSymbols), JSON.stringify(counterSymbols));
    assert.equal(counterSymbols.length, 1, JSON.stringify(counterSymbols));
    assert.equal(counterSymbols[0].name, "myCounter");
    assert.equal(counterSymbols[0].kind, 13);
    assert.equal(counterSymbols[0].containerName, "script setup");
    assert.equal(counterSymbols[0].location.uri, editorUri);

    // A `function` declaration is classified as FUNCTION (12).
    const handlerSymbols = (await session.request("workspace/symbol", {
      query: "handleSubmit",
    })) as SymbolInformation[] | null;
    assert.ok(Array.isArray(handlerSymbols), JSON.stringify(handlerSymbols));
    assert.equal(handlerSymbols.length, 1, JSON.stringify(handlerSymbols));
    assert.equal(handlerSymbols[0].name, "handleSubmit");
    assert.equal(handlerSymbols[0].kind, 12);
    assert.equal(handlerSymbols[0].containerName, "script setup");
    assert.equal(handlerSymbols[0].location.uri, editorUri);

    // The component name (PascalCase of the file stem) is classified as
    // CLASS (5) and located at the file head.
    const widgetSymbols = (await session.request("workspace/symbol", {
      query: "Widget",
    })) as SymbolInformation[] | null;
    assert.ok(Array.isArray(widgetSymbols), JSON.stringify(widgetSymbols));
    const widgetSymbol = widgetSymbols.find((symbol) => symbol.name === "Widget");
    assert.ok(widgetSymbol, JSON.stringify(widgetSymbols));
    assert.equal(widgetSymbol.kind, 5);
    assert.equal(widgetSymbol.location.uri, widgetUri);
    assert.deepEqual(widgetSymbol.location.range.start, { line: 0, character: 0 });

    // A query with no matches returns null, not an empty array.
    const unmatched = await session.request("workspace/symbol", {
      query: "zzzNoSuchSymbol",
    });
    assert.equal(unmatched, null);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

/**
 * semanticTokens/full advertises a fixed legend and returns a flat 5-tuple
 * token stream. The capability declares both full and range support without a
 * delta channel. An empty document parses successfully and yields an empty
 * token array (distinct from the `null` returned for unopened documents).
 */
test("vize lsp semanticTokens/full returns 5-tuple data and advertises full+range", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-semantic-tokens");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    })) as {
      capabilities?: {
        semanticTokensProvider?: {
          range?: unknown;
          full?: unknown;
          legend?: {
            tokenTypes?: string[];
            tokenModifiers?: string[];
          };
        };
      };
    };

    const provider = init.capabilities?.semanticTokensProvider;
    assert.ok(provider, JSON.stringify(init.capabilities));

    // full is advertised as a strict boolean `true` (no delta channel), and
    // range support is enabled.
    assert.strictEqual(provider.full, true);
    assert.strictEqual(provider.range, true);

    // The legend is a fixed contract: 23 token types (namespace first) and 10
    // token modifiers.
    assert.equal(provider.legend?.tokenTypes?.length, 23);
    assert.equal(provider.legend?.tokenTypes?.[0], "namespace");
    assert.equal(provider.legend?.tokenModifiers?.length, 10);

    const source = `<script setup lang="ts">
const greeting = "hello"
</script>

<template>
  <p>{{ greeting }}</p>
</template>
`;
    const filePath = path.join(workspaceDir, "Tokens.vue");
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

    const tokens = (await session.request("textDocument/semanticTokens/full", {
      textDocument: { uri },
    })) as { data?: unknown[] } | null;

    assert.ok(tokens, JSON.stringify(tokens));
    assert.ok(Array.isArray(tokens.data), JSON.stringify(tokens));
    assert.ok(tokens.data.length > 0, JSON.stringify(tokens.data));
    assert.equal(tokens.data.length % 5, 0, JSON.stringify(tokens.data));

    // An empty (but parsed) .vue document yields an empty token array.
    const emptyPath = path.join(workspaceDir, "Empty.vue");
    const emptyUri = pathToFileURL(emptyPath).href;
    fs.writeFileSync(emptyPath, "", "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: emptyUri,
        languageId: "vue",
        version: 1,
        text: "",
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, emptyUri),
    );

    const emptyTokens = (await session.request("textDocument/semanticTokens/full", {
      textDocument: { uri: emptyUri },
    })) as { data?: unknown[] } | null;

    assert.ok(emptyTokens, JSON.stringify(emptyTokens));
    assert.deepEqual(emptyTokens.data, []);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

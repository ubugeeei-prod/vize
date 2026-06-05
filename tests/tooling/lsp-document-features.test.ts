import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

/**
 * Characterization tests for the parse-driven document features exposed by
 * `vize lsp`: `textDocument/documentSymbol` and `textDocument/foldingRange`.
 *
 * Both handlers run purely off the SFC parser (no type checking), so the
 * descriptors they emit are deterministic functions of the block layout. Every
 * assertion below mirrors what the production server actually returns for the
 * given fixture, including discovery order, MODULE symbol kinds, selection-range
 * widths, CRLF line math, and the omission of folding-range character fields.
 */

type DocumentSymbol = {
  name: string;
  kind: number;
  detail?: string;
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
  selectionRange: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
};

type FoldingRange = {
  startLine: number;
  endLine: number;
  startCharacter?: number;
  endCharacter?: number;
  kind?: string;
  collapsedText?: string;
};

async function withSession(
  name: string,
  run: (ctx: { session: LspSession; workspaceDir: string }) => Promise<void>,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, name);
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: false,
    });
    await run({ session, workspaceDir });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

async function openDocument(
  session: LspSession,
  workspaceDir: string,
  fileName: string,
  text: string,
  languageId = "vue",
): Promise<{ uri: string; publish: PublishDiagnosticsParams }> {
  const filePath = path.join(workspaceDir, fileName);
  const uri = pathToFileURL(filePath).href;
  fs.writeFileSync(filePath, text, "utf8");

  session.notify("textDocument/didOpen", {
    textDocument: { uri, languageId, version: 1, text },
  });

  const publish = (await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
    isDiagnosticsForUri(params, uri),
  )) as PublishDiagnosticsParams;

  return { uri, publish };
}

function requestDocumentSymbol(session: LspSession, uri: string): Promise<DocumentSymbol[] | null> {
  return session.request("textDocument/documentSymbol", {
    textDocument: { uri },
  }) as Promise<DocumentSymbol[] | null>;
}

function requestFoldingRange(session: LspSession, uri: string): Promise<FoldingRange[] | null> {
  return session.request("textDocument/foldingRange", {
    textDocument: { uri },
  }) as Promise<FoldingRange[] | null>;
}

const SYMBOL_KIND_MODULE = 2;

test("documentSymbol returns MODULE entries for template/script-setup/style", async () => {
  await withSession("lsp-docfeat-symbols-basic", async ({ session, workspaceDir }) => {
    const source = `<script setup lang="ts">
const count = ref(0)
</script>

<template>
  <div>{{ count }}</div>
</template>

<style scoped>
.a {
  color: red;
}
</style>
`;
    const { uri } = await openDocument(session, workspaceDir, "Basic.vue", source);
    const symbols = await requestDocumentSymbol(session, uri);

    assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
    assert.equal(symbols.length, 3);

    // Discovery order is fixed: template, script setup (no plain <script>), styles.
    assert.deepEqual(
      symbols.map((symbol) => symbol.name),
      ["template", "script setup", "style scoped"],
    );

    for (const symbol of symbols) {
      assert.equal(symbol.kind, SYMBOL_KIND_MODULE);
    }

    const byName = new Map(symbols.map((symbol) => [symbol.name, symbol]));
    // Only script-setup carries a lang detail; template/style have none here.
    assert.equal(byName.get("script setup")?.detail, "ts");
    assert.equal("detail" in (byName.get("template") as object), false);
    assert.equal("detail" in (byName.get("style scoped") as object), false);

    // selectionRange end-character is a fixed per-block width.
    assert.equal(byName.get("template")?.selectionRange.end.character, 10);
    assert.equal(byName.get("script setup")?.selectionRange.end.character, 14);
    assert.equal(byName.get("style scoped")?.selectionRange.end.character, 7);

    // script setup opens the file, so its block range starts on line 0.
    assert.equal(byName.get("script setup")?.range.start.line, 0);
  });
});

test("documentSymbol surfaces all block kinds and labels styles by scoped/module", async () => {
  await withSession("lsp-docfeat-symbols-all-blocks", async ({ session, workspaceDir }) => {
    const source = `<script lang="ts">
export default {}
</script>

<script setup lang="ts">
const x = 1
</script>

<template>
  <div>{{ x }}</div>
</template>

<style scoped>
.a {
  color: red;
}
</style>

<style module="m">
.b {
  color: blue;
}
</style>
`;
    const { uri } = await openDocument(session, workspaceDir, "AllBlocks.vue", source);
    const symbols = await requestDocumentSymbol(session, uri);

    assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
    assert.equal(symbols.length, 5);

    assert.deepEqual(
      symbols.map((symbol) => symbol.name),
      ["template", "script", "script setup", "style scoped", "style module=m"],
    );

    for (const symbol of symbols) {
      assert.equal(symbol.kind, SYMBOL_KIND_MODULE);
    }

    // detail is the block lang and only set for the two script blocks here.
    assert.deepEqual(
      symbols.map((symbol) => symbol.detail),
      [undefined, "ts", "ts", undefined, undefined],
    );
  });
});

test("documentSymbol distinguishes unopened (null) from empty-open (empty array)", async () => {
  await withSession("lsp-docfeat-symbols-empty", async ({ session, workspaceDir }) => {
    const unopenedUri = pathToFileURL(path.join(workspaceDir, "NeverOpened.vue")).href;
    const unopenedSymbols = await requestDocumentSymbol(session, unopenedUri);
    assert.equal(unopenedSymbols, null);

    const { uri } = await openDocument(session, workspaceDir, "Empty.vue", "");
    const emptySymbols = await requestDocumentSymbol(session, uri);
    assert.ok(Array.isArray(emptySymbols), JSON.stringify(emptySymbols));
    assert.equal(emptySymbols.length, 0);

    // No foldable blocks -> folding range is null on the same parsed-but-empty doc.
    const emptyFolding = await requestFoldingRange(session, uri);
    assert.equal(emptyFolding, null);
  });
});

test("documentSymbol line math is correct for a CRLF-terminated SFC", async () => {
  await withSession("lsp-docfeat-symbols-crlf", async ({ session, workspaceDir }) => {
    // script setup occupies lines 0-2, blank line 3, template occupies lines 4-6.
    const lines = [
      `<script setup lang="ts">`,
      `const a = 1`,
      `</script>`,
      ``,
      `<template>`,
      `  <div>{{ a }}</div>`,
      `</template>`,
    ];
    const source = lines.join("\r\n") + "\r\n";

    const { uri } = await openDocument(session, workspaceDir, "Crlf.vue", source);
    const symbols = await requestDocumentSymbol(session, uri);

    assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
    const byName = new Map(symbols.map((symbol) => [symbol.name, symbol]));

    const scriptSetup = byName.get("script setup");
    assert.ok(scriptSetup, JSON.stringify(symbols));
    assert.equal(scriptSetup.range.start.line, 0);
    assert.equal(scriptSetup.range.end.line, 2);

    const template = byName.get("template");
    assert.ok(template, JSON.stringify(symbols));
    assert.equal(template.range.start.line, 4);
    assert.equal(template.range.end.line, 6);
  });
});

test("documentSymbol on an art-vue file lists only the script setup block", async () => {
  await withSession("lsp-docfeat-symbols-art", async ({ session, workspaceDir }) => {
    fs.writeFileSync(
      path.join(workspaceDir, "Child.vue"),
      `<script setup lang="ts"></script>
<template><button /></template>
`,
      "utf8",
    );

    const artSource = `<script setup lang="ts">
import { ref } from 'vue'

const label = ref('label')
</script>

<art title="Button" component="./Child.vue">
  <variant name="Primary" default>
    <Child>{{ label }}</Child>
  </variant>
</art>
`;
    const { uri, publish } = await openDocument(
      session,
      workspaceDir,
      "Button.art.vue",
      artSource,
      "art-vue",
    );

    // didOpen publishes an empty diagnostics set at version 1 for the valid art file.
    assert.equal(publish.version, 1);
    assert.deepEqual(publish.diagnostics, []);

    const symbols = await requestDocumentSymbol(session, uri);
    assert.ok(Array.isArray(symbols), JSON.stringify(symbols));
    assert.equal(symbols.length, 1);
    assert.equal(symbols[0].name, "script setup");
    assert.equal(symbols[0].kind, SYMBOL_KIND_MODULE);
    assert.equal(symbols[0].detail, "ts");
  });
});

test("foldingRange emits one region per multi-line block with block-named collapsedText", async () => {
  await withSession("lsp-docfeat-folding-regions", async ({ session, workspaceDir }) => {
    const source = `<script setup lang="ts">
const x = 1
</script>

<template>
  <div>{{ x }}</div>
</template>

<style scoped>
.a {
  color: red;
}
</style>

<style module="m">
.b {
  color: blue;
}
</style>
`;
    const { uri } = await openDocument(session, workspaceDir, "Folding.vue", source);
    const ranges = await requestFoldingRange(session, uri);

    assert.ok(Array.isArray(ranges), JSON.stringify(ranges));

    // template, script setup, then the two styles (styles always collapse to "style").
    assert.deepEqual(
      ranges.map((range) => range.collapsedText),
      ["template", "script setup", "style", "style"],
    );

    for (const range of ranges) {
      assert.equal(range.kind, "region");
      // Block folds span lines only; the character fields are not serialized.
      assert.equal(range.startCharacter, undefined);
      assert.equal(range.endCharacter, undefined);
      assert.ok(
        range.startLine < range.endLine,
        `expected startLine < endLine for ${range.collapsedText}: ${JSON.stringify(range)}`,
      );
    }
  });
});

test("foldingRange omits single-line blocks and returns null when nothing is foldable", async () => {
  await withSession("lsp-docfeat-folding-singleline", async ({ session, workspaceDir }) => {
    // Entire template lives on line 0, so there is nothing to fold.
    const source = "<template><div/></template>\n";
    const { uri } = await openDocument(session, workspaceDir, "OneLine.vue", source);

    const ranges = await requestFoldingRange(session, uri);
    assert.equal(ranges, null);
  });
});

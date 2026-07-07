import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  FULL_RANGE,
  assertCodeLensWorks,
  assertDefinitionWorks,
  assertDocumentLinksWork,
  assertDocumentSymbolsWork,
  assertFoldingRangesWork,
  assertHoverWorks,
  assertInlayHintsWork,
  assertReferencesWork,
  assertSemanticTokensWork,
  assertWorkspaceSymbolsWork,
  completionStylePosition,
  expectNull,
  templateMessagePosition,
  withFeatureDocument,
} from "./support/lsp/feature-isolation.ts";

test("granular hover disable does not shut down structural and index features", async () => {
  await withFeatureDocument("hover-off", { hover: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/hover", {
      textDocument: { uri: ctx.uri },
      position: templateMessagePosition(ctx.source),
    });
    await assertDocumentSymbolsWork(ctx);
    await assertWorkspaceSymbolsWork(ctx);
    await assertSemanticTokensWork(ctx);
    await assertDocumentLinksWork(ctx);
  });
});

test("granular documentSymbols disable leaves hover workspace symbols and folding alive", async () => {
  await withFeatureDocument("document-symbols-off", { documentSymbols: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/documentSymbol", { textDocument: { uri: ctx.uri } });
    await assertHoverWorks(ctx);
    await assertWorkspaceSymbolsWork(ctx);
    await assertFoldingRangesWork(ctx);
  });
});

test("granular workspaceSymbols disable leaves document-local authoring alive", async () => {
  await withFeatureDocument("workspace-symbols-off", { workspaceSymbols: false }, async (ctx) => {
    await expectNull(ctx, "workspace/symbol", { query: "submitMessage" });
    await assertHoverWorks(ctx);
    await assertDocumentSymbolsWork(ctx);
    await assertCodeLensWorks(ctx);
  });
});

test("granular semanticTokens disable leaves parser-backed ranges alive", async () => {
  await withFeatureDocument("semantic-tokens-off", { semanticTokens: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/semanticTokens/full", { textDocument: { uri: ctx.uri } });
    await expectNull(ctx, "textDocument/semanticTokens/range", {
      textDocument: { uri: ctx.uri },
      range: FULL_RANGE,
    });
    await assertDocumentSymbolsWork(ctx);
    await assertFoldingRangesWork(ctx);
  });
});

test("granular foldingRanges disable leaves semantic tokens and symbols alive", async () => {
  await withFeatureDocument("folding-ranges-off", { foldingRanges: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/foldingRange", { textDocument: { uri: ctx.uri } });
    await assertSemanticTokensWork(ctx);
    await assertDocumentSymbolsWork(ctx);
  });
});

test("granular inlayHints disable leaves code lenses and hover alive", async () => {
  await withFeatureDocument("inlay-hints-off", { inlayHints: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/inlayHint", {
      textDocument: { uri: ctx.uri },
      range: FULL_RANGE,
    });
    await assertCodeLensWorks(ctx);
    await assertHoverWorks(ctx);
  });
});

test("granular codeLens disable leaves inlay hints and hover alive", async () => {
  await withFeatureDocument("code-lens-off", { codeLens: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/codeLens", { textDocument: { uri: ctx.uri } });
    await assertInlayHintsWork(ctx);
    await assertHoverWorks(ctx);
  });
});

test("granular documentLinks disable leaves symbols and hover alive", async () => {
  await withFeatureDocument("document-links-off", { documentLinks: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/documentLink", { textDocument: { uri: ctx.uri } });
    await assertDocumentSymbolsWork(ctx);
    await assertHoverWorks(ctx);
  });
});

test("granular definition disable leaves hover and references alive", async () => {
  await withFeatureDocument("definition-off", { definition: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/definition", {
      textDocument: { uri: ctx.uri },
      position: templateMessagePosition(ctx.source),
    });
    await assertHoverWorks(ctx);
    await assertReferencesWork(ctx);
  });
});

test("granular references disable leaves hover and definition alive", async () => {
  await withFeatureDocument("references-off", { references: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/references", {
      textDocument: { uri: ctx.uri },
      position: templateMessagePosition(ctx.source),
      context: { includeDeclaration: true },
    });
    await expectNull(ctx, "textDocument/documentHighlight", {
      textDocument: { uri: ctx.uri },
      position: templateMessagePosition(ctx.source),
    });
    await assertHoverWorks(ctx);
    await assertDefinitionWorks(ctx);
  });
});

test("granular completion disable leaves hover and symbols alive", async () => {
  await withFeatureDocument("completion-off", { completion: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/completion", {
      textDocument: { uri: ctx.uri },
      position: completionStylePosition(ctx.source),
    });
    await assertHoverWorks(ctx);
    await assertDocumentSymbolsWork(ctx);
  });
});

test("granular rename disable leaves hover and definition alive", async () => {
  await withFeatureDocument("rename-off", { rename: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/prepareRename", {
      textDocument: { uri: ctx.uri },
      position: templateMessagePosition(ctx.source),
    });
    await expectNull(ctx, "textDocument/rename", {
      textDocument: { uri: ctx.uri },
      position: templateMessagePosition(ctx.source),
      newName: "renamedMessage",
    });
    await assertHoverWorks(ctx);
    await assertDefinitionWorks(ctx);
  });
});

test("granular formatting disable leaves parser-backed symbols alive", async () => {
  await withFeatureDocument("formatting-off", { formatting: false }, async (ctx) => {
    await expectNull(ctx, "textDocument/formatting", {
      textDocument: { uri: ctx.uri },
      options: { tabSize: 2, insertSpaces: true },
    });
    await expectNull(ctx, "textDocument/rangeFormatting", {
      textDocument: { uri: ctx.uri },
      range: FULL_RANGE,
      options: { tabSize: 2, insertSpaces: true },
    });
    await assertDocumentSymbolsWork(ctx);
  });
});

test("granular codeActions disable does not suppress lint diagnostics", async () => {
  await withFeatureDocument("code-actions-off", { codeActions: false, lint: true }, async (ctx) => {
    await expectNull(ctx, "textDocument/codeAction", {
      textDocument: { uri: ctx.uri },
      range: FULL_RANGE,
      context: { diagnostics: ctx.publish.diagnostics },
    });
    assert.ok(
      ctx.publish.diagnostics.some(
        (diagnostic) =>
          diagnostic.source === "vize/lint" && diagnostic.code === "vue/require-v-for-key",
      ),
      JSON.stringify(ctx.publish.diagnostics),
    );
    await assertHoverWorks(ctx);
  });
});

test("granular fileRename disable leaves document links alive", async () => {
  await withFeatureDocument("file-rename-off", { fileRename: false }, async (ctx) => {
    await expectNull(ctx, "workspace/willRenameFiles", {
      files: [
        {
          oldUri: pathToFileURL(path.join(ctx.workspaceDir, "Dep.vue")).href,
          newUri: pathToFileURL(path.join(ctx.workspaceDir, "RenamedDep.vue")).href,
        },
      ],
    });
    await assertDocumentLinksWork(ctx);
  });
});

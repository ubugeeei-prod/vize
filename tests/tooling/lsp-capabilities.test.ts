import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { LspInitializationOptions, ServerCapabilities } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";

// Capability-advertisement suite for `vize lsp`.
//
// These tests inspect ONLY the `initialize` result: no document features and no
// corsa are exercised, so every assertion is a pure, deterministic function of
// the initialization options. The smoke suite asserts a handful of default
// providers (`hoverProvider`, `definitionProvider`, `referencesProvider`,
// `semanticTokensProvider.range`, `completionProvider.triggerCharacters` has
// "."); here we pin the full provider set and the per-feature gating shapes for
// distinct option bundles, which the smoke suite does not cover.

async function withCapabilities(
  label: string,
  initializationOptions: LspInitializationOptions,
  run: (capabilities: ServerCapabilities) => void,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, `lsp-capabilities-${label}`);
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, initializationOptions)) as {
      capabilities?: ServerCapabilities;
    };
    assert.ok(init.capabilities, "initialize result should advertise capabilities");
    run(init.capabilities);
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

test("vize lsp advertises the full editor-feature provider set with exact shapes", async () => {
  await withCapabilities("editor-full", { editor: true, lint: true }, (capabilities) => {
    // Boolean providers advertised as plain `true`.
    assert.equal(capabilities.documentSymbolProvider, true);
    assert.equal(capabilities.foldingRangeProvider, true);
    assert.equal(capabilities.inlayHintProvider, true);
    assert.equal(capabilities.documentHighlightProvider, true);
    assert.equal(capabilities.workspaceSymbolProvider, true);
    assert.equal(capabilities.hoverProvider, true);
    assert.equal(capabilities.definitionProvider, true);
    assert.equal(capabilities.referencesProvider, true);

    // Resolve-provider shapes.
    assert.equal(capabilities.codeLensProvider?.resolveProvider, false);
    assert.equal(capabilities.documentLinkProvider?.resolveProvider, false);

    // Rename advertises prepareRename support.
    assert.equal(capabilities.renameProvider?.prepareProvider, true);

    // Code actions: kinds plus no resolve step.
    assert.deepEqual(capabilities.codeActionProvider?.codeActionKinds, [
      "quickfix",
      "refactor",
      "source",
    ]);
    assert.equal(capabilities.codeActionProvider?.resolveProvider, false);

    // Document sync: incremental (2), open/close on, save without text.
    assert.equal(capabilities.textDocumentSync?.change, 2);
    assert.equal(capabilities.textDocumentSync?.openClose, true);
    assert.equal(capabilities.textDocumentSync?.save?.includeText, false);

    // Completion trigger characters, exact ordered set.
    assert.deepEqual(capabilities.completionProvider?.triggerCharacters, [
      ".",
      ":",
      "@",
      "#",
      "<",
      "/",
      '"',
      "'",
      " ",
    ]);

    // Selection ranges ship with the document-structure group.
    assert.equal(capabilities.selectionRangeProvider, true);

    // Providers that are intentionally not yet advertised stay absent.
    assert.equal(capabilities.signatureHelpProvider, undefined);
    assert.equal(capabilities.documentRangeFormattingProvider, undefined);
  });
});

test("vize lsp editor:false strips editor providers but keeps lint-driven codeAction", async () => {
  await withCapabilities(
    "editor-off",
    { editor: false, lint: true, typecheck: false },
    (capabilities) => {
      // Editor bundle providers are all gone.
      assert.equal(capabilities.semanticTokensProvider, undefined);
      assert.equal(capabilities.documentSymbolProvider, undefined);
      assert.equal(capabilities.foldingRangeProvider, undefined);
      assert.equal(capabilities.selectionRangeProvider, undefined);
      assert.equal(capabilities.inlayHintProvider, undefined);
      assert.equal(capabilities.completionProvider, undefined);
      assert.equal(capabilities.codeLensProvider, undefined);
      assert.equal(capabilities.documentLinkProvider, undefined);
      assert.equal(capabilities.workspaceSymbolProvider, undefined);
      assert.equal(capabilities.hoverProvider, undefined);
      assert.equal(capabilities.definitionProvider, undefined);
      assert.equal(capabilities.referencesProvider, undefined);
      assert.equal(capabilities.documentHighlightProvider, undefined);

      // Lint code actions survive without the editor bundle.
      assert.ok(capabilities.codeActionProvider, "codeActionProvider should remain present");
      assert.deepEqual(capabilities.codeActionProvider?.codeActionKinds, [
        "quickfix",
        "refactor",
        "source",
      ]);
    },
  );
});

test("vize lsp per-feature init flags toggle individual providers independently", async () => {
  await withCapabilities(
    "granular-four-off",
    {
      editor: true,
      inlayHints: false,
      foldingRanges: false,
      documentSymbols: false,
      semanticTokens: false,
    },
    (capabilities) => {
      assert.equal(capabilities.inlayHintProvider, undefined);
      assert.equal(capabilities.foldingRangeProvider, undefined);
      // Selection ranges share the document-structure flag with folding ranges.
      assert.equal(capabilities.selectionRangeProvider, undefined);
      assert.equal(capabilities.documentSymbolProvider, undefined);
      assert.equal(capabilities.semanticTokensProvider, undefined);
      // A sibling editor provider is untouched.
      assert.equal(capabilities.hoverProvider, true);
    },
  );

  await withCapabilities(
    "granular-authoring-off",
    {
      editor: true,
      completion: false,
      hover: false,
      definition: false,
      references: false,
    },
    (capabilities) => {
      assert.equal(capabilities.completionProvider, undefined);
      assert.equal(capabilities.hoverProvider, undefined);
      assert.equal(capabilities.definitionProvider, undefined);
      assert.equal(capabilities.referencesProvider, undefined);
      assert.equal(capabilities.documentHighlightProvider, undefined);
      // A sibling editor provider is untouched.
      assert.equal(capabilities.documentSymbolProvider, true);
    },
  );

  await withCapabilities(
    "granular-codelens-off",
    { editor: true, codeLens: false },
    (capabilities) => {
      assert.equal(capabilities.codeLensProvider, undefined);
      // The rest of the editor bundle is intact.
      assert.equal(capabilities.inlayHintProvider, true);
      assert.equal(capabilities.foldingRangeProvider, true);
      assert.equal(capabilities.documentSymbolProvider, true);
      assert.ok(capabilities.semanticTokensProvider, "semanticTokensProvider should remain");
      assert.equal(capabilities.hoverProvider, true);
    },
  );

  await withCapabilities("granular-lint-off", { editor: true, lint: false }, (capabilities) => {
    // Code actions are gated on lint, so disabling lint removes only them.
    assert.equal(capabilities.codeActionProvider, undefined);
    assert.equal(capabilities.hoverProvider, true);
    assert.equal(capabilities.codeLensProvider?.resolveProvider, false);
    assert.ok(capabilities.semanticTokensProvider, "semanticTokensProvider should remain");
  });
});

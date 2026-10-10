import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type {
  JsonRpcMessage,
  LspInitializationOptions,
  ServerCapabilities,
} from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import { EDITOR_BUNDLE_CAPABILITIES } from "./support/lsp/capability-oracles.ts";
import {
  assertInitializePacket,
  FRESH_EDITOR_CAPABILITIES,
  observeInitializePacket,
} from "./support/lsp/fresh-capability-policy.ts";

// Capability-advertisement suite for `vize lsp`.
//
// These tests inspect ONLY the `initialize` result: no document features and no
// corsa are exercised. Fresh project policy, dedicated configuration and the
// initialization options determine the complete result. The smoke suite asserts
// a handful of default providers (`hoverProvider`, `definitionProvider`, `referencesProvider`,
// `semanticTokensProvider.range`, `completionProvider.triggerCharacters` has
// "."); here we pin the full provider set and the per-feature gating shapes for
// distinct option bundles, which the smoke suite does not cover.

async function withCapabilities(
  label: string,
  initializationOptions: LspInitializationOptions,
  run: (capabilities: ServerCapabilities, packet: JsonRpcMessage) => void,
  prepare?: (workspaceDir: string) => void,
): Promise<void> {
  const testRootDir = path.join(testOutputRoot, `lsp-capabilities-${label}`);
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  const initializePacket = observeInitializePacket(session, label);

  try {
    prepare?.(workspaceDir);
    const init = (await session.initialize(workspaceDir, initializationOptions)) as {
      capabilities?: ServerCapabilities;
    };
    assert.ok(init.capabilities, "initialize result should advertise capabilities");
    run(init.capabilities, initializePacket());
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
}

test("vize lsp advertises exactly this capability set for the default editor bundle", async () => {
  await withCapabilities("editor-full", { editor: true, lint: true }, (_capabilities, packet) => {
    assertInitializePacket(packet, FRESH_EDITOR_CAPABILITIES);
  });
});

test("vize lsp advertises the exact measured auto-insertion extension only when opted in", async () => {
  await withCapabilities(
    "auto-insert",
    { editor: true, autoInsert: true },
    (_capabilities, packet) => {
      assertInitializePacket(packet, {
        ...FRESH_EDITOR_CAPABILITIES,
        experimental: {
          vize: { jsxTypecheck: true },
          autoInsertionProvider: {
            triggerCharacters: ["}", "=", ">", "/", "\\w"],
            configurationSections: [
              ["vize.autoInsert.bracketSpacing"],
              ["vize.autoInsert.autoCreateQuotes"],
              ["vize.autoInsert.autoClosingTags"],
              ["vize.autoInsert.dotValue"],
            ],
          },
        },
      });
    },
  );

  await withCapabilities(
    "auto-insert-off",
    { editor: true, autoInsert: false },
    (_capabilities, packet) => {
      assertInitializePacket(packet, FRESH_EDITOR_CAPABILITIES);
    },
  );
});

test("vize lsp preserves the complete historical default for dedicated empty configuration", async () => {
  await withCapabilities(
    "dedicated-empty",
    { editor: true, lint: true },
    (_capabilities, packet) => assertInitializePacket(packet, EDITOR_BUNDLE_CAPABILITIES),
    (workspaceDir) => fs.writeFileSync(path.join(workspaceDir, "vize.config.json"), "{}\n"),
  );
});

test("vize lsp honors complete explicit-false Vite and initialization controls", async () => {
  await withCapabilities(
    "vite-explicit-false",
    { editor: true, lint: true },
    (_capabilities, packet) => assertInitializePacket(packet, EDITOR_BUNDLE_CAPABILITIES),
    (workspaceDir) =>
      fs.writeFileSync(
        path.join(workspaceDir, "vite.config.mjs"),
        "export default { vize: { languageServer: { formatting: false }, typeChecker: { jsxTypecheck: false } } };\n",
      ),
  );
  await withCapabilities(
    "init-formatting-false",
    { editor: true, lint: true, formatting: false },
    (_capabilities, packet) =>
      assertInitializePacket(packet, {
        ...EDITOR_BUNDLE_CAPABILITIES,
        experimental: { vize: { jsxTypecheck: true } },
      }),
  );
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
      assert.equal(capabilities.colorProvider, undefined);
      assert.equal(capabilities.workspaceSymbolProvider, undefined);
      assert.equal(capabilities.hoverProvider, undefined);
      assert.equal(capabilities.declarationProvider, undefined);
      assert.equal(capabilities.definitionProvider, undefined);
      assert.equal(capabilities.typeDefinitionProvider, undefined);
      assert.equal(capabilities.implementationProvider, undefined);
      assert.equal(capabilities.referencesProvider, undefined);
      assert.equal(capabilities.documentHighlightProvider, undefined);

      // Lint code actions survive without the editor bundle.
      assert.ok(capabilities.codeActionProvider, "codeActionProvider should remain present");
      assert.deepEqual(capabilities.codeActionProvider?.codeActionKinds, ["quickfix"]);
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
      signatureHelp: false,
      hover: false,
      definition: false,
      references: false,
    },
    (capabilities) => {
      assert.equal(capabilities.completionProvider, undefined);
      assert.equal(capabilities.signatureHelpProvider, undefined);
      assert.equal(capabilities.hoverProvider, undefined);
      assert.equal(capabilities.definitionProvider, undefined);
      assert.equal(capabilities.typeDefinitionProvider, undefined);
      assert.equal(capabilities.referencesProvider, undefined);
      assert.equal(capabilities.documentHighlightProvider, undefined);
      // A sibling editor provider is untouched.
      assert.equal(capabilities.documentSymbolProvider, true);
    },
  );

  await withCapabilities(
    "granular-typecheck-off",
    { editor: true, typecheck: false },
    (capabilities) => {
      assert.equal(capabilities.implementationProvider, undefined);
      assert.equal(capabilities.declarationProvider, undefined);
      assert.equal(capabilities.typeDefinitionProvider, undefined);
      // Definition keeps its lexical/editor fallback when typecheck is disabled.
      assert.equal(capabilities.definitionProvider, true);
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

  await withCapabilities(
    "granular-lint-off",
    { editor: true, lint: false, typecheck: false },
    (capabilities) => {
      // Without either diagnostic provider there are no quick fixes.
      assert.equal(capabilities.codeActionProvider, undefined);
      assert.equal(capabilities.hoverProvider, true);
      assert.equal(capabilities.codeLensProvider?.resolveProvider, false);
      assert.ok(capabilities.semanticTokensProvider, "semanticTokensProvider should remain");
    },
  );

  await withCapabilities(
    "granular-formatting-on",
    { editor: true, formatting: true },
    (capabilities) => {
      // Opting into formatting brings all three commands: one formatter scoped
      // to the document, to the SFC blocks a selection touches, and to the line
      // under the caret (#3456).
      assert.equal(capabilities.documentFormattingProvider, true);
      assert.equal(capabilities.documentRangeFormattingProvider, true);
      // The on-type trigger set is `@vue/language-server`'s, character for
      // character, so an editor configured for one behaves the same under the
      // other.
      assert.deepEqual(capabilities.documentOnTypeFormattingProvider, {
        firstTriggerCharacter: ";",
        moreTriggerCharacter: ["}", "\n"],
      });
    },
  );
});

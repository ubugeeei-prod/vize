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

/** Declaration shims, the only files type checking has to watch itself. */
const DECLARATION_FILTERS = [
  { scheme: "file", pattern: { glob: "**/*.d.{ts,mts,cts}", matches: "file" } },
];
/** Everything a rename can move, plus folders. */
const RENAME_FILTERS = [
  {
    scheme: "file",
    pattern: { glob: "**/*.{vue,ts,tsx,d.ts,d.mts,d.cts,js,jsx,mts,cts,mjs,cjs}", matches: "file" },
  },
  { scheme: "file", pattern: { glob: "**/*", matches: "folder" } },
];

/**
 * The COMPLETE capability set for the default editor bundle, pinned as one
 * value rather than field by field: adding, dropping or reshaping any provider
 * then shows up in this diff instead of slipping past a spot check (#3456).
 */
const EDITOR_BUNDLE_CAPABILITIES = {
  // Incremental (2) sync, open/close on, save without the text.
  textDocumentSync: {
    openClose: true,
    change: 2,
    willSave: false,
    willSaveWaitUntil: false,
    save: { includeText: false },
  },
  // Selection ranges ship with the document-structure group.
  selectionRangeProvider: true,
  hoverProvider: true,
  completionProvider: {
    resolveProvider: true,
    // `@vue/language-server` 3.3.8's list in its order, plus `'` (a
    // single-quoted attribute value is legal Vue and Maestro answers inside
    // one). Space is deliberately absent — it opened the list on every space
    // typed in a template (#3458).
    triggerCharacters: [
      '"',
      "'",
      ":",
      "@",
      ".",
      "<",
      "=",
      "/",
      ">",
      "+",
      "^",
      "*",
      "(",
      ")",
      "#",
      "[",
      "]",
      "$",
      "-",
      "{",
      "}",
    ],
  },
  definitionProvider: true,
  referencesProvider: true,
  documentHighlightProvider: true,
  documentSymbolProvider: true,
  workspaceSymbolProvider: true,
  codeActionProvider: {
    codeActionKinds: ["quickfix", "refactor", "source"],
    resolveProvider: false,
  },
  codeLensProvider: { resolveProvider: false },
  documentLinkProvider: { resolveProvider: false },
  // Colour swatches ship with document links: both decorate a literal in the
  // authored text and make it interactive (#3456).
  colorProvider: true,
  foldingRangeProvider: true,
  // Rename advertises prepareRename, and carries linked editing
  // (rename-as-you-type over tag names) with it.
  renameProvider: { prepareProvider: true },
  linkedEditingRangeProvider: true,
  semanticTokensProvider: {
    legend: {
      tokenTypes: [
        "namespace",
        "type",
        "class",
        "enum",
        "interface",
        "struct",
        "typeParameter",
        "parameter",
        "variable",
        "property",
        "enumMember",
        "event",
        "function",
        "method",
        "macro",
        "keyword",
        "modifier",
        "comment",
        "string",
        "number",
        "regexp",
        "operator",
        "decorator",
      ],
      tokenModifiers: [
        "declaration",
        "definition",
        "readonly",
        "static",
        "deprecated",
        "abstract",
        "async",
        "modification",
        "documentation",
        "defaultLibrary",
      ],
    },
    range: true,
    full: true,
  },
  inlayHintProvider: true,
  workspace: {
    workspaceFolders: { supported: true, changeNotifications: true },
    fileOperations: {
      didCreate: { filters: DECLARATION_FILTERS },
      didRename: { filters: RENAME_FILTERS },
      willRename: { filters: RENAME_FILTERS },
      didDelete: { filters: DECLARATION_FILTERS },
    },
  },
  // Absent on purpose, and therefore absent from this object: the three
  // formatting providers (opt-in, see below), `signatureHelpProvider`,
  // `typeDefinitionProvider`, `implementationProvider`, `declarationProvider`,
  // `executeCommandProvider`, `callHierarchyProvider`, `monikerProvider` and
  // `experimental` — none of which has a handler behind it.
};

test("vize lsp advertises exactly this capability set for the default editor bundle", async () => {
  await withCapabilities("editor-full", { editor: true, lint: true }, (capabilities) => {
    assert.deepEqual(capabilities, EDITOR_BUNDLE_CAPABILITIES);
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
      assert.equal(capabilities.colorProvider, undefined);
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

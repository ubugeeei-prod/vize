/** Closed Vue files feed workspace symbols; declarations feed type checking. */
const FILE_EVENT_FILTERS = [
  { scheme: "file", pattern: { glob: "**/*.d.{ts,mts,cts}", matches: "file" } },
  { scheme: "file", pattern: { glob: "**/*.vue", matches: "file" } },
  { scheme: "file", pattern: { glob: "**/*", matches: "folder" } },
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
export const EDITOR_BUNDLE_CAPABILITIES = {
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
  // TypeScript/tsgo signature help: opens on a call or generic argument list
  // and re-opens once the caller closes it.
  signatureHelpProvider: {
    triggerCharacters: ["(", ",", "<"],
    retriggerCharacters: [")"],
  },
  definitionProvider: true,
  declarationProvider: true,
  typeDefinitionProvider: true,
  implementationProvider: true,
  callHierarchyProvider: true,
  referencesProvider: true,
  documentHighlightProvider: true,
  documentSymbolProvider: true,
  workspaceSymbolProvider: true,
  codeActionProvider: {
    codeActionKinds: ["quickfix"],
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
  experimental: { vize: { jsxTypecheck: false } },
  workspace: {
    workspaceFolders: { supported: true, changeNotifications: true },
    fileOperations: {
      didCreate: { filters: FILE_EVENT_FILTERS },
      didRename: { filters: RENAME_FILTERS },
      willRename: { filters: RENAME_FILTERS },
      didDelete: { filters: FILE_EVENT_FILTERS },
    },
  },
  // Opt-in formatting providers, `executeCommandProvider` and
  // `monikerProvider` are absent from this default capability set.
};

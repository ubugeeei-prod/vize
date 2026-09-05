export type JsonRpcId = number;

export type JsonRpcMessage = {
  jsonrpc: "2.0";
  id?: JsonRpcId;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
};

export type LspInitializationOptions = {
  codeActions?: boolean;
  codeLens?: boolean;
  completion?: boolean;
  definition?: boolean;
  documentLinks?: boolean;
  documentSymbols?: boolean;
  ecosystem?: boolean;
  editor?: boolean;
  fileRename?: boolean;
  foldingRanges?: boolean;
  formatting?: boolean;
  hover?: boolean;
  inlayHints?: boolean;
  legacyVue2?: boolean;
  lint?: boolean;
  optionsApi?: boolean;
  references?: boolean;
  rename?: boolean;
  semanticTokens?: boolean;
  signatureHelp?: boolean;
  typecheck?: boolean;
  workspaceSymbols?: boolean;
  autoInsert?: boolean;
};

export type LspPosition = {
  line: number;
  character: number;
};

export type LspRange = {
  start: LspPosition;
  end: LspPosition;
};

export type TextDocumentIdentifier = {
  uri: string;
};

export type TextDocumentPositionParams = {
  textDocument: TextDocumentIdentifier;
  position: LspPosition;
};

export type TextDocumentRangeParams = {
  textDocument: TextDocumentIdentifier;
  range: LspRange;
};

export type WorkspaceEdit = {
  changes?: Record<string, Array<{ range: LspRange; newText: string }>>;
  documentChanges?: unknown[];
};

export type ServerCapabilities = {
  codeActionProvider?: {
    codeActionKinds?: string[];
    resolveProvider?: boolean;
  };
  codeLensProvider?: { resolveProvider?: boolean };
  colorProvider?: unknown;
  completionProvider?: {
    triggerCharacters?: string[];
    resolveProvider?: boolean;
  };
  callHierarchyProvider?: unknown;
  declarationProvider?: unknown;
  definitionProvider?: unknown;
  typeDefinitionProvider?: unknown;
  documentHighlightProvider?: unknown;
  documentFormattingProvider?: unknown;
  documentLinkProvider?: { resolveProvider?: boolean };
  documentOnTypeFormattingProvider?: {
    firstTriggerCharacter?: string;
    moreTriggerCharacter?: string[];
  };
  documentRangeFormattingProvider?: unknown;
  documentSymbolProvider?: unknown;
  foldingRangeProvider?: unknown;
  hoverProvider?: unknown;
  implementationProvider?: unknown;
  inlayHintProvider?: unknown;
  linkedEditingRangeProvider?: unknown;
  referencesProvider?: unknown;
  renameProvider?: { prepareProvider?: boolean };
  selectionRangeProvider?: unknown;
  semanticTokensProvider?: unknown;
  signatureHelpProvider?: unknown;
  textDocumentSync?: {
    change?: number;
    openClose?: boolean;
    save?: { includeText?: boolean };
  };
  workspace?: unknown;
  workspaceSymbolProvider?: unknown;
  experimental?: {
    autoInsertionProvider?: {
      triggerCharacters: string[];
      configurationSections: Array<string[] | null>;
    };
  };
};

export type AutoInsertParams = {
  textDocument: TextDocumentIdentifier;
  selection: LspPosition;
  change: {
    rangeOffset: number;
    rangeLength: number;
    text: string;
  };
};

export type LspDiagnostic = {
  code?: unknown;
  source?: string;
  severity?: number;
  message?: string;
  range?: {
    start?: { line?: number; character?: number };
    end?: { line?: number; character?: number };
  };
};

export type PublishDiagnosticsParams = {
  uri: string;
  version?: number;
  diagnostics: LspDiagnostic[];
};

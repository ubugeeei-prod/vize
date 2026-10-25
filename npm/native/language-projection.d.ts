export interface LanguageProjection {
  text: string;
  extension: ".tsx";
  /** Both coordinate spaces are UTF-16 after conversion. */
  mappings: Array<[number, number, number, number, 0 | 1 | 2, number]>;
  diagnostics: ProjectionDiagnostic[];
  semanticLinks: Array<{
    sourceStart: number;
    sourceLength: number;
    targetStart: number;
    targetLength: number;
    kind: string;
  }>;
  diagnosticDirectives?: {
    directives: Array<[number, number, number, number, 0 | 1, number]>;
    unusedExpectDirectiveDiagnostics: Array<{ code: number; messageText: string }>;
  };
}
export interface ProjectionDiagnostic {
  start: number;
  length: number;
  code: number;
  messageText: string;
}
export declare const LanguageFeature: Readonly<
  Record<
    | "Hover"
    | "SignatureHelp"
    | "Completion"
    | "Definition"
    | "TypeDefinition"
    | "Implementation"
    | "References"
    | "DocumentHighlights"
    | "Rename"
    | "CallHierarchy"
    | "CodeActions"
    | "Formatting"
    | "InlayHints"
    | "SemanticTokens"
    | "FoldingRanges"
    | "SelectionRanges"
    | "LinkedEditing"
    | "AutoInsert"
    | "DocumentSymbols"
    | "CodeLens",
    number
  >
>;
export declare function toUtf16LanguageProjection(
  source: string,
  rawTransformJSON: string,
): LanguageProjection;
export declare function applyDiagnosticDirectives<T extends { start?: number }>(
  projection: LanguageProjection,
  generatedDiagnostics: readonly T[],
): { diagnostics: T[]; sourceDiagnostics: ProjectionDiagnostic[] };
/** Diagnostic-only gap semantics from the native Content Mapper protocol. */
export declare function mapLanguageDiagnosticRange(
  projection: LanguageProjection,
  start: number,
  length: number,
): {
  start: number;
  length: number;
  fidelity: "exact" | "atom" | "approximate" | "none";
};

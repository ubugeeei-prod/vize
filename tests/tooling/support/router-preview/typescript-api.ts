// Structural boundary for the pinned real compiler loaded from the consumer.
// This describes the SDK calls only; it declares no provider modules or helpers.
export type CompilerSourceFile = { fileName: string; isDeclarationFile: boolean };
export type CompilerDeclaration = {
  getSourceFile(): CompilerSourceFile;
  getStart(): number;
  end: number;
};
export type CompilerSymbol = {
  name: string;
  flags: number;
  declarations?: readonly CompilerDeclaration[];
};
export type CompilerChecker = {
  getSymbolAtLocation(file: CompilerSourceFile): CompilerSymbol | undefined;
  getExportsOfModule(symbol: CompilerSymbol): CompilerSymbol[];
  getAliasedSymbol(symbol: CompilerSymbol): CompilerSymbol;
};
export type PinnedTypeScriptApi = {
  version: string;
  ModuleKind: { ESNext: number };
  ModuleResolutionKind: { Bundler: number };
  ScriptTarget: { ESNext: number };
  SymbolFlags: { Alias: number; TypeAlias: number };
  createProgram(
    files: string[],
    options: Record<string, unknown>,
  ): {
    getTypeChecker(): CompilerChecker;
    getSourceFile(file: string): CompilerSourceFile | undefined;
  };
};

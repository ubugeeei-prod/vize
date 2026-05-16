// Compiler-related type definitions

export interface CompilerOptions {
  mode?: "function" | "module";
  prefixIdentifiers?: boolean;
  hoistStatic?: boolean;
  cacheHandlers?: boolean;
  ssr?: boolean;
  scopeId?: string | null;
  filename?: string;
  // SFC output target selection
  outputMode?: "vdom" | "vapor";
  isTs?: boolean;
  // Script extension: 'preserve' keeps TypeScript, 'downcompile' (default) transpiles to JS
  scriptExt?: "preserve" | "downcompile";
  bindingMetadata?: SfcBindingMetadata;
}

export interface CompileResult {
  code: string;
  preamble: string;
  ast: object;
  map?: object | null;
  helpers: string[];
  templates?: string[];
}

export interface SfcBlock {
  content: string;
  loc: { start: number; end: number };
  lang?: string;
  src?: string;
  attrs: Record<string, string>;
}

export interface SfcScriptBlock extends SfcBlock {
  setup: boolean;
}

export interface SfcStyleBlock extends SfcBlock {
  scoped: boolean;
  module?: string;
}

export interface SfcDescriptor {
  filename: string;
  source: string;
  template?: SfcBlock;
  script?: SfcScriptBlock;
  scriptSetup?: SfcScriptBlock;
  styles: SfcStyleBlock[];
  customBlocks: Array<{ type: string; content: string; attrs: Record<string, string> }>;
}

export interface SfcCompileResult {
  descriptor: SfcDescriptor;
  template?: CompileResult;
  script?: {
    code: string;
    bindings?: SfcBindingMetadata;
  };
  css?: string;
  errors?: string[];
  warnings?: string[];
  bindingMetadata?: Record<string, string | number>;
}

export interface SfcBindingMetadata {
  bindings: Record<string, string | number>;
  propsAliases?: Record<string, string>;
  isScriptSetup?: boolean;
}

export interface CssCompileOptions {
  scopeId?: string;
  scoped?: boolean;
  minify?: boolean;
  sourceMap?: boolean;
  filename?: string;
  targets?: {
    chrome?: number;
    firefox?: number;
    safari?: number;
    edge?: number;
    ios?: number;
    android?: number;
  };
}

export interface CssCompileResult {
  code: string;
  map?: string;
  cssVars: string[];
  errors: string[];
  warnings: string[];
}

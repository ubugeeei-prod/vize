/* eslint-disable */

/** Batch compile options for NAPI */
export interface BatchCompileOptionsNapi {
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  vueParserQuirks?: boolean;
  /** Preserve TypeScript in output when true */
  isTs?: boolean;
  threads?: number;
}

/** Batch compile result for NAPI */
export interface BatchCompileResultNapi {
  /** Number of files compiled successfully */
  success: number;
  /** Number of files that failed */
  failed: number;
  /** Total input bytes */
  inputBytes: number;
  /** Total output bytes */
  outputBytes: number;
  /** Compilation time in milliseconds */
  timeMs: number;
}

/** Batch compile result with per-file results */
export interface BatchCompileResultWithFilesNapi {
  /** Per-file compilation results */
  results: Array<BatchFileResultNapi>;
  /** Number of files compiled successfully */
  successCount: number;
  /** Number of files that failed */
  failedCount: number;
  /** Compilation time in milliseconds */
  timeMs: number;
}

/** Input file for batch compilation with results */
export interface BatchFileInputNapi {
  /** File path */
  path: string;
  /** Source code */
  source: string;
}

/** Per-file result from batch compilation */
export interface BatchFileResultNapi {
  /** File path */
  path: string;
  /** Generated JavaScript code */
  code: string;
  /** Generated CSS (if any) */
  css?: string;
  /** Scope ID for scoped styles */
  scopeId: string;
  /** Whether the file has scoped styles */
  hasScoped: boolean;
  /** Compilation errors */
  errors: Array<string>;
  /** Compilation warnings */
  warnings: Array<string>;
  /** Hash of template content (for HMR) */
  templateHash?: string;
  /** Hash of style content (for HMR) */
  styleHash?: string;
  /** Hash of script content (for HMR) */
  scriptHash?: string;
  /** Per-block style metadata */
  styles: Array<StyleBlockNapi>;
  /** Custom block metadata */
  customBlocks: Array<CustomBlockNapi>;
  /** Compile-time macro artifacts */
  macroArtifacts: Array<MacroArtifactNapi>;
}

export interface CustomBlockNapi {
  blockType: string;
  content: string;
  src?: string;
  attrs: Array<SfcBlockAttributeNapi>;
  index: number;
}

export interface MacroArtifactNapi {
  kind: string;
  name: string;
  source: string;
  content: string;
  moduleCode?: string;
  start: number;
  end: number;
}

export interface SfcBlockAttributeNapi {
  name: string;
  value?: string;
}

/** SFC compile options for NAPI */
export interface SfcCompileOptionsNapi {
  filename?: string;
  sourceMap?: boolean;
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  vueParserQuirks?: boolean;
  /** Preserve TypeScript in output when true */
  isTs?: boolean;
  /** Scope ID for scoped CSS (e.g., "data-v-abc123") */
  scopeId?: string;
}

/** SFC compile result for NAPI */
export interface SfcCompileResultNapi {
  /** Generated JavaScript code */
  code: string;
  /** Generated CSS (if any) */
  css?: string;
  /** Compilation errors */
  errors: Array<string>;
  /** Compilation warnings */
  warnings: Array<string>;
  /** Hash of template content (for HMR) */
  templateHash?: string;
  /** Hash of style content (for HMR) */
  styleHash?: string;
  /** Hash of script content (for HMR) */
  scriptHash?: string;
  /** Whether the file has scoped styles */
  hasScoped: boolean;
  /** Per-block style metadata */
  styles: Array<StyleBlockNapi>;
  /** Custom block metadata */
  customBlocks: Array<CustomBlockNapi>;
  /** Compile-time macro artifacts */
  macroArtifacts: Array<MacroArtifactNapi>;
}

/** SFC parse options for NAPI */
export interface SfcParseOptionsNapi {
  filename?: string;
}

export interface SfcSrcInfoNapi {
  scriptSrc?: string;
  templateSrc?: string;
}

export interface StyleBlockNapi {
  content: string;
  src?: string;
  lang?: string;
  scoped: boolean;
  module: boolean;
  moduleName?: string;
  index: number;
}

export interface TemplateAssetTagRuleNapi {
  tag: string;
  attrs: Array<string>;
}

export interface TemplateAssetUrlNapi {
  url: string;
  varName: string;
}

/** Compile SFC (.vue file) to JavaScript - main use case */
export declare function compileSfc(
  source: string,
  options?: SfcCompileOptionsNapi | undefined | null,
): SfcCompileResultNapi;

/** Batch compile SFC files matching a glob pattern (native multithreading) */
export declare function compileSfcBatch(
  pattern: string,
  options?: BatchCompileOptionsNapi | undefined | null,
): BatchCompileResultNapi;

/** Batch compile SFC files with per-file results (in-memory, native multithreading) */
export declare function compileSfcBatchWithResults(
  files: Array<BatchFileInputNapi>,
  options?: BatchCompileOptionsNapi | undefined | null,
): BatchCompileResultWithFilesNapi;

export declare function collectSfcTemplateAssetUrls(
  source: string,
  rules?: Array<TemplateAssetTagRuleNapi> | undefined | null,
  filename?: string | undefined | null,
): Array<TemplateAssetUrlNapi>;

export declare function extractSfcCustomBlocks(
  source: string,
  filename?: string | undefined | null,
): Array<CustomBlockNapi>;

export declare function extractSfcSrcInfo(
  source: string,
  filename?: string | undefined | null,
): SfcSrcInfoNapi;

export declare function extractSfcStyleBlocks(
  source: string,
  filename?: string | undefined | null,
): Array<StyleBlockNapi>;

export declare function generateSfcScopeId(
  filename: string,
  root?: string | undefined | null,
  isProduction?: boolean | undefined | null,
  source?: string | undefined | null,
): string;

export declare function hasSfcScopedStyle(
  source: string,
  filename?: string | undefined | null,
): boolean;

export declare function isSfcImportableAssetUrl(url: string): boolean;

/** Parse SFC (.vue file) - returns lightweight result for speed */
export declare function parseSfc(
  source: string,
  options?: SfcParseOptionsNapi | undefined | null,
): any;

export declare function stripSfcScopedCssComments(css: string): string;

export declare function wrapSfcScopedPreprocessorStyle(
  content: string,
  scoped?: string | undefined | null,
  lang?: string | undefined | null,
): string;

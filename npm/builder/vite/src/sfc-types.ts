import type { ExperimentalCompileFlags } from "./experimental-options.ts";
import type { ModuleOutputInfo } from "./utils/module-output.js";

export interface SfcCompileOptionsNapi extends ExperimentalCompileFlags {
  filename?: string;
  mode?: "module" | "function";
  sourceMap?: boolean;
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  templateSyntax?: "standard" | "strict" | "quirks";
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string;
  scopeId?: string;
  styleTrim?: boolean;
  templateCacheHandlers?: boolean;
  templateComments?: boolean;
  templateHoistStatic?: boolean;
  templatePrefixIdentifiers?: boolean;
}

export interface MacroArtifact {
  kind: string;
  name: string;
  source: string;
  content: string;
  moduleCode?: string;
  start: number;
  end: number;
}

export interface NativeStyleBlockInfo {
  /** Raw style content (uncompiled for preprocessor langs) */
  content: string;
  /** External source path from `<style src>`, when present */
  src?: string | null;
  /** Language of the style block (e.g., "css", "scss", "less", "sass", "stylus") */
  lang?: string | null;
  /** Whether the style block has the scoped attribute */
  scoped: boolean;
  /** Whether the style block has the module attribute */
  module: boolean;
  /** CSS Modules binding name for named module attributes */
  moduleName?: string | null;
  /** Index of this style block in the SFC */
  index: number;
}

export interface SfcCompileResultNapi {
  code: string;
  /**
   * Source Map v3 document (JSON) describing `code`, present only when
   * `sourceMap` was requested and a line could be anchored (#3399). Its single
   * `sources` entry is the authored `.vue` path.
   */
  map?: string;
  css?: string;
  errors: string[];
  warnings: string[];
  templateHash?: string;
  styleHash?: string;
  scriptHash?: string;
  hasScoped: boolean;
  styles: NativeStyleBlockInfo[];
  macroArtifacts?: MacroArtifact[];
  moduleShape?: ModuleOutputInfo;
}

export type CompileSfcFn = (
  source: string,
  options?: SfcCompileOptionsNapi,
) => SfcCompileResultNapi;

export interface StyleBlockInfo {
  /** Raw style content (uncompiled for preprocessor langs) */
  content: string;
  /** External source path from `<style src>`, when present */
  src?: string | null;
  /** Language of the style block (e.g., "css", "scss", "less", "sass", "stylus") */
  lang: string | null;
  /** Whether the style block has the scoped attribute */
  scoped: boolean;
  /** CSS Modules: true for unnamed `module`, or the binding name for `module="name"` */
  module: boolean | string;
  /** Index of this style block in the SFC */
  index: number;
}

export interface CompiledModule {
  code: string;
  /**
   * Source Map v3 document (JSON) describing `code`, when the compiler was
   * asked for one (#3399). Absent when source maps are off, when the SFC has no
   * script block, and for the rspack and unplugin builders, which do not request
   * maps. Persisted with the rest of the module in the pre-compile cache.
   */
  map?: string;
  css?: string;
  scopeId: string;
  hasScoped: boolean;
  templateHash?: string;
  styleHash?: string;
  scriptHash?: string;
  /** Compile-time macro artifacts extracted from the source SFC */
  macroArtifacts?: MacroArtifact[];
  /** Per-block style metadata extracted from the source SFC */
  styles?: StyleBlockInfo[];
  /** Files loaded through SFC `src` imports */
  dependencies?: string[];
  /**
   * Module shape reported by the native compiler, so `generateOutput` need not
   * re-parse the emitted module (#3425). Absent for a cache entry written before
   * the field existed, and for the rspack and unplugin builders, which never set
   * it — both fall back to parsing.
   */
  moduleShape?: ModuleOutputInfo;
}

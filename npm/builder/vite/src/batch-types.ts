/**
 * Batch-compilation types, split out of `types.ts` so that module stays inside
 * the per-file source-length budget.
 */

import type { ExperimentalCompileFlags } from "./experimental-options.ts";
import type { ModuleOutputInfo } from "./utils/module-output.ts";
import type { MacroArtifact, NativeStyleBlockInfo } from "./types.ts";

export interface BatchFileInput {
  path: string;
  source: string;
}

export interface BatchFileResult {
  path: string;
  code: string;
  css?: string;
  scopeId: string;
  hasScoped: boolean;
  errors: string[];
  warnings: string[];
  templateHash?: string;
  styleHash?: string;
  scriptHash?: string;
  /** Compile-time macro artifacts extracted from the source SFC */
  macroArtifacts?: MacroArtifact[];
  /** Per-block style metadata extracted from the source SFC */
  styles?: NativeStyleBlockInfo[];
  /**
   * Shape of the emitted module, reported by the native compiler so consumers
   * need not re-parse it (#3425). Absent when the module did not parse.
   */
  moduleShape?: ModuleOutputInfo;
}

export interface BatchCompileOptionsNapi extends ExperimentalCompileFlags {
  mode?: "module" | "function";
  ssr?: boolean;
  vapor?: boolean;
  customRenderer?: boolean;
  templateSyntax?: "standard" | "strict" | "quirks";
  runtimeModuleName?: string;
  runtimeGlobalName?: string;
  vueVersion?: string;
  threads?: number;
  /**
   * Include per-block style metadata (incl. `styles[].content`). Default OFF.
   * `code`/`css` are always returned; this opts into the extra CSS-modules /
   * preprocessor metadata the bundler pipeline needs.
   */
  includeStyles?: boolean;
  /** Include parsed custom blocks. Default OFF. */
  includeCustomBlocks?: boolean;
  /** Include compile-time macro artifacts. Default OFF. */
  includeMacroArtifacts?: boolean;
  /** Include template/style/script content hashes (for HMR). Default OFF. */
  includeHashes?: boolean;
}

export interface BatchCompileResultWithFiles {
  results: BatchFileResult[];
  successCount: number;
  failedCount: number;
  timeMs: number;
}

export type CompileSfcBatchWithResultsFn = (
  files: BatchFileInput[],
  options?: BatchCompileOptionsNapi,
) => BatchCompileResultWithFiles;

import type { SfcCompileOptionsNapi } from "@vizejs/native";

export type LoaderItem = string | { loader: string; options?: Record<string, unknown> };

export interface VizeTurbopackOptions {
  /** Project root used for portable, deterministic component identities. */
  root?: string;
  sourceMap?: boolean;
  vapor?: boolean;
  hotReload?: boolean;
  compilerOptions?: Omit<SfcCompileOptionsNapi, "filename" | "scopeId" | "ssr" | "mode">;
  /** Host preprocessor loaders, in normal webpack right-to-left order. */
  styleLoaders?: Record<string, LoaderItem[]>;
}

export interface SfcLoaderOptions extends Omit<VizeTurbopackOptions, "styleLoaders"> {
  ssr: boolean;
  isProduction: boolean;
  styleLanguages: string[];
}

export interface LoaderContext<T> {
  resourcePath: string;
  resourceQuery: string;
  rootContext: string;
  sourceMap?: boolean;
  getOptions(): T;
  addDependency(filename: string): void;
  emitWarning(warning: Error): void;
  async(): (error: Error | null, code?: string, map?: object | string | null) => void;
}

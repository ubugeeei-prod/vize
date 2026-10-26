import type { SfcCompileOptionsNapi } from "@vizejs/native";

export type JsonValue = string | number | boolean | JsonValue[] | { [key: string]: JsonValue };
export type LoaderItem = string | { loader: string; options?: Record<string, JsonValue> };

export interface VizeTurbopackOptions {
  /** Project root used for portable, deterministic component identities. */
  root?: string;
  sourceMap?: boolean;
  hotReload?: boolean;
  compilerOptions?: Omit<
    SfcCompileOptionsNapi,
    | "filename"
    | "scopeId"
    | "ssr"
    | "mode"
    | "sourceMap"
    | "vapor"
    | "isProd"
    | "isTs"
    | "inlineTemplate"
  >;
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

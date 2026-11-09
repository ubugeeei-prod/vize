import type { SfcCompileOptionsNapi } from "@vizejs/native";

/** JSON-compatible values admitted by serialized Turbopack loader options. */
export type JsonValue = string | number | boolean | JsonValue[] | { [key: string]: JsonValue };
/** A loader request or a loader request paired with JSON-compatible options. */
export type LoaderItem = string | { loader: string; options?: Record<string, JsonValue> };

/** Public options for the Vize Turbopack integration. */
export interface VizeTurbopackOptions {
  /** Project root used for portable, deterministic component identities. @default process.cwd() */
  root?: string;
  /** Emit source maps for compiled SFC modules. @default true */
  sourceMap?: boolean;
  /** Install component hot-reload boundaries in development builds. @default true */
  hotReload?: boolean;
  /** Native SFC compiler overrides forwarded by the host integration. @default undefined */
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
  /** Host preprocessor loaders, in normal webpack right-to-left order. @default undefined */
  styleLoaders?: Record<string, LoaderItem[]>;
}

/** Fully resolved options passed to the SFC compilation loader. */
export interface SfcLoaderOptions extends Omit<VizeTurbopackOptions, "styleLoaders"> {
  ssr: boolean;
  isProduction: boolean;
  styleLanguages: string[];
}

/** Minimal webpack-compatible context implemented by Turbopack loaders. */
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

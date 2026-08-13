export type VitePluginVueFilterPattern = string | RegExp | (string | RegExp)[];
export type VitePluginVueCustomElementOption = boolean | VitePluginVueFilterPattern;

export type VitePluginVueComponentIdGenerator =
  | "filepath"
  | "filepath-source"
  | ((
      filepath: string,
      source: string,
      isProduction: boolean,
      getHash: (text: string) => string,
    ) => string);

export interface VitePluginVueScriptOptions {
  hoistStatic?: boolean;
  propsDestructure?: boolean | "error";
  globalTypeFiles?: string[];
  [key: string]: unknown;
}

export interface VitePluginVueTemplateCompilerOptions {
  comments?: boolean;
  hoistStatic?: boolean;
  cacheHandlers?: boolean;
  prefixIdentifiers?: boolean;
  [key: string]: unknown;
}

export interface VitePluginVueTemplateOptions {
  compilerOptions?: VitePluginVueTemplateCompilerOptions;
  transformAssetUrls?: boolean | Record<string, unknown>;
  preprocessCustomRequire?: unknown;
  preprocessOptions?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface VitePluginVueStyleOptions {
  trim?: boolean;
  inMap?: unknown;
  [key: string]: unknown;
}

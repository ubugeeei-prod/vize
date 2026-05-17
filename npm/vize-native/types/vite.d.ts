/* eslint-disable */

export interface CssAliasRuleNapi {
  find: string;
  replacement: string;
  isRegex: boolean;
  flags?: string;
}

export interface DefineReplacementNapi {
  key: string;
  value: string;
}

export interface DynamicImportAliasRuleNapi {
  fromPrefix: string;
  toPrefix: string;
}

export interface HmrHashesNapi {
  scriptHash?: string;
  templateHash?: string;
  styleHash?: string;
}

export interface ViteIdPartsNapi {
  request: string;
  querySuffix: string;
}

export interface VitePluginRequestNapi {
  /** Path segment before the query string. */
  path: string;
  /** Query suffix including the leading `?`, or an empty string. */
  querySuffix: string;
  /** Path normalized for macro virtual modules (`.vue.ts` -> `.vue`). */
  normalizedVuePath: string;
  /** For `\0...` virtual macro IDs, the real path without the virtual prefix. */
  strippedVirtualPath?: string;
  /** Whether this ID is a Vize-compiled virtual Vue module. */
  isVizeVirtual: boolean;
  /** Whether this ID is a Vize SSR virtual Vue module. */
  isVizeSsrVirtual: boolean;
  /** Real `.vue` path extracted from a Vize virtual Vue module ID. */
  vizeVirtualPath?: string;
  /** Build-safe ID with Vite's `/@fs` prefix removed when present. */
  normalizedFsId?: string;
  /** Whether the query contains `macro=true`. */
  hasMacroQuery: boolean;
  /** Whether the query contains `definePage`. */
  hasDefinePageQuery: boolean;
  /** Whether this is a `\0` virtual ID carrying a macro query. */
  isMacroVirtualId: boolean;
  /** Whether the request points at a Vue SFC after macro normalization. */
  isVueSfcPath: boolean;
  /** Whether the request is a Vite Vue style virtual query. */
  isVueStyleQuery: boolean;
  /** Style block language, defaulting to `css` for style virtual queries. */
  styleLang?: string;
  /** Style block index for style virtual queries. */
  styleIndex?: number;
  /** Scoped attribute value for style virtual queries. */
  styleScoped?: string;
  /** Whether the style query carries a CSS modules marker. */
  hasStyleModule: boolean;
  /** Extension suffix Vite should see for the style pipeline. */
  styleVirtualSuffix?: string;
  /** Vue boundary file kind: `client`, `server`, or undefined. */
  boundaryKind?: string;
}

export declare function applyViteDefineReplacements(
  code: string,
  defines: Array<DefineReplacementNapi>,
): string;

/**
 * Classify a Vite plugin module request using the native Vize request model.
 * This keeps pure query parsing and virtual module categorization in Rust while
 * JavaScript keeps Vite hook orchestration and filesystem interactions.
 */
export declare function classifyVitePluginRequest(id: string): VitePluginRequestNapi;

export declare function createViteBareImportBases(
  root: string,
  importer?: string | undefined | null,
): Array<string>;

export declare function createViteBareImportCandidates(
  id: string,
  aliasRules: Array<CssAliasRuleNapi>,
  resolvedId?: string | undefined | null,
): Array<string>;

export declare function createViteVirtualId(
  realPath: string,
  ssr?: boolean | undefined | null,
): string;

export declare function detectViteHmrUpdateType(
  prev: HmrHashesNapi | undefined | null,
  next: HmrHashesNapi,
): string;

export declare function fromViteVirtualId(virtualId: string): string;

export declare function generateViteHmrCode(scopeId: string, updateType: string): string;

export declare function hasViteHmrChanges(
  prev: HmrHashesNapi | undefined | null,
  next: HmrHashesNapi,
): boolean;

export declare function isBuiltinViteDefine(key: string): boolean;

export declare function isViteBareSpecifier(id: string): boolean;

export declare function normalizeViteFsIdForBuild(id: string): string;

export declare function normalizeViteRequireBase(
  importer?: string | undefined | null,
): string | null;

export declare function normalizeViteResolvedVuePath(id: string): string | null;

export declare function normalizeViteVirtualVueModuleId(id: string): string;

export declare function resolveViteAliasRequest(
  id: string,
  aliasRules: Array<CssAliasRuleNapi>,
): string | null;

export declare function resolveViteCssImports(
  css: string,
  importer: string,
  aliasRules: Array<CssAliasRuleNapi>,
  isDev?: boolean | undefined | null,
  devUrlBase?: string | undefined | null,
): string;

export declare function resolveViteRelativeImport(id: string, importer: string): string | null;

export declare function resolveViteVuePath(
  root: string,
  id: string,
  importer?: string | undefined | null,
): string;

export declare function rewriteViteDynamicTemplateImports(
  code: string,
  aliasRules: Array<DynamicImportAliasRuleNapi>,
): string;

export declare function rewriteViteStaticAssetUrls(
  code: string,
  aliasRules: Array<DynamicImportAliasRuleNapi>,
): string;

export declare function scopeViteCssForPipeline(css: string, scopeId: string): string;

export declare function shouldApplyViteDefineInVirtualModule(key: string): boolean;

export declare function splitViteIdQuery(id: string): ViteIdPartsNapi;

export declare function toViteBrowserImportPrefix(replacement: string): string;

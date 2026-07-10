/* eslint-disable */

/** Art descriptor for NAPI */
export interface ArtDescriptorNapi {
  filename: string;
  metadata: ArtMetadataNapi;
  variants: Array<ArtVariantNapi>;
  hasScriptSetup: boolean;
  hasScript: boolean;
  styleCount: number;
}

/** Art metadata for NAPI */
export interface ArtMetadataNapi {
  title: string;
  description?: string;
  component?: string;
  category?: string;
  tags: Array<string>;
  status: string;
  order?: number;
}

/** Art parse options for NAPI */
export interface ArtParseOptionsNapi {
  filename?: string;
}

/** Art variant for NAPI */
export interface ArtVariantNapi {
  name: string;
  template: string;
  isDefault: boolean;
  skipVrt: boolean;
}

export interface AutogenConfigNapi {
  maxVariants?: number;
  includeDefault?: boolean;
  includeBooleanToggles?: boolean;
  includeEnumVariants?: boolean;
  includeBoundaryValues?: boolean;
  includeEmptyStrings?: boolean;
}

export interface AutogenOutputNapi {
  variants: Array<GeneratedVariantNapi>;
  artFileContent: string;
  componentName: string;
}

/** Catalog entry for NAPI */
export interface CatalogEntryNapi {
  title: string;
  description?: string;
  category?: string;
  tags: Array<string>;
  status: string;
  variantCount: number;
  docPath: string;
  sourcePath: string;
}

/** Catalog output for NAPI */
export interface CatalogOutputNapi {
  markdown: string;
  filename: string;
  componentCount: number;
  categories: Array<string>;
  tags: Array<string>;
}

/** CSF output for NAPI */
export interface CsfOutputNapi {
  code: string;
  filename: string;
}

/** Declaration generation options for NAPI */
export interface DeclarationOptionsNapi {
  filename?: string;
}

/** Declaration generation result for NAPI */
export interface DeclarationResultNapi {
  code: string;
}

/** Doc options for NAPI */
export interface DocOptionsNapi {
  includeSource?: boolean;
  includeTemplates?: boolean;
  includeMetadata?: boolean;
  includeToc?: boolean;
  tocThreshold?: number;
  basePath?: string;
  title?: string;
}

/** Doc output for NAPI */
export interface DocOutputNapi {
  markdown: string;
  filename: string;
  title: string;
  category?: string;
  variantCount: number;
}

/** Format options for NAPI. */
export interface FormatOptionsNapi {
  printWidth?: number;
  tabWidth?: number;
  useTabs?: boolean;
  semi?: boolean;
  singleQuote?: boolean;
  sortAttributes?: boolean;
  singleAttributePerLine?: boolean;
  maxAttributesPerLine?: number;
  normalizeDirectiveShorthands?: boolean;
}

/** Format result for NAPI. */
export interface FormatResultNapi {
  code: string;
  changed: boolean;
}

export interface GeneratedVariantNapi {
  name: string;
  isDefault: boolean;
  props: any;
  description?: string;
}

/** Palette options for NAPI */
export interface PaletteOptionsNapi {
  inferOptions?: boolean;
  minSelectValues?: number;
  maxSelectValues?: number;
  groupByType?: boolean;
}

/** Palette output for NAPI */
export interface PaletteOutputNapi {
  title: string;
  controls: Array<PropControlNapi>;
  groups: Array<string>;
  json: string;
  typescript: string;
}

/** Prop control for NAPI */
export interface PropControlNapi {
  name: string;
  control: string;
  defaultValue?: any;
  description?: string;
  required: boolean;
  options: Array<SelectOptionNapi>;
  range?: RangeConfigNapi;
  group?: string;
}

export interface PropDefinitionNapi {
  name: string;
  propType: string;
  required: boolean;
  defaultValue?: any;
}

/** Range config for NAPI */
export interface RangeConfigNapi {
  min: number;
  max: number;
  step?: number;
}

/** Select option for NAPI */
export interface SelectOptionNapi {
  label: string;
  value: any;
}

/** Transform Art to Storybook CSF 3.0 */
export declare function artToCsf(
  source: string,
  options?: ArtParseOptionsNapi | undefined | null,
): CsfOutputNapi;

/** Format a Vue SFC source string. */
export declare function formatSfc(
  source: string,
  options?: FormatOptionsNapi | undefined | null,
): FormatResultNapi;

/** Generate catalog from multiple Art sources (high-performance batch) */
export declare function generateArtCatalog(
  sources: Array<string>,
  docOptions?: DocOptionsNapi | undefined | null,
): CatalogOutputNapi;

/** Generate component documentation from Art source */
export declare function generateArtDoc(
  source: string,
  artOptions?: ArtParseOptionsNapi | undefined | null,
  docOptions?: DocOptionsNapi | undefined | null,
): DocOutputNapi;

/** Batch generate docs with parallel processing */
export declare function generateArtDocsBatch(
  sources: Array<string>,
  docOptions?: DocOptionsNapi | undefined | null,
): Array<DocOutputNapi>;

/** Generate props palette from Art source */
export declare function generateArtPalette(
  source: string,
  artOptions?: ArtParseOptionsNapi | undefined | null,
  paletteOptions?: PaletteOptionsNapi | undefined | null,
): PaletteOutputNapi;

/** Generate a Vue SFC `.d.ts` declaration from Croquis analysis. */
export declare function generateDeclaration(
  source: string,
  options?: DeclarationOptionsNapi | undefined | null,
): DeclarationResultNapi;

/** Generate .art.vue variants from component prop definitions */
export declare function generateVariants(
  componentPath: string,
  props: Array<PropDefinitionNapi>,
  config?: AutogenConfigNapi | undefined | null,
): AutogenOutputNapi;

/** Parse Art file (*.art.vue) */
export declare function parseArt(
  source: string,
  options?: ArtParseOptionsNapi | undefined | null,
): ArtDescriptorNapi;

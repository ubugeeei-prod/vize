/**
 * Vite plugin for Musea - Component gallery for Vue components.
 *
 * @example
 * ```ts
 * import { defineConfig } from 'vite';
 * import { vize } from '@vizejs/vite-plugin';
 * import { musea } from '@vizejs/vite-plugin-musea';
 *
 * export default defineConfig({
 *   plugins: [vize(), musea()],
 * });
 * ```
 */

export { musea } from "./plugin/lazy.js";

export type {
  MuseaOptions,
  MuseaPreviewContext,
  MuseaToolbarControl,
  MuseaToolbarOption,
  MuseaGlobals,
  MuseaGlobalValue,
  MuseaVrtOptions,
  MuseaTokenPreviewConfig,
  MuseaTokenPreviewKind,
  MuseaTokenPreviewRule,
  MuseaTheme,
  MuseaThemeColors,
  ResolvedTokenPreview,
  ArtFileInfo,
  ArtMetadata,
  ArtVariant,
  CsfOutput,
  VrtOptions,
  ViewportConfig,
  PaletteApiResponse,
  AnalysisApiResponse,
  A11yOptions,
  A11yResult,
  CaptureConfig,
  BeforeCaptureContext,
  ComparisonConfig,
  DiffStyle,
  CiConfig,
} from "./types/index.js";

export { MuseaVrtRunner } from "./root-vrt.js";

export { generateVrtReport, generateVrtJsonReport } from "./vrt/report.js";
export type { VrtResult, VrtSummary } from "./vrt/types.js";

export { UnsupportedDiffStyleError } from "./vrt/diff-style-option.js";

export {
  processStyleDictionary,
  parseTokens,
  generateTokensHtml,
  generateTokensMarkdown,
  buildTokenMap,
  resolveReferences,
  scanTokenUsage,
  resolveTokenPreview,
  type DesignToken,
  type TokenCategory,
  type StyleDictionaryConfig,
  type StyleDictionaryOutput,
  type TokenUsageMap,
} from "./style-dictionary.js";

export { MuseaA11yRunner } from "./root-a11y.js";
export type { A11ySummary } from "./a11y/index.js";

export {
  generateArtFile,
  writeArtFile,
  type AutogenOptions,
  type AutogenOutput,
  type PropDefinition,
  type GeneratedVariant,
} from "./autogen/index.js";

import { musea } from "./plugin/lazy.js";
export default musea;

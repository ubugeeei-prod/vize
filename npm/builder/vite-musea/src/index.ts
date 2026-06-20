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

export { musea } from "./plugin/index.js";

export type {
  MuseaOptions,
  MuseaTheme,
  MuseaThemeColors,
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
  ComparisonConfig,
  CiConfig,
} from "./types/index.js";

export {
  MuseaVrtRunner,
  generateVrtReport,
  generateVrtJsonReport,
  type VrtResult,
  type VrtSummary,
} from "./vrt.js";

export {
  processStyleDictionary,
  parseTokens,
  generateTokensHtml,
  generateTokensMarkdown,
  buildTokenMap,
  resolveReferences,
  scanTokenUsage,
  type DesignToken,
  type TokenCategory,
  type StyleDictionaryConfig,
  type StyleDictionaryOutput,
  type TokenUsageMap,
} from "./style-dictionary.js";

export { MuseaA11yRunner, type A11ySummary } from "./a11y/index.js";

export {
  generateArtFile,
  writeArtFile,
  type AutogenOptions,
  type AutogenOutput,
  type PropDefinition,
  type GeneratedVariant,
} from "./autogen/index.js";

import { musea } from "./plugin/index.js";
export default musea;

/**
 * Style Dictionary integration for Musea.
 * Generates design token documentation from Style Dictionary format.
 *
 * This file re-exports from the split tokens submodules for backward compatibility.
 */

export {
  parseTokens,
  type DesignToken,
  type TokenCategory,
  type StyleDictionaryOutput,
  type StyleDictionaryConfig,
  type TokenTransform,
} from "./tokens/parser.js";

export {
  buildTokenMap,
  resolveReferences,
  readRawTokenFile,
  writeRawTokenFile,
  setTokenAtPath,
  deleteTokenAtPath,
  validateSemanticReference,
  findDependentTokens,
  normalizeTokenValue,
  scanTokenUsage,
  type TokenUsageMatch,
  type TokenUsageEntry,
  type TokenUsageMap,
} from "./tokens/resolver.js";

export {
  generateTokensHtml,
  generateTokensMarkdown,
  processStyleDictionary,
} from "./tokens/generator.js";

import { processStyleDictionary } from "./tokens/generator.js";
export default processStyleDictionary;

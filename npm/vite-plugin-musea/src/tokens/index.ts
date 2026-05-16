/**
 * Style Dictionary integration for Musea.
 *
 * Re-exports all token functionality from submodules:
 * - parser: Token file/directory parsing and type definitions
 * - resolver: Token map building, reference resolution, CRUD, validation, usage scanning
 * - generator: HTML, Markdown, and JSON documentation generation
 */

export {
  parseTokens,
  type DesignToken,
  type TokenCategory,
  type StyleDictionaryOutput,
  type StyleDictionaryConfig,
  type TokenTransform,
} from "./parser.js";

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
} from "./resolver.js";

export { generateTokensHtml, generateTokensMarkdown, processStyleDictionary } from "./generator.js";

import { processStyleDictionary } from "./generator.js";
export default processStyleDictionary;

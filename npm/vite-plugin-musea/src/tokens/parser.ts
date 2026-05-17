/**
 * Token parsing utilities for Style Dictionary integration.
 *
 * Thin native binding for design token files (JSON) and directories.
 */

import { normalizeCategories, tokenNative } from "./native.js";

/**
 * Design token value.
 */
export interface DesignToken {
  value: string | number;
  type?: string;
  description?: string;
  attributes?: Record<string, unknown>;
  $tier?: "primitive" | "semantic";
  $reference?: string;
  $resolvedValue?: string | number;
}

/**
 * Token category (e.g., colors, spacing, typography).
 */
export interface TokenCategory {
  name: string;
  tokens: Record<string, DesignToken>;
  subcategories?: TokenCategory[];
}

/**
 * Style Dictionary output format.
 */
export interface StyleDictionaryOutput {
  categories: TokenCategory[];
  metadata: {
    name: string;
    version?: string;
    generatedAt: string;
  };
}

/**
 * Configuration for Style Dictionary integration.
 */
export interface StyleDictionaryConfig {
  /**
   * Path to tokens JSON/JS file or directory.
   */
  tokensPath: string;

  /**
   * Output format for documentation.
   * @default 'html'
   */
  outputFormat?: "html" | "json" | "markdown";

  /**
   * Output directory for generated documentation.
   * @default '.vize/tokens'
   */
  outputDir?: string;

  /**
   * Custom token transformations.
   */
  transforms?: TokenTransform[];
}

/**
 * Token transformation function.
 */
export type TokenTransform = (token: DesignToken, path: string[]) => DesignToken;

/**
 * Parse Style Dictionary tokens file or directory.
 */
export async function parseTokens(tokensPath: string): Promise<TokenCategory[]> {
  return normalizeCategories(tokenNative().parseDesignTokensFromPath(tokensPath));
}

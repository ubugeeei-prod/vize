import { loadNative } from "../native-loader.js";
import type { DesignToken, TokenCategory } from "./parser.js";

interface ResolvedTokensNative {
  categories: TokenCategory[];
  tokenMap: Record<string, DesignToken>;
  tokenCount: number;
  primitiveCount: number;
  semanticCount: number;
}

interface ValidationResultNative {
  valid: boolean;
  error?: string;
}

interface TokenNativeBinding {
  parseDesignTokensFromPath(path: string): TokenCategory[];
  buildDesignTokenMap(categories: TokenCategory[]): Record<string, DesignToken>;
  resolveDesignTokenReferences(categories: TokenCategory[]): ResolvedTokensNative;
  flattenDesignTokenCategories(categories: TokenCategory[]): unknown[];
  generateDesignTokensMarkdown(categories: TokenCategory[], generatedAt?: string): string;
  validateDesignTokenReference(
    tokenMap: Record<string, DesignToken>,
    reference: string,
    selfPath?: string,
  ): ValidationResultNative;
  findDependentDesignTokens(tokenMap: Record<string, DesignToken>, targetPath: string): string[];
}

export function tokenNative(): TokenNativeBinding {
  return loadNative() as typeof loadNative extends () => infer T ? T & TokenNativeBinding : never;
}

export function normalizeCategories(categories: TokenCategory[]): TokenCategory[] {
  for (const category of categories) {
    category.tokens = nullRecord(category.tokens);
    if (category.subcategories) {
      normalizeCategories(category.subcategories);
    }
  }
  return categories;
}

export function nullRecord<T>(record: Record<string, T>): Record<string, T> {
  return Object.assign(Object.create(null) as Record<string, T>, record);
}

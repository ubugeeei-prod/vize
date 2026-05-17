import { loadNative } from "./native.js";

export interface TokenValue {
  value: string | number;
  type?: string;
  description?: string;
  $tier?: "primitive" | "semantic";
  $reference?: string;
  $resolvedValue?: string | number;
}

export interface TokenCategory {
  name: string;
  tokens: Record<string, TokenValue>;
  subcategories?: TokenCategory[];
}

export interface FlattenedToken {
  name: string;
  path: string;
  categoryPath: string[];
  value: string | number;
  type?: string;
  description?: string;
}

export async function parseTokensFromPath(tokensPath: string): Promise<TokenCategory[]> {
  return normalizeCategories(loadNative().parseDesignTokensFromPath(tokensPath) as TokenCategory[]);
}

export function generateTokensMarkdown(categories: TokenCategory[]): string {
  return loadNative().generateDesignTokensMarkdown(categories) as string;
}

export function flattenTokenCategories(categories: TokenCategory[]): FlattenedToken[] {
  return loadNative().flattenDesignTokenCategories(categories) as FlattenedToken[];
}

function normalizeCategories(categories: TokenCategory[]): TokenCategory[] {
  for (const category of categories) {
    category.tokens = Object.assign(Object.create(null), category.tokens);
    if (category.subcategories) {
      normalizeCategories(category.subcategories);
    }
  }
  return categories;
}

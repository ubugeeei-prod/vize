import { isCssTokenPath, parseCssTokenFiles } from "./css-tokens.js";
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
  if (await isCssTokenPath(tokensPath)) {
    return normalizeCategories(await parseCssTokenFiles(tokensPath));
  }
  return categoriesFromNativeResult(loadNative().parseDesignTokensFromPath(tokensPath));
}

export function categoriesFromNativeResult(value: unknown): TokenCategory[] {
  const parsed = parseJsonResult<unknown>(value);
  if (!Array.isArray(parsed)) {
    throw new TypeError("Design token parser did not return a category array");
  }
  return normalizeCategories(parsed as TokenCategory[]);
}

export function generateTokensMarkdown(categories: TokenCategory[]): string {
  return String(loadNative().generateDesignTokensMarkdown(JSON.stringify(categories)));
}

export function flattenTokenCategories(categories: TokenCategory[]): FlattenedToken[] {
  return parseJsonResult<FlattenedToken[]>(
    loadNative().flattenDesignTokenCategories(JSON.stringify(categories)),
  );
}

function parseJsonResult<T>(value: unknown): T {
  if (typeof value === "string") return JSON.parse(value) as T;
  return value as T;
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

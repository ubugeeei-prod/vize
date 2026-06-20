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

interface RawTokenNativeBinding {
  parseDesignTokensFromPath(path: string): string;
  buildDesignTokenMap(categories: string): string;
  resolveDesignTokenReferences(categories: string): string;
  flattenDesignTokenCategories(categories: string): string;
  generateDesignTokensMarkdown(categories: string, generatedAt?: string): string;
  validateDesignTokenReference(tokenMap: string, reference: string, selfPath?: string): string;
  findDependentDesignTokens(tokenMap: string, targetPath: string): string[];
}

export function tokenNative(): TokenNativeBinding {
  const native = loadNative() as typeof loadNative extends () => infer T
    ? T & RawTokenNativeBinding
    : RawTokenNativeBinding;

  return {
    parseDesignTokensFromPath(path) {
      return normalizeCategories(parseJsonResult(native.parseDesignTokensFromPath(path)));
    },
    buildDesignTokenMap(categories) {
      return nullRecord(parseJsonResult(native.buildDesignTokenMap(JSON.stringify(categories))));
    },
    resolveDesignTokenReferences(categories) {
      const resolved = parseJsonResult<ResolvedTokensNative>(
        native.resolveDesignTokenReferences(JSON.stringify(categories)),
      );
      return {
        ...resolved,
        categories: normalizeCategories(resolved.categories),
        tokenMap: nullRecord(resolved.tokenMap),
      };
    },
    flattenDesignTokenCategories(categories) {
      return parseJsonResult(native.flattenDesignTokenCategories(JSON.stringify(categories)));
    },
    generateDesignTokensMarkdown(categories, generatedAt) {
      return native.generateDesignTokensMarkdown(JSON.stringify(categories), generatedAt);
    },
    validateDesignTokenReference(tokenMap, reference, selfPath) {
      return parseJsonResult(
        native.validateDesignTokenReference(JSON.stringify(tokenMap), reference, selfPath),
      );
    },
    findDependentDesignTokens(tokenMap, targetPath) {
      return native.findDependentDesignTokens(JSON.stringify(tokenMap), targetPath);
    },
  };
}

function parseJsonResult<T>(value: unknown): T {
  if (typeof value === "string") {
    return JSON.parse(value) as T;
  }
  return value as T;
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

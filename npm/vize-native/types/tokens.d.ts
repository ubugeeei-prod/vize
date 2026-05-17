/* eslint-disable */

export interface DesignTokenNapi {
  value: string | number;
  type?: string;
  description?: string;
  attributes?: any;
  $tier?: "primitive" | "semantic";
  $reference?: string;
  $resolvedValue?: string | number;
}

export interface DesignTokenCategoryNapi {
  name: string;
  tokens: Record<string, DesignTokenNapi>;
  subcategories?: Array<DesignTokenCategoryNapi>;
}

export interface DesignTokenValidationNapi {
  valid: boolean;
  error?: string;
}

export interface FlattenedDesignTokenNapi {
  name: string;
  path: string;
  categoryPath: Array<string>;
  value: string | number;
  type?: string;
  description?: string;
}

export interface ResolvedDesignTokensNapi {
  categories: Array<DesignTokenCategoryNapi>;
  tokenMap: Record<string, DesignTokenNapi>;
  tokenCount: number;
  primitiveCount: number;
  semanticCount: number;
}

export declare function buildDesignTokenMap(
  categories: Array<DesignTokenCategoryNapi>,
): Record<string, DesignTokenNapi>;

export declare function findDependentDesignTokens(
  tokenMap: Record<string, DesignTokenNapi>,
  targetPath: string,
): Array<string>;

export declare function flattenDesignTokenCategories(
  categories: Array<DesignTokenCategoryNapi>,
): Array<FlattenedDesignTokenNapi>;

export declare function generateDesignTokensMarkdown(
  categories: Array<DesignTokenCategoryNapi>,
  generatedAt?: string | undefined | null,
): string;

export declare function parseDesignTokensFromJson(source: string): Array<DesignTokenCategoryNapi>;

export declare function parseDesignTokensFromPath(
  tokensPath: string,
): Array<DesignTokenCategoryNapi>;

export declare function resolveDesignTokenReferences(
  categories: Array<DesignTokenCategoryNapi>,
): ResolvedDesignTokensNapi;

export declare function validateDesignTokenReference(
  tokenMap: Record<string, DesignTokenNapi>,
  reference: string,
  selfPath?: string | undefined | null,
): DesignTokenValidationNapi;

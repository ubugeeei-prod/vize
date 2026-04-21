/**
 * Token parsing utilities for Style Dictionary integration.
 *
 * Reads and parses design token files (JSON) and directories,
 * flattening nested structures into categorized token collections.
 */

import fs from "node:fs";
import path from "node:path";

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
 * Parse Style Dictionary tokens file.
 */
export async function parseTokens(tokensPath: string): Promise<TokenCategory[]> {
  const absolutePath = path.resolve(tokensPath);
  const stat = await fs.promises.stat(absolutePath);

  if (stat.isDirectory()) {
    return parseTokenDirectory(absolutePath);
  }

  const content = await fs.promises.readFile(absolutePath, "utf-8");
  const tokens = JSON.parse(content);
  return flattenTokens(tokens);
}

/**
 * Parse tokens from a directory.
 */
async function parseTokenDirectory(dirPath: string): Promise<TokenCategory[]> {
  const mergedTokens: Record<string, unknown> = {};
  await mergeTokenDirectory(mergedTokens, dirPath);
  return flattenTokens(mergedTokens);
}

async function mergeTokenDirectory(
  target: Record<string, unknown>,
  dirPath: string,
): Promise<void> {
  const entries = await fs.promises.readdir(dirPath, { withFileTypes: true });

  for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
    const fullPath = path.join(dirPath, entry.name);

    if (entry.isDirectory()) {
      await mergeTokenDirectory(target, fullPath);
      continue;
    }

    if (
      !entry.isFile() ||
      (!entry.name.endsWith(".json") && !entry.name.endsWith(".tokens.json"))
    ) {
      continue;
    }

    const content = await fs.promises.readFile(fullPath, "utf-8");
    const tokens = JSON.parse(content) as Record<string, unknown>;
    deepMergeTokenTrees(target, tokens);
  }
}

function deepMergeTokenTrees(
  target: Record<string, unknown>,
  source: Record<string, unknown>,
): void {
  for (const [key, value] of Object.entries(source)) {
    const existing = target[key];

    if (
      isPlainObject(existing) &&
      isPlainObject(value) &&
      !isTokenValue(existing) &&
      !isTokenValue(value)
    ) {
      deepMergeTokenTrees(existing, value);
      continue;
    }

    target[key] = value;
  }
}

/**
 * Flatten nested token structure into categories.
 */
function flattenTokens(tokens: Record<string, unknown>, prefix: string[] = []): TokenCategory[] {
  const categories: TokenCategory[] = [];

  for (const [key, value] of Object.entries(tokens)) {
    if (isTokenValue(value)) {
      // This is a token leaf node
      continue;
    }

    if (typeof value === "object" && value !== null) {
      const categoryTokens = extractTokens(value as Record<string, unknown>);
      const subcategories = flattenTokens(value as Record<string, unknown>, [...prefix, key]);

      if (Object.keys(categoryTokens).length > 0 || subcategories.length > 0) {
        categories.push({
          name: formatCategoryName(key),
          tokens: categoryTokens,
          subcategories: subcategories.length > 0 ? subcategories : undefined,
        });
      }
    }
  }

  return categories;
}

/**
 * Extract token values from an object.
 */
function extractTokens(obj: Record<string, unknown>): Record<string, DesignToken> {
  const tokens: Record<string, DesignToken> = {};

  for (const [key, value] of Object.entries(obj)) {
    if (isTokenValue(value)) {
      tokens[key] = normalizeToken(value as Record<string, unknown>);
    }
  }

  return tokens;
}

/**
 * Check if a value is a token definition.
 */
function isTokenValue(value: unknown): boolean {
  if (typeof value !== "object" || value === null) return false;
  const obj = value as Record<string, unknown>;
  // Support both "value" and DTCG "$value" formats
  return (
    ("value" in obj && (typeof obj.value === "string" || typeof obj.value === "number")) ||
    ("$value" in obj && (typeof obj.$value === "string" || typeof obj.$value === "number"))
  );
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * Normalize token to DesignToken interface.
 */
function normalizeToken(raw: Record<string, unknown>): DesignToken {
  const token: DesignToken = {
    value: (raw.value ?? raw.$value) as string | number,
    type: (raw.type ?? raw.$type) as string | undefined,
    description: raw.description as string | undefined,
    attributes: raw.attributes as Record<string, unknown> | undefined,
  };
  if (raw.$tier === "primitive" || raw.$tier === "semantic") {
    token.$tier = raw.$tier;
  }
  if (typeof raw.$reference === "string") {
    token.$reference = raw.$reference;
  }
  return token;
}

/**
 * Format category name for display.
 */
function formatCategoryName(name: string): string {
  return name
    .replace(/[-_]/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .split(" ")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(" ");
}

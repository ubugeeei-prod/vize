/**
 * Token resolution, CRUD operations, and validation.
 *
 * Handles building flat token maps from categories, resolving reference chains,
 * reading/writing raw token files, and validating semantic references.
 */

import fs from "node:fs";

import type { DesignToken, TokenCategory } from "./parser.js";

// Re-export usage scanning and normalization from usage
export { normalizeTokenValue, scanTokenUsage } from "./usage.js";

/**
 * Token usage match within a single CSS property.
 */
export interface TokenUsageMatch {
  line: number;
  lineContent: string;
  property: string;
}

/**
 * Token usage entry for a single art file.
 */
export interface TokenUsageEntry {
  artPath: string;
  artTitle: string;
  artCategory?: string;
  matches: TokenUsageMatch[];
}

/**
 * Map of token paths to their usage locations across art files.
 */
export type TokenUsageMap = Record<string, TokenUsageEntry[]>;

const REFERENCE_PATTERN = /^\{(.+)\}$/;
const MAX_RESOLVE_DEPTH = 10;
const UNSAFE_TOKEN_PATH_SEGMENTS = new Set(["__proto__", "prototype", "constructor"]);

function parseTokenPath(dotPath: string): string[] {
  const parts = dotPath.split(".");
  if (parts.length === 0 || parts.some((part) => part.trim() === "")) {
    throw new Error(`Invalid token path "${dotPath}"`);
  }

  const unsafeSegment = parts.find((part) => UNSAFE_TOKEN_PATH_SEGMENTS.has(part));
  if (unsafeSegment) {
    throw new Error(`Token path segment "${unsafeSegment}" is not allowed`);
  }

  return parts;
}

/**
 * Flatten nested categories into a flat map keyed by dot-path.
 */
export function buildTokenMap(
  categories: TokenCategory[],
  prefix: string[] = [],
): Record<string, DesignToken> {
  const map: Record<string, DesignToken> = {};

  for (const cat of categories) {
    const catKey = cat.name.toLowerCase().replace(/\s+/g, "-");
    const catPath = [...prefix, catKey];

    for (const [name, token] of Object.entries(cat.tokens)) {
      const dotPath = [...catPath, name].join(".");
      map[dotPath] = token;
    }

    if (cat.subcategories) {
      const subMap = buildTokenMap(cat.subcategories, catPath);
      Object.assign(map, subMap);
    }
  }

  return map;
}

/**
 * Resolve references in categories, setting $tier, $reference, and $resolvedValue.
 */
export function resolveReferences(
  categories: TokenCategory[],
  tokenMap: Record<string, DesignToken>,
): void {
  for (const cat of categories) {
    for (const token of Object.values(cat.tokens)) {
      resolveTokenReference(token, tokenMap);
    }
    if (cat.subcategories) {
      resolveReferences(cat.subcategories, tokenMap);
    }
  }
}

function resolveTokenReference(token: DesignToken, tokenMap: Record<string, DesignToken>): void {
  if (typeof token.value === "string") {
    const match = token.value.match(REFERENCE_PATTERN);
    if (match) {
      token.$tier = token.$tier ?? "semantic";
      token.$reference = match[1];
      token.$resolvedValue = resolveValue(match[1], tokenMap, 0, new Set());
      return;
    }
  }
  token.$tier = token.$tier ?? "primitive";
}

function resolveValue(
  ref: string,
  tokenMap: Record<string, DesignToken>,
  depth: number,
  visited: Set<string>,
): string | number | undefined {
  if (depth >= MAX_RESOLVE_DEPTH || visited.has(ref)) return undefined;
  visited.add(ref);

  const target = tokenMap[ref];
  if (!target) return undefined;

  if (typeof target.value === "string") {
    const match = target.value.match(REFERENCE_PATTERN);
    if (match) {
      return resolveValue(match[1], tokenMap, depth + 1, visited);
    }
  }
  return target.value;
}

/**
 * Read raw JSON token file.
 */
export async function readRawTokenFile(tokensPath: string): Promise<Record<string, unknown>> {
  const content = await fs.promises.readFile(tokensPath, "utf-8");
  return JSON.parse(content) as Record<string, unknown>;
}

/**
 * Write raw JSON token file atomically (write tmp, rename).
 */
export async function writeRawTokenFile(
  tokensPath: string,
  data: Record<string, unknown>,
): Promise<void> {
  const tmpPath = tokensPath + ".tmp";
  await fs.promises.writeFile(tmpPath, JSON.stringify(data, null, 2) + "\n", "utf-8");
  await fs.promises.rename(tmpPath, tokensPath);
}

/**
 * Set a token at a dot-separated path in the raw JSON structure.
 */
export function setTokenAtPath(
  data: Record<string, unknown>,
  dotPath: string,
  token: Omit<DesignToken, "$resolvedValue">,
): void {
  const parts = parseTokenPath(dotPath);
  let current: Record<string, unknown> = data;

  for (let i = 0; i < parts.length - 1; i++) {
    const key = parts[i];
    if (typeof current[key] !== "object" || current[key] === null) {
      current[key] = {};
    }
    current = current[key] as Record<string, unknown>;
  }

  const leafKey = parts[parts.length - 1];
  const raw: Record<string, unknown> = { value: token.value };
  if (token.type) raw.type = token.type;
  if (token.description) raw.description = token.description;
  if (token.$tier) raw.$tier = token.$tier;
  if (token.$reference) raw.$reference = token.$reference;
  if (token.attributes) raw.attributes = token.attributes;
  current[leafKey] = raw;
}

/**
 * Delete a token at a dot-separated path, cleaning empty parents.
 */
export function deleteTokenAtPath(data: Record<string, unknown>, dotPath: string): boolean {
  const parts = parseTokenPath(dotPath);
  const parents: Array<{ obj: Record<string, unknown>; key: string }> = [];
  let current: Record<string, unknown> = data;

  for (let i = 0; i < parts.length - 1; i++) {
    const key = parts[i];
    if (typeof current[key] !== "object" || current[key] === null) {
      return false;
    }
    parents.push({ obj: current, key });
    current = current[key] as Record<string, unknown>;
  }

  const leafKey = parts[parts.length - 1];
  if (!(leafKey in current)) return false;
  delete current[leafKey];

  // Clean empty parents
  for (let i = parents.length - 1; i >= 0; i--) {
    const { obj, key } = parents[i];
    const child = obj[key] as Record<string, unknown>;
    if (Object.keys(child).length === 0) {
      delete obj[key];
    } else {
      break;
    }
  }

  return true;
}

/**
 * Validate that a semantic reference points to an existing token and has no cycles.
 */
export function validateSemanticReference(
  tokenMap: Record<string, DesignToken>,
  reference: string,
  selfPath?: string,
): { valid: boolean; error?: string } {
  if (!tokenMap[reference]) {
    return { valid: false, error: `Reference target "${reference}" does not exist` };
  }

  // Check for cycles
  const visited = new Set<string>();
  if (selfPath) visited.add(selfPath);
  let current = reference;
  let depth = 0;

  while (depth < MAX_RESOLVE_DEPTH) {
    if (visited.has(current)) {
      return { valid: false, error: `Circular reference detected at "${current}"` };
    }
    visited.add(current);

    const target = tokenMap[current];
    if (!target) break;

    if (typeof target.value === "string") {
      const match = target.value.match(REFERENCE_PATTERN);
      if (match) {
        current = match[1];
        depth++;
        continue;
      }
    }
    break;
  }

  if (depth >= MAX_RESOLVE_DEPTH) {
    return { valid: false, error: "Reference chain too deep (max 10)" };
  }

  return { valid: true };
}

/**
 * Find all tokens that reference the given path.
 */
export function findDependentTokens(
  tokenMap: Record<string, DesignToken>,
  targetPath: string,
): string[] {
  const dependents: string[] = [];
  for (const [path, token] of Object.entries(tokenMap)) {
    if (typeof token.value === "string") {
      const match = token.value.match(REFERENCE_PATTERN);
      if (match && match[1] === targetPath) {
        dependents.push(path);
      }
    }
  }
  return dependents;
}

/**
 * Token usage scanning and value normalization.
 *
 * Scans art file `<style>` blocks for CSS property values that match
 * design token values, and provides value normalization utilities.
 */

import fs from "node:fs";

import type { DesignToken } from "./parser.js";
import type { TokenUsageMap } from "./resolver.js";

/**
 * Normalize a token value for comparison.
 * - Lowercase, trim
 * - Leading-zero: `.5rem` -> `0.5rem`
 * - Short hex: `#fff` -> `#ffffff`
 */
export function normalizeTokenValue(value: string | number): string {
  let v = String(value).trim().toLowerCase();

  // Expand short hex (#abc -> #aabbcc, #abcd -> #aabbccdd)
  const shortHex = v.match(/^#([0-9a-f])([0-9a-f])([0-9a-f])([0-9a-f])?$/);
  if (shortHex) {
    const [, r, g, b, a] = shortHex;
    v = a ? `#${r}${r}${g}${g}${b}${b}${a}${a}` : `#${r}${r}${g}${g}${b}${b}`;
  }

  // Add leading zero: `.5rem` -> `0.5rem`
  v = v.replace(/(?<![0-9])\.(\d)/g, "0.$1");

  return v;
}

const STYLE_BLOCK_RE = /<style[^>]*>([\s\S]*?)<\/style>/g;
const CSS_PROPERTY_RE = /^\s*([\w-]+)\s*:\s*(.+?)\s*;?\s*$/;

/**
 * Scan art file sources for token value matches in `<style>` blocks.
 */
export function scanTokenUsage(
  artFiles: Map<string, { path: string; metadata: { title: string; category?: string } }>,
  tokenMap: Record<string, DesignToken>,
): TokenUsageMap {
  // Build reverse lookup: normalizedValue -> tokenPath[]
  const valueLookup = new Map<string, string[]>();
  for (const [tokenPath, token] of Object.entries(tokenMap)) {
    const rawValue = token.$resolvedValue ?? token.value;
    const normalized = normalizeTokenValue(rawValue);
    if (!normalized) continue;
    const existing = valueLookup.get(normalized);
    if (existing) {
      existing.push(tokenPath);
    } else {
      valueLookup.set(normalized, [tokenPath]);
    }
  }

  const usageMap: TokenUsageMap = {};

  for (const [artPath, artInfo] of artFiles) {
    let source: string;
    try {
      source = fs.readFileSync(artPath, "utf-8");
    } catch {
      continue;
    }

    const allLines = source.split("\n");

    // Find style block line offsets
    const styleRegions: Array<{ startLine: number; content: string }> = [];
    let match: RegExpExecArray | null;
    STYLE_BLOCK_RE.lastIndex = 0;
    while ((match = STYLE_BLOCK_RE.exec(source)) !== null) {
      const beforeMatch = source.slice(0, match.index);
      const startTag = source.slice(match.index, match.index + match[0].indexOf(match[1]));
      const startLine = beforeMatch.split("\n").length + startTag.split("\n").length - 1;
      styleRegions.push({ startLine, content: match[1] });
    }

    // Scan each style block line
    for (const region of styleRegions) {
      const lines = region.content.split("\n");
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const propMatch = line.match(CSS_PROPERTY_RE);
        if (!propMatch) continue;

        const property = propMatch[1];
        const valueStr = propMatch[2];

        // Split on whitespace for multi-value properties (e.g., `border: 1px solid #3b82f6`)
        const valueParts = valueStr.split(/\s+/);
        for (const part of valueParts) {
          const normalizedPart = normalizeTokenValue(part);
          const matchingTokens = valueLookup.get(normalizedPart);
          if (!matchingTokens) continue;

          const lineNumber = region.startLine + i;
          const lineContent = allLines[lineNumber - 1]?.trim() ?? line.trim();

          for (const tokenPath of matchingTokens) {
            if (!usageMap[tokenPath]) {
              usageMap[tokenPath] = [];
            }

            // Find or create entry for this art file
            let entry = usageMap[tokenPath].find((e) => e.artPath === artPath);
            if (!entry) {
              entry = {
                artPath,
                artTitle: artInfo.metadata.title,
                artCategory: artInfo.metadata.category,
                matches: [],
              };
              usageMap[tokenPath].push(entry);
            }

            // Avoid duplicate matches on same line+property
            if (!entry.matches.some((m) => m.line === lineNumber && m.property === property)) {
              entry.matches.push({ line: lineNumber, lineContent, property });
            }
          }
        }
      }
    }
  }

  return usageMap;
}

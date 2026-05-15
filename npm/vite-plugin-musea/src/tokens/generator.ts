/**
 * Token documentation generators for Style Dictionary integration.
 *
 * Generates HTML, Markdown, and JSON documentation from parsed token categories,
 * and provides the main processStyleDictionary orchestrator function.
 */

import fs from "node:fs";
import path from "node:path";

import type { TokenCategory, StyleDictionaryConfig, StyleDictionaryOutput } from "./parser.js";
import { parseTokens } from "./parser.js";
import { escapeHtml } from "../utils.js";

const SAFE_CSS_COLOR_PATTERN = /^(?:#[0-9a-fA-F]{3,8}|(?:rgb|hsl)a?\(\s*[-+.\d,%\s]+\))$/;

function safeCssColor(value: string | number, type?: string): string | null {
  if (typeof value !== "string") return null;

  const trimmed = value.trim();
  const looksLikeColor =
    type === "color" ||
    trimmed.startsWith("#") ||
    trimmed.startsWith("rgb") ||
    trimmed.startsWith("hsl");

  return looksLikeColor && SAFE_CSS_COLOR_PATTERN.test(trimmed) ? trimmed : null;
}

/**
 * Generate HTML documentation for tokens.
 */
export function generateTokensHtml(categories: TokenCategory[]): string {
  const renderToken = (
    name: string,
    token: { value: string | number; type?: string; description?: string },
  ): string => {
    const color = safeCssColor(token.value, token.type);

    return `
      <div class="token">
        <div class="token-preview">
          ${color ? `<div class="color-swatch" style="background: ${color}"></div>` : ""}
        </div>
        <div class="token-info">
          <div class="token-name">${escapeHtml(name)}</div>
          <div class="token-value">${escapeHtml(String(token.value))}</div>
          ${token.description ? `<div class="token-description">${escapeHtml(token.description)}</div>` : ""}
        </div>
      </div>
    `;
  };

  const renderCategory = (category: TokenCategory, level: number = 2): string => {
    const heading = `h${Math.min(level, 6)}`;
    let html = `<${heading}>${escapeHtml(category.name)}</${heading}>`;
    html += '<div class="tokens-grid">';

    for (const [name, token] of Object.entries(category.tokens)) {
      html += renderToken(name, token);
    }

    html += "</div>";

    if (category.subcategories) {
      for (const sub of category.subcategories) {
        html += renderCategory(sub, level + 1);
      }
    }

    return html;
  };

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Design Tokens - Musea</title>
  <style>
    :root {
      --musea-bg: #0d0d0d;
      --musea-bg-secondary: #1a1815;
      --musea-text: #e6e9f0;
      --musea-text-muted: #7b8494;
      --musea-accent: #a34828;
      --musea-border: #3a3530;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: 'Inter', -apple-system, sans-serif;
      background: var(--musea-bg);
      color: var(--musea-text);
      line-height: 1.6;
      padding: 2rem;
    }
    h1 { margin-bottom: 2rem; color: var(--musea-accent); }
    h2 { margin: 2rem 0 1rem; padding-bottom: 0.5rem; border-bottom: 1px solid var(--musea-border); }
    h3, h4, h5, h6 { margin: 1.5rem 0 0.75rem; }
    .tokens-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
      gap: 1rem;
      margin-bottom: 1.5rem;
    }
    .token {
      background: var(--musea-bg-secondary);
      border: 1px solid var(--musea-border);
      border-radius: 8px;
      padding: 1rem;
      display: flex;
      gap: 1rem;
      align-items: center;
    }
    .token-preview {
      flex-shrink: 0;
      width: 48px;
      height: 48px;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    .color-swatch {
      width: 48px;
      height: 48px;
      border-radius: 8px;
      border: 1px solid var(--musea-border);
    }
    .token-info {
      flex: 1;
      min-width: 0;
    }
    .token-name {
      font-weight: 600;
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.875rem;
    }
    .token-value {
      color: var(--musea-text-muted);
      font-family: 'JetBrains Mono', monospace;
      font-size: 0.75rem;
      word-break: break-all;
    }
    .token-description {
      color: var(--musea-text-muted);
      font-size: 0.75rem;
      margin-top: 0.25rem;
    }
  </style>
</head>
<body>
  <h1>Design Tokens</h1>
  ${categories.map((cat) => renderCategory(cat)).join("")}
</body>
</html>`;
}

/**
 * Generate Markdown documentation for tokens.
 */
export function generateTokensMarkdown(categories: TokenCategory[]): string {
  const renderCategory = (category: TokenCategory, level: number = 2): string => {
    const heading = "#".repeat(level);
    let md = `\n${heading} ${category.name}\n\n`;

    if (Object.keys(category.tokens).length > 0) {
      md += "| Token | Value | Description |\n";
      md += "|-------|-------|-------------|\n";

      for (const [name, token] of Object.entries(category.tokens)) {
        const desc = token.description || "-";
        md += `| \`${name}\` | \`${token.value}\` | ${desc} |\n`;
      }
      md += "\n";
    }

    if (category.subcategories) {
      for (const sub of category.subcategories) {
        md += renderCategory(sub, level + 1);
      }
    }

    return md;
  };

  let markdown = "# Design Tokens\n\n";
  markdown += `> Generated by Musea on ${new Date().toISOString()}\n`;

  for (const category of categories) {
    markdown += renderCategory(category);
  }

  return markdown;
}

/**
 * Style Dictionary plugin for Musea.
 * Parses tokens and generates documentation in the specified format.
 */
export async function processStyleDictionary(
  config: StyleDictionaryConfig,
): Promise<StyleDictionaryOutput> {
  const categories = await parseTokens(config.tokensPath);
  const outputDir = config.outputDir ?? ".vize/tokens";
  const outputFormat = config.outputFormat ?? "html";

  // Ensure output directory exists
  await fs.promises.mkdir(outputDir, { recursive: true });

  // Generate documentation
  let content: string;
  let filename: string;

  switch (outputFormat) {
    case "html":
      content = generateTokensHtml(categories);
      filename = "tokens.html";
      break;
    case "markdown":
      content = generateTokensMarkdown(categories);
      filename = "tokens.md";
      break;
    case "json":
    default:
      content = JSON.stringify({ categories }, null, 2);
      filename = "tokens.json";
  }

  const outputPath = path.join(outputDir, filename);
  await fs.promises.writeFile(outputPath, content, "utf-8");

  console.log(`[musea] Generated token documentation: ${outputPath}`);

  return {
    categories,
    metadata: {
      name: path.basename(config.tokensPath),
      generatedAt: new Date().toISOString(),
    },
  };
}

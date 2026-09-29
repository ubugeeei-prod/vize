import fs from "node:fs";
import path from "node:path";

import type { TokenCategory, TokenValue } from "./tokens.js";

const CSS_EXTENSIONS = new Set([".css", ".pcss", ".postcss"]);
const VARIABLE_DECLARATION_RE = /(--[A-Za-z0-9_-]+)\s*:\s*([^;{}]+);/g;
const CSS_COMMENT_RE = /\/\*[\s\S]*?\*\//g;
const VARIABLE_REFERENCE_RE = /^var\(\s*(--[A-Za-z0-9_-]+)\s*(?:,[^)]+)?\)$/;

const NAMESPACE_MAPPINGS: Array<{
  prefix: string;
  category: string;
  pathPrefix?: string[];
  type: string;
}> = [
  { prefix: "color", category: "color", type: "color" },
  { prefix: "spacing", category: "spacing", type: "dimension" },
  { prefix: "font-weight", category: "typography", pathPrefix: ["fontWeight"], type: "fontWeight" },
  { prefix: "font", category: "typography", pathPrefix: ["font"], type: "fontFamily" },
  { prefix: "text", category: "typography", pathPrefix: ["fontSize"], type: "dimension" },
  { prefix: "radius", category: "radius", type: "dimension" },
  { prefix: "shadow", category: "shadow", type: "shadow" },
];

export async function isCssTokenPath(tokensPath: string): Promise<boolean> {
  const stat = await fs.promises.stat(tokensPath).catch(() => null);
  if (!stat) return false;
  if (stat.isFile()) return CSS_EXTENSIONS.has(path.extname(tokensPath).toLowerCase());
  if (!stat.isDirectory()) return false;
  const entries = await fs.promises.readdir(tokensPath, { withFileTypes: true });
  return entries.some(
    (entry) => entry.isFile() && CSS_EXTENSIONS.has(path.extname(entry.name).toLowerCase()),
  );
}

export async function parseCssTokenFiles(tokensPath: string): Promise<TokenCategory[]> {
  const stat = await fs.promises.stat(tokensPath);
  const files = stat.isDirectory()
    ? (await fs.promises.readdir(tokensPath))
        .filter((name) => CSS_EXTENSIONS.has(path.extname(name).toLowerCase()))
        .sort()
        .map((name) => path.join(tokensPath, name))
    : [tokensPath];
  const css = (await Promise.all(files.map((file) => fs.promises.readFile(file, "utf8")))).join(
    "\n",
  );
  return parseCssTokenSource(css);
}

export function parseCssTokenSource(css: string): TokenCategory[] {
  const source = css.replace(CSS_COMMENT_RE, "");
  const declarations: Array<{ variable: string; value: string }> = [];
  let match: RegExpExecArray | null;
  VARIABLE_DECLARATION_RE.lastIndex = 0;
  while ((match = VARIABLE_DECLARATION_RE.exec(source)) !== null) {
    const value = match[2]?.trim();
    if (match[1] && value) declarations.push({ variable: match[1], value });
  }

  const paths = new Map<string, string>();
  const mapped = declarations.map((declaration) => {
    const token = mapDeclaration(declaration.variable, declaration.value);
    paths.set(declaration.variable, [token.category, ...token.path].join("."));
    return { ...declaration, ...token };
  });

  const categories = new Map<string, TokenCategory>();
  for (const declaration of mapped) {
    const category = categories.get(declaration.category) ?? {
      name: declaration.category,
      tokens: Object.create(null) as Record<string, TokenValue>,
    };
    categories.set(declaration.category, category);
    const referenceVariable = declaration.value.match(VARIABLE_REFERENCE_RE)?.[1];
    const reference = referenceVariable ? paths.get(referenceVariable) : undefined;
    const token: TokenValue = {
      value: reference ? `{${reference}}` : declaration.value,
      type: declaration.type,
      description: declaration.description,
      $tier: reference ? "semantic" : "primitive",
      ...(reference ? { $reference: reference } : {}),
    };
    setToken(category, declaration.path, token);
  }
  return [...categories.values()];
}

function mapDeclaration(
  variable: string,
  value: string,
): { category: string; path: string[]; type: string; description: string } {
  const name = variable.slice(2);
  const mapping = NAMESPACE_MAPPINGS.find(
    (candidate) => name === candidate.prefix || name.startsWith(`${candidate.prefix}-`),
  );
  if (mapping) {
    const rest = name === mapping.prefix ? "DEFAULT" : name.slice(mapping.prefix.length + 1);
    const pathParts = [...(mapping.pathPrefix ?? []), rest].filter(Boolean);
    return {
      category: mapping.category,
      path: pathParts.length > 0 ? pathParts : ["DEFAULT"],
      type: mapping.type,
      description: `Tailwind theme variable ${variable}`,
    };
  }
  const segments = name.split("-").filter(Boolean);
  const category = segments[0] ?? "token";
  const rest = segments.slice(1).join("-");
  return {
    category,
    path: rest ? [rest] : ["DEFAULT"],
    type: inferType(value),
    description: `Custom property ${variable}`,
  };
}

function inferType(value: string): string {
  if (/^(?:#|rgba?\(|hsla?\(|oklch\(|oklab\(|color-mix\(|color\()/i.test(value)) return "color";
  if (/^-?(?:\d+\.\d+|\d+)(?:px|rem|em|vh|vw|vmin|vmax|%)$/i.test(value)) return "dimension";
  if (/^-?(?:\d+\.\d+|\d+)$/.test(value)) return "number";
  return "other";
}

function setToken(category: TokenCategory, pathParts: string[], token: TokenValue): void {
  if (pathParts.length === 1) {
    category.tokens[pathParts[0]!] = token;
    return;
  }
  let current = category;
  for (const part of pathParts.slice(0, -1)) {
    current.subcategories ??= [];
    let next = current.subcategories.find((subcategory) => subcategory.name === part);
    if (!next) {
      next = { name: part, tokens: Object.create(null) as Record<string, TokenValue> };
      current.subcategories.push(next);
    }
    current = next;
  }
  current.tokens[pathParts[pathParts.length - 1]!] = token;
}

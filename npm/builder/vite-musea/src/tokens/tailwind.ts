import fs from "node:fs";
import path from "node:path";

import { nullRecord, normalizeCategories } from "./native.js";
import type { DesignToken, TokenCategory } from "./parser.js";

interface TailwindTokenCandidate {
  variable: string;
  value: string;
  category: string;
  path: string[];
  type: string;
}

const TAILWIND_TOKEN_EXTENSIONS = new Set([".css", ".pcss", ".postcss"]);
const VARIABLE_DECLARATION_RE = /(--[A-Za-z0-9_-]+)\s*:\s*([^;{}]+);/g;
const CSS_COMMENT_RE = /\/\*[\s\S]*?\*\//g;

const NAMESPACE_MAPPINGS: Array<{
  prefix: string;
  category: string;
  pathPrefix?: string[];
  type: string;
}> = [
  { prefix: "inset-shadow", category: "shadow", pathPrefix: ["inset"], type: "shadow" },
  { prefix: "drop-shadow", category: "shadow", pathPrefix: ["drop"], type: "shadow" },
  { prefix: "font-weight", category: "typography", pathPrefix: ["fontWeight"], type: "fontWeight" },
  { prefix: "color", category: "color", type: "color" },
  { prefix: "spacing", category: "spacing", type: "dimension" },
  { prefix: "font", category: "typography", pathPrefix: ["font"], type: "fontFamily" },
  { prefix: "text", category: "typography", pathPrefix: ["fontSize"], type: "dimension" },
  { prefix: "leading", category: "typography", pathPrefix: ["lineHeight"], type: "number" },
  {
    prefix: "tracking",
    category: "typography",
    pathPrefix: ["letterSpacing"],
    type: "dimension",
  },
  { prefix: "radius", category: "radius", type: "dimension" },
  { prefix: "shadow", category: "shadow", type: "shadow" },
  { prefix: "breakpoint", category: "breakpoint", type: "dimension" },
  { prefix: "container", category: "container", type: "dimension" },
  { prefix: "ease", category: "easing", type: "cubicBezier" },
  { prefix: "animate", category: "animation", type: "transition" },
  { prefix: "blur", category: "blur", type: "dimension" },
  { prefix: "perspective", category: "perspective", type: "dimension" },
  { prefix: "aspect", category: "aspectRatio", type: "number" },
];

export async function isTailwindTokenPath(tokensPath: string): Promise<boolean> {
  const stat = await fs.promises.stat(tokensPath).catch(() => null);
  if (!stat) return false;
  if (stat.isFile()) {
    return TAILWIND_TOKEN_EXTENSIONS.has(path.extname(tokensPath));
  }
  if (!stat.isDirectory()) {
    return false;
  }
  const entries = await fs.promises.readdir(tokensPath, { withFileTypes: true });
  return entries.some(
    (entry) => entry.isFile() && TAILWIND_TOKEN_EXTENSIONS.has(path.extname(entry.name)),
  );
}

export async function parseTailwindTokens(tokensPath: string): Promise<TokenCategory[]> {
  const files = await collectTailwindTokenFiles(tokensPath);
  const candidates: TailwindTokenCandidate[] = [];

  for (const file of files) {
    const css = await fs.promises.readFile(file, "utf-8");
    candidates.push(...extractTailwindTokenCandidates(css));
  }

  return normalizeCategories(buildCategories(candidates));
}

async function collectTailwindTokenFiles(tokensPath: string): Promise<string[]> {
  const stat = await fs.promises.stat(tokensPath);
  if (stat.isFile()) {
    return [tokensPath];
  }

  const entries = await fs.promises.readdir(tokensPath, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile() && TAILWIND_TOKEN_EXTENSIONS.has(path.extname(entry.name)))
    .map((entry) => path.join(tokensPath, entry.name))
    .sort();
}

function extractTailwindTokenCandidates(css: string): TailwindTokenCandidate[] {
  const source = css.replace(CSS_COMMENT_RE, "");
  const candidates: TailwindTokenCandidate[] = [];
  let match: RegExpExecArray | null;
  VARIABLE_DECLARATION_RE.lastIndex = 0;

  while ((match = VARIABLE_DECLARATION_RE.exec(source)) !== null) {
    const variable = match[1];
    const value = match[2].trim();
    const mapped = mapTailwindVariable(variable);
    if (!mapped || !value) {
      continue;
    }
    candidates.push({ variable, value, ...mapped });
  }

  return candidates;
}

function mapTailwindVariable(
  variable: string,
): Omit<TailwindTokenCandidate, "variable" | "value"> | null {
  const name = variable.replace(/^--/, "");
  const mapping = NAMESPACE_MAPPINGS.find(
    (candidate) => name === candidate.prefix || name.startsWith(`${candidate.prefix}-`),
  );
  if (!mapping) {
    return null;
  }

  const rest = name === mapping.prefix ? "DEFAULT" : name.slice(mapping.prefix.length + 1);
  const pathParts = [...(mapping.pathPrefix ?? []), ...tailwindNameToPath(rest)];
  if (pathParts.length === 0) {
    return null;
  }

  return {
    category: mapping.category,
    path: pathParts,
    type: mapping.type,
  };
}

function tailwindNameToPath(name: string): string[] {
  if (!name || name === "DEFAULT") {
    return ["DEFAULT"];
  }

  const parts = name.split("-").filter(Boolean);
  if (parts.length <= 1) {
    return parts;
  }

  const last = parts[parts.length - 1];
  if (/^\d+$/.test(last)) {
    return [...parts.slice(0, -1), last];
  }

  return [parts.join("-")];
}

function buildCategories(candidates: TailwindTokenCandidate[]): TokenCategory[] {
  const variableToPath = new Map<string, string>();
  for (const candidate of candidates) {
    variableToPath.set(candidate.variable, [candidate.category, ...candidate.path].join("."));
  }

  const categoryMap = new Map<string, TokenCategory>();
  for (const candidate of candidates) {
    const category = categoryMap.get(candidate.category) ?? {
      name: candidate.category,
      tokens: nullRecord<DesignToken>({}),
    };
    categoryMap.set(candidate.category, category);

    const valueReference = parseTailwindVariableReference(candidate.value);
    const referencedPath = valueReference ? variableToPath.get(valueReference) : undefined;
    const token: DesignToken = {
      value: referencedPath ? `{${referencedPath}}` : candidate.value,
      type: candidate.type,
      description: `Tailwind theme variable ${candidate.variable}`,
      attributes: {
        tailwindVariable: candidate.variable,
      },
      $tier: referencedPath ? "semantic" : "primitive",
      ...(referencedPath ? { $reference: referencedPath } : {}),
    };

    setCategoryToken(category, candidate.path, token);
  }

  return [...categoryMap.values()];
}

function parseTailwindVariableReference(value: string): string | undefined {
  const match = value.match(/^var\(\s*(--[A-Za-z0-9_-]+)\s*(?:,[^)]+)?\)$/);
  return match?.[1];
}

function setCategoryToken(category: TokenCategory, pathParts: string[], token: DesignToken): void {
  if (pathParts.length === 1) {
    category.tokens[pathParts[0]] = token;
    return;
  }

  let current = category;
  for (const part of pathParts.slice(0, -1)) {
    current.subcategories ??= [];
    let next = current.subcategories.find((subcategory) => subcategory.name === part);
    if (!next) {
      next = { name: part, tokens: nullRecord<DesignToken>({}) };
      current.subcategories.push(next);
    }
    current = next;
  }

  current.tokens[pathParts[pathParts.length - 1]] = token;
}

import fs from "node:fs";
import path from "node:path";

export interface CssAliasRule {
  find: string;
  replacement: string;
}

const deepSelectorPattern = /:deep\(([^()]*(?:\([^()]*\))*[^()]*)\)/;
const globalSelectorPattern = /:global\(([^()]*(?:\([^()]*\))*[^()]*)\)/g;
const recursiveAtRules = new Set(["@container", "@layer", "@media", "@supports"]);

export function scopeCssForPipeline(css: string, scopeId: string): string {
  return transformCssBlock(css, scopeId);
}

/**
 * Resolve CSS @import statements by inlining the imported files,
 * then resolve @custom-media definitions within the combined CSS.
 *
 * This is necessary because Vize embeds CSS as a JS string via
 * document.createElement('style'), bypassing Vite's CSS pipeline.
 */
export function resolveCssImports(
  css: string,
  importer: string,
  aliasRules: CssAliasRule[],
  isDev?: boolean,
  devUrlBase?: string,
): string {
  // Collect @custom-media definitions and imported content
  const customMedia = new Map<string, string>();
  const importRegex = /^@import\s+(?:"([^"]+)"|'([^']+)');?\s*$/gm;
  let result = css;

  // Phase 1: Resolve @import — inline imported file contents
  result = result.replace(importRegex, (_match, dqPath?: string, sqPath?: string) => {
    const importPath = dqPath || sqPath;
    if (!importPath) return _match;

    const resolved = resolveCssPath(importPath, importer, aliasRules);
    if (!resolved || !fs.existsSync(resolved)) {
      return _match; // Keep unresolved imports as-is
    }

    try {
      const content = fs.readFileSync(resolved, "utf-8");
      // Parse @custom-media from imported file
      parseCustomMedia(content, customMedia);
      return content;
    } catch {
      return _match;
    }
  });

  // Also parse @custom-media from the main CSS itself
  parseCustomMedia(result, customMedia);

  // Phase 2: Remove @custom-media definitions from output
  result = result.replace(/^@custom-media\s+[^;]+;\s*$/gm, "");

  // Phase 3: Replace @media (--name) with resolved values
  if (customMedia.size > 0) {
    for (const [name, query] of customMedia) {
      // Replace (--name) in @media rules
      const escaped = name.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&");
      result = result.replace(new RegExp(`\\(${escaped}\\)`, "g"), query);
    }
  }

  // Phase 4: Resolve url() references with alias prefixes
  if (isDev) {
    result = result.replace(/url\(\s*(["']?)([^"')]+)\1\s*\)/g, (_match, quote, urlPath) => {
      const trimmed = urlPath.trim();
      // Skip data: URLs, absolute http(s) URLs, and already-resolved paths
      if (
        trimmed.startsWith("data:") ||
        trimmed.startsWith("http://") ||
        trimmed.startsWith("https://") ||
        trimmed.startsWith("/@fs/")
      ) {
        return _match;
      }
      const resolved = resolveCssPath(trimmed, importer, aliasRules);
      if (resolved && fs.existsSync(resolved)) {
        const normalized = resolved.replace(/\\/g, "/");
        // In Nuxt, Vite is mounted under a base path (e.g., /_nuxt/),
        // so /@fs/ URLs must be prefixed with the base to reach Vite's middleware.
        const base = devUrlBase ?? "/";
        const prefix = base.endsWith("/") ? base : base + "/";
        return `url("${prefix}@fs${normalized}")`;
      }
      return _match;
    });
  }

  // Phase 5: Unwrap Vue scoped CSS pseudo-selectors (:deep, :slotted, :global)
  // Vize uses native CSS nesting with scope attribute only on the root element,
  // so :deep(X) is simply X (no scope attribute to remove from child selectors).
  result = result.replace(new RegExp(deepSelectorPattern.source, "g"), "$1");

  // Clean up excessive blank lines
  result = result.replace(/\n{3,}/g, "\n\n");

  return result;
}

function transformCssBlock(css: string, scopeId: string): string {
  let output = "";
  let cursor = 0;

  while (cursor < css.length) {
    const brace = findNextTopLevelBrace(css, cursor);
    if (brace === -1) {
      output += css.slice(cursor);
      break;
    }

    const end = findMatchingBrace(css, brace);
    if (end === -1) {
      output += css.slice(cursor);
      break;
    }

    const header = css.slice(cursor, brace);
    const body = css.slice(brace + 1, end);
    const leadingLength = header.search(/\S/);
    const leading = leadingLength === -1 ? header : header.slice(0, leadingLength);
    const statement = leadingLength === -1 ? "" : header.slice(leadingLength);

    output += leading;
    if (statement.trimStart().startsWith("@")) {
      output += statement;
      output += "{";
      output += shouldRecurseAtRule(statement) ? transformCssBlock(body, scopeId) : body;
      output += "}";
    } else {
      output += scopeSelectorList(statement, scopeId);
      output += "{";
      output += body;
      output += "}";
    }

    cursor = end + 1;
  }

  return output;
}

function shouldRecurseAtRule(statement: string): boolean {
  const name = statement.trimStart().split(/\s+/, 1)[0];
  return name !== undefined && recursiveAtRules.has(name);
}

function findNextTopLevelBrace(css: string, start: number): number {
  let parenDepth = 0;
  let bracketDepth = 0;
  let quote: string | null = null;
  let inComment = false;

  for (let index = start; index < css.length; index += 1) {
    const char = css[index]!;
    const next = css[index + 1];

    if (inComment) {
      if (char === "*" && next === "/") {
        inComment = false;
        index += 1;
      }
      continue;
    }

    if (quote !== null) {
      if (char === "\\") {
        index += 1;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === "/" && next === "*") {
      inComment = true;
      index += 1;
      continue;
    }

    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }

    if (char === "(") parenDepth += 1;
    else if (char === ")" && parenDepth > 0) parenDepth -= 1;
    else if (char === "[") bracketDepth += 1;
    else if (char === "]" && bracketDepth > 0) bracketDepth -= 1;
    else if (char === "{" && parenDepth === 0 && bracketDepth === 0) return index;
  }

  return -1;
}

function findMatchingBrace(css: string, start: number): number {
  let depth = 0;
  let quote: string | null = null;
  let inComment = false;

  for (let index = start; index < css.length; index += 1) {
    const char = css[index]!;
    const next = css[index + 1];

    if (inComment) {
      if (char === "*" && next === "/") {
        inComment = false;
        index += 1;
      }
      continue;
    }

    if (quote !== null) {
      if (char === "\\") {
        index += 1;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === "/" && next === "*") {
      inComment = true;
      index += 1;
      continue;
    }

    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }

    if (char === "{") depth += 1;
    else if (char === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }

  return -1;
}

function scopeSelectorList(selectorList: string, scopeId: string): string {
  return splitSelectorList(selectorList)
    .map((selector) => scopeSelector(selector, scopeId))
    .join(",");
}

function splitSelectorList(selectorList: string): string[] {
  const selectors: string[] = [];
  let start = 0;
  let parenDepth = 0;
  let bracketDepth = 0;
  let quote: string | null = null;

  for (let index = 0; index < selectorList.length; index += 1) {
    const char = selectorList[index]!;
    if (quote !== null) {
      if (char === "\\") {
        index += 1;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }
    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }
    if (char === "(") parenDepth += 1;
    else if (char === ")" && parenDepth > 0) parenDepth -= 1;
    else if (char === "[") bracketDepth += 1;
    else if (char === "]" && bracketDepth > 0) bracketDepth -= 1;
    else if (char === "," && parenDepth === 0 && bracketDepth === 0) {
      selectors.push(selectorList.slice(start, index));
      start = index + 1;
    }
  }

  selectors.push(selectorList.slice(start));
  return selectors;
}

function scopeSelector(selector: string, scopeId: string): string {
  const leadingLength = selector.search(/\S/);
  if (leadingLength === -1) return selector;

  const leading = selector.slice(0, leadingLength);
  const trailingLength = selector.match(/\s*$/)?.[0].length ?? 0;
  const bodyEnd = trailingLength === 0 ? selector.length : selector.length - trailingLength;
  const trailing = selector.slice(bodyEnd);
  let body = selector.slice(leadingLength, bodyEnd).replace(globalSelectorPattern, "$1");

  const deep = body.match(deepSelectorPattern);
  if (deep?.index !== undefined) {
    const before = body.slice(0, deep.index).trimEnd();
    const inner = deep[1] ?? "";
    const after = body.slice(deep.index + deep[0].length);
    const scopedBefore =
      before.length === 0 ? `[${scopeId}]` : addScopeToSelectorEnd(before, scopeId);
    body = `${scopedBefore} ${inner}${after}`;
  } else {
    body = addScopeToSelectorEnd(body, scopeId);
  }

  return leading + body + trailing;
}

function addScopeToSelectorEnd(selector: string, scopeId: string): string {
  const targetStart = findLastCompoundStart(selector);
  const beforeTarget = selector.slice(0, targetStart);
  const target = selector.slice(targetStart);
  const insertAt = findScopeInsertPosition(target);
  return `${beforeTarget}${target.slice(0, insertAt)}[${scopeId}]${target.slice(insertAt)}`;
}

function findLastCompoundStart(selector: string): number {
  let parenDepth = 0;
  let bracketDepth = 0;
  let quote: string | null = null;

  for (let index = selector.length - 1; index >= 0; index -= 1) {
    const char = selector[index]!;
    if (quote !== null) {
      if (char === quote) quote = null;
      continue;
    }
    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }
    if (char === ")") parenDepth += 1;
    else if (char === "(" && parenDepth > 0) parenDepth -= 1;
    else if (char === "]") bracketDepth += 1;
    else if (char === "[" && bracketDepth > 0) bracketDepth -= 1;
    else if (
      parenDepth === 0 &&
      bracketDepth === 0 &&
      (char === ">" || char === "+" || char === "~")
    ) {
      return index + 1;
    } else if (parenDepth === 0 && bracketDepth === 0 && /\s/.test(char)) {
      while (index > 0 && /\s/.test(selector[index - 1]!)) index -= 1;
      return index + 1;
    }
  }

  return 0;
}

function findScopeInsertPosition(target: string): number {
  let parenDepth = 0;
  let bracketDepth = 0;
  let quote: string | null = null;

  for (let index = 0; index < target.length; index += 1) {
    const char = target[index]!;
    if (quote !== null) {
      if (char === "\\") {
        index += 1;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }
    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }
    if (char === "(") parenDepth += 1;
    else if (char === ")" && parenDepth > 0) parenDepth -= 1;
    else if (char === "[") bracketDepth += 1;
    else if (char === "]" && bracketDepth > 0) bracketDepth -= 1;
    else if (char === ":" && parenDepth === 0 && bracketDepth === 0) {
      return index;
    }
  }

  return target.length;
}

function parseCustomMedia(css: string, map: Map<string, string>): void {
  const re = /@custom-media\s+(--[\w-]+)\s+(.+?)\s*;/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(css)) !== null) {
    map.set(m[1], m[2]);
  }
}

function resolveCssPath(
  importPath: string,
  importer: string,
  aliasRules: CssAliasRule[],
): string | null {
  // Try alias resolution
  for (const rule of aliasRules) {
    if (importPath.startsWith(rule.find)) {
      const resolved = importPath.replace(rule.find, rule.replacement);
      return path.resolve(resolved);
    }
  }

  // Relative path
  if (importPath.startsWith(".")) {
    const dir = path.dirname(importer);
    return path.resolve(dir, importPath);
  }

  // Absolute path
  if (path.isAbsolute(importPath)) {
    return importPath;
  }

  return null;
}

/** Shared utilities for @vizejs/rspack-plugin. */

import { createHash } from "node:crypto";
import path from "node:path";
import type {
  StyleBlockInfo,
  CustomBlockInfo,
  SfcSrcInfo,
  TemplateAssetUrl,
} from "../types/index.js";

/** Generate scope ID (8-char SHA256 prefix). Uses relative path for cross-env consistency. */
export function generateScopeId(
  filename: string,
  rootContext?: string,
  isProduction?: boolean,
  source?: string,
): string {
  let input: string;
  if (rootContext) {
    const relative = path
      .relative(rootContext, filename)
      .replace(/^(\.\.[/\\])+/, "")
      .replace(/\\/g, "/");
    input = isProduction && source ? relative + "\n" + source.replace(/\r\n/g, "\n") : relative;
  } else {
    input = filename;
  }
  const hash = createHash("sha256").update(input).digest("hex");
  return hash.slice(0, 8);
}

/** Extract style block metadata from SFC source. */
export function extractStyleBlocks(source: string): StyleBlockInfo[] {
  const blocks: StyleBlockInfo[] = [];
  const styleRegex = /<style([^>]*)>([\s\S]*?)<\/style>/gi;
  let match;
  let index = 0;

  while ((match = styleRegex.exec(source)) !== null) {
    const attrs = match[1];
    const content = match[2];
    const src = attrs.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null;
    const lang = attrs.match(/\blang=["']([^"']+)["']/)?.[1] ?? null;
    const scoped = /\bscoped\b/.test(attrs);
    const moduleMatch = attrs.match(/\bmodule(?:=["']([^"']+)["'])?/);
    const isModule = moduleMatch ? moduleMatch[1] || true : false;

    blocks.push({ content, src, lang, scoped, module: isModule, index });
    index++;
  }

  return blocks;
}

/** Fallback scoped CSS transformer using regex. Does not support @media nesting, :deep()/:global()/:slotted(). */
export function addScopeToCssFallback(css: string, scopeId: string): string {
  const scopeAttr = `[data-v-${scopeId}]`;

  // Block-aware: only transform selectors (before {), not property values.
  let result = "";
  let depth = 0;
  let i = 0;
  let selectorStart = 0;

  while (i < css.length) {
    const ch = css[i];

    if (ch === "{") {
      if (depth === 0) {
        const selectorGroup = css.slice(selectorStart, i);
        result += scopeSelectors(selectorGroup, scopeAttr);
        result += "{";
      } else {
        result += ch;
      }
      depth++;
      i++;
      selectorStart = i;
    } else if (ch === "}") {
      depth--;
      result += ch;
      i++;
      selectorStart = i;
    } else {
      if (depth > 0) {
        result += ch;
      }
      i++;
    }
  }

  // Remaining text after the last }
  if (selectorStart < css.length && depth === 0) {
    result += css.slice(selectorStart);
  }

  return result;
}

/** Strip CSS block comments while preserving string literals and line positions. */
export function stripCssCommentsForScoped(css: string): string {
  // Fast path: no block comment
  if (!css.includes("/*")) return css;

  const len = css.length;
  const parts: string[] = [];
  let copyStart = 0; // start of the current non-comment segment
  let i = 0;

  while (i < len) {
    const code = css.charCodeAt(i);

    if (code === 34 /* " */ || code === 39 /* ' */) {
      const quote = code;
      i++;
      while (i < len) {
        const c = css.charCodeAt(i);
        if (c === 92 /* \\ */) {
          i += 2;
          continue;
        }
        i++;
        if (c === quote) break;
      }
      continue;
    }

    if (code === 47 /* / */ && i + 1 < len && css.charCodeAt(i + 1) === 42 /* * */) {
      if (i > copyStart) parts.push(css.slice(copyStart, i));
      i += 2; // skip `/*`
      let commentBuf = "  "; // replacement for `/*`
      while (i < len) {
        if (
          css.charCodeAt(i) === 42 /* * */ &&
          i + 1 < len &&
          css.charCodeAt(i + 1) === 47 /* / */
        ) {
          commentBuf += "  "; // replacement for `*/`
          i += 2;
          break;
        }
        commentBuf += css.charCodeAt(i) === 10 /* \n */ ? "\n" : " ";
        i++;
      }
      parts.push(commentBuf);
      copyStart = i;
      continue;
    }

    i++;
  }

  // If no comment was replaced, return original.
  if (copyStart === 0) return css;
  if (copyStart < len) parts.push(css.slice(copyStart));

  return parts.join("");
}

/** Scope selectors in a comma-separated group. Skips at-rules. */
function scopeSelectors(group: string, scopeAttr: string): string {
  const trimmed = group.trim();
  // At-rules pass through
  if (trimmed.startsWith("@")) {
    return group;
  }
  return group
    .split(",")
    .map((sel) => {
      const s = sel.trim();
      if (!s || s.startsWith("@")) return sel;
      // Preserve leading whitespace/newlines from original
      const leadingWs = sel.match(/^(\s*)/)?.[1] ?? "";
      return `${leadingWs}${s}${scopeAttr}`;
    })
    .join(",");
}

/** Extract custom block metadata from SFC source (non-script/template/style tags). */
export function extractCustomBlocks(source: string): CustomBlockInfo[] {
  const blocks: CustomBlockInfo[] = [];
  const knownTopLevel = new Set(["script", "template", "style"]);
  let blockIndex = 0;
  let pos = 0;

  while (pos < source.length) {
    const ltPos = source.indexOf("<", pos);
    if (ltPos === -1) break;

    // Skip HTML comments
    if (source.startsWith("<!--", ltPos)) {
      const endComment = source.indexOf("-->", ltPos + 4);
      pos = endComment === -1 ? source.length : endComment + 3;
      continue;
    }

    // Skip closing tags at the top level
    if (source[ltPos + 1] === "/") {
      const gt = source.indexOf(">", ltPos + 2);
      pos = gt === -1 ? source.length : gt + 1;
      continue;
    }

    const openTagMatch = /^<([a-zA-Z][a-zA-Z0-9-]*)(\s[^>]*)?\s*\/?>/.exec(source.slice(ltPos));
    if (!openTagMatch) {
      pos = ltPos + 1;
      continue;
    }

    const tagName = openTagMatch[1];
    const attrsStr = (openTagMatch[2] || "").trim();
    const selfClosing = openTagMatch[0].trimEnd().endsWith("/>");
    const afterOpenTag = ltPos + openTagMatch[0].length;

    if (selfClosing) {
      if (!knownTopLevel.has(tagName.toLowerCase())) {
        blocks.push({
          type: tagName,
          content: "",
          src: attrsStr.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null,
          attrs: parseAttributes(attrsStr),
          index: blockIndex++,
        });
      }
      pos = afterOpenTag;
      continue;
    }

    // Find the matching closing tag while tracking same-name nesting depth
    const tagNameLower = tagName.toLowerCase();
    let depth = 1;
    let scanPos = afterOpenTag;

    while (scanPos < source.length && depth > 0) {
      const nextLt = source.indexOf("<", scanPos);
      if (nextLt === -1) {
        scanPos = source.length;
        break;
      }

      const closeRe = new RegExp(`^</${tagName}\\s*>`, "i");
      const closeMatch = closeRe.exec(source.slice(nextLt));
      if (closeMatch) {
        depth--;
        if (depth === 0) {
          const content = source.slice(afterOpenTag, nextLt);
          if (!knownTopLevel.has(tagNameLower)) {
            blocks.push({
              type: tagName,
              content,
              src: attrsStr.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null,
              attrs: parseAttributes(attrsStr),
              index: blockIndex++,
            });
          }
          scanPos = nextLt + closeMatch[0].length;
          break;
        }
        scanPos = nextLt + closeMatch[0].length;
        continue;
      }

      const openRe = new RegExp(`^<${tagName}\\b[^>]*>`, "i");
      const openMatch = openRe.exec(source.slice(nextLt));
      if (openMatch && !openMatch[0].trimEnd().endsWith("/>")) {
        depth++;
        scanPos = nextLt + openMatch[0].length;
        continue;
      }

      scanPos = nextLt + 1;
    }

    pos = scanPos;
  }

  return blocks;
}

/** Parse HTML-like attributes from a tag's attribute string. */
function parseAttributes(attrsStr: string): Record<string, string | true> {
  const attrs: Record<string, string | true> = {};
  const attrRegex = /\b([a-z][a-z0-9-]*)(?:=["']([^"']*)["'])?/gi;
  let match;

  while ((match = attrRegex.exec(attrsStr)) !== null) {
    attrs[match[1]] = match[2] ?? true;
  }

  return attrs;
}

/** Extract <script src> and <template src> references from SFC source. */
export function extractSrcInfo(source: string): SfcSrcInfo {
  const scriptMatch = source.match(/<script([^>]*)>/i);
  const templateMatch = source.match(/<template([^>]*)>/i);

  const scriptSrc = scriptMatch?.[1]?.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null;
  const templateSrc = templateMatch?.[1]?.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null;

  return { scriptSrc, templateSrc };
}

/** Replace <script src> or <template src> with inline content from external files. */
export function inlineSrcBlocks(
  source: string,
  scriptContent: string | null,
  templateContent: string | null,
): string {
  let result = source;

  if (scriptContent !== null) {
    // Replace <script src> with inline content
    result = result.replace(
      /(<script)([^>]*)\bsrc=["'][^"']+["']([^>]*>)[\s\S]*?(<\/script>)/i,
      (_, open, beforeSrc, afterSrc, close) => {
        // Remove src attribute remnants and rebuild
        const attrs = (beforeSrc + afterSrc).replace(/\bsrc=["'][^"']+["']\s*/g, "");
        return `${open}${attrs}\n${scriptContent}\n${close}`;
      },
    );
  }

  if (templateContent !== null) {
    // Replace <template src> with inline content
    result = result.replace(
      /(<template)([^>]*)\bsrc=["'][^"']+["']([^>]*>)[\s\S]*?(<\/template>)/i,
      (_, open, beforeSrc, afterSrc, close) => {
        const attrs = (beforeSrc + afterSrc).replace(/\bsrc=["'][^"']+["']\s*/g, "");
        return `${open}${attrs}\n${templateContent}\n${close}`;
      },
    );
  }

  return result;
}

/** Match a file path against include/exclude patterns. Normalizes backslashes. */
export function matchesPattern(
  file: string,
  pattern: string | RegExp | (string | RegExp)[] | undefined,
  defaultValue: boolean,
): boolean {
  if (!pattern) {
    return defaultValue;
  }

  // Normalize Windows backslashes
  const normalizedFile = file.replace(/\\/g, "/");

  const patterns = Array.isArray(pattern) ? pattern : [pattern];
  return patterns.some((item) => {
    if (typeof item === "string") {
      return normalizedFile.includes(item) || file.includes(item);
    }
    return item.test(normalizedFile);
  });
}

// Template Asset URL Transformation

/** Default element→attribute mapping for transformAssetUrls. */
export const DEFAULT_ASSET_URL_TAGS: Readonly<Record<string, string[]>> = Object.freeze({
  img: ["src"],
  video: ["src", "poster"],
  source: ["src"],
  image: ["xlink:href", "href"],
  use: ["xlink:href", "href"],
});

/** Returns true when a URL should be rewritten as an import (relative, alias, tilde). */
export function isImportableUrl(url: string): boolean {
  if (!url) return false;
  // External / protocol-relative / data URIs
  if (/^(https?:)?\/\//.test(url) || url.startsWith("data:")) return false;
  // Relative paths
  if (url.startsWith("./") || url.startsWith("../")) return true;
  // Alias and tilde module paths
  if (url.startsWith("@/") || url.startsWith("~")) return true;
  return false;
}

/** Extract top-level `<template>` content, tracking depth for nested templates. */
function extractSfcTemplateContent(source: string): string | null {
  let pos = 0;

  while (pos < source.length) {
    const ltPos = source.indexOf("<", pos);
    if (ltPos === -1) break;

    // Skip HTML comments
    if (source.startsWith("<!--", ltPos)) {
      const end = source.indexOf("-->", ltPos + 4);
      pos = end === -1 ? source.length : end + 3;
      continue;
    }

    // Skip closing tags
    if (source[ltPos + 1] === "/") {
      const gt = source.indexOf(">", ltPos + 2);
      pos = gt === -1 ? source.length : gt + 1;
      continue;
    }

    const openMatch = /^<template(\s[^>]*)?>/.exec(source.slice(ltPos));
    if (!openMatch) {
      pos = ltPos + 1;
      continue;
    }

    // Found the opening <template> tag — track depth to find its closing tag
    const afterOpen = ltPos + openMatch[0].length;
    let depth = 1;
    let scan = afterOpen;

    while (scan < source.length && depth > 0) {
      const next = source.indexOf("<", scan);
      if (next === -1) break;

      const closeMatch = /^<\/template\s*>/i.exec(source.slice(next));
      if (closeMatch) {
        depth--;
        if (depth === 0) return source.slice(afterOpen, next);
        scan = next + closeMatch[0].length;
        continue;
      }

      const innerOpenMatch = /^<template\b[^>]*>/i.exec(source.slice(next));
      if (innerOpenMatch && !innerOpenMatch[0].trimEnd().endsWith("/>")) {
        depth++;
        scan = next + innerOpenMatch[0].length;
        continue;
      }

      scan = next + 1;
    }

    return null; // malformed/unclosed template
  }

  return null; // no <template>
}

/** Escape a string for RegExp. */
function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

/** Scan SFC template for static asset URLs that should become import bindings. Deduplicated. */
export function collectTemplateAssetUrls(
  source: string,
  tags?: boolean | Record<string, string[]>,
): TemplateAssetUrl[] {
  if (tags === false) return [];

  const tagConfig: Record<string, string[]> =
    tags == null || tags === true ? (DEFAULT_ASSET_URL_TAGS as Record<string, string[]>) : tags;

  const templateContent = extractSfcTemplateContent(source);
  if (!templateContent) return [];

  const urlToVar = new Map<string, string>();
  let counter = 0;

  for (const [tag, attrs] of Object.entries(tagConfig)) {
    // Match an opening tag (including self-closing).
    // Attribute values may contain spaces, so we consume everything up to `>`,
    // skipping over quoted attribute values that might include `>`.
    const tagRe = new RegExp(`<${escapeRegex(tag)}((?:\\s+[^>]*)?)(?:/>|>)`, "gi");
    let tagMatch: RegExpExecArray | null;

    while ((tagMatch = tagRe.exec(templateContent)) !== null) {
      const attrStr = tagMatch[1] ?? "";

      for (const attr of attrs) {
        // Static attribute: must be preceded by whitespace (or start of attrStr),
        // NOT by `:` or `v-bind:` (which mark dynamic bindings).
        const doubleQuoteRe = new RegExp(`(?:^|\\s)${escapeRegex(attr)}="([^"]+)"`, "i");
        const singleQuoteRe = new RegExp(`(?:^|\\s)${escapeRegex(attr)}='([^']+)'`, "i");

        const m = doubleQuoteRe.exec(attrStr) ?? singleQuoteRe.exec(attrStr);
        if (m) {
          const url = m[1];
          if (isImportableUrl(url) && !urlToVar.has(url)) {
            urlToVar.set(url, `_imports_${counter++}`);
          }
        }
      }
    }
  }

  return Array.from(urlToVar.entries()).map(([url, varName]) => ({
    url,
    varName,
  }));
}

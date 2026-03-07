/**
 * Shared utility functions for @vizejs/rspack-plugin
 * Copied from vite-plugin-vize and adapted
 */

import { createHash } from "node:crypto";
import path from "node:path";
import type { StyleBlockInfo, CustomBlockInfo, SfcSrcInfo } from "../types/index.js";

/**
 * Generate a unique scope ID for scoped CSS based on file path.
 * Uses SHA256 hash and takes the first 8 characters.
 *
 * Uses relative path (when rootContext is provided) for cross-environment
 * consistency (e.g. SSR hydration).  In production, mixes source content
 * into the hash to avoid collisions across packages with identically-named files.
 */
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

/**
 * Extract style block metadata from a Vue SFC source string.
 * Parses `<style>` tags to determine lang, scoped, and module attributes.
 *
 * Copied from vite-plugin-vize/src/compiler.ts
 */
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
    const moduleMatch = attrs.match(/\bmodule(?:=["']([^"']+)["'])?\b/);
    const isModule = moduleMatch ? moduleMatch[1] || true : false;

    blocks.push({ content, src, lang, scoped, module: isModule, index });
    index++;
  }

  return blocks;
}

/**
 * Fallback scoped CSS transformer using simple regex.
 *
 * ⚠️ Known limitations:
 *   - Does not support @media / @supports nested selectors
 *   - Does not support Vue's :deep() / :global() / :slotted() pseudo-classes
 *   - Does not handle CSS comments containing { or ,
 *   - Does not support :root and other pseudo-class selectors
 *
 * Recommendation: This regex implementation is for MVP stage only;
 * production-grade scoped semantics should use native-side precise API.
 *
 * Copied from rspack-plugin-design.md
 */
export function addScopeToCssFallback(css: string, scopeId: string): string {
  const scopeAttr = `[data-v-${scopeId}]`;

  // Block-aware approach: only transform selectors (before {), not property values.
  // Walk through the CSS tracking brace depth to distinguish selectors from declarations.
  let result = "";
  let depth = 0;
  let i = 0;
  let selectorStart = 0;

  while (i < css.length) {
    const ch = css[i];

    if (ch === "{") {
      if (depth === 0) {
        // Everything from selectorStart..i is a selector group
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
        // Inside a declaration block — emit as-is
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

/**
 * Add scoped attribute to each selector in a comma-separated selector group.
 * Skips at-rules (@media, @keyframes, etc.).
 */
function scopeSelectors(group: string, scopeAttr: string): string {
  const trimmed = group.trim();
  // At-rules pass through unchanged
  if (trimmed.startsWith("@")) {
    return group;
  }
  // Split by comma, scope each individual selector
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

/**
 * Extract custom block metadata from a Vue SFC source string.
 * Parses root-level tags that are not <script>, <template>, or <style>.
 */
export function extractCustomBlocks(source: string): CustomBlockInfo[] {
  // First, strip the content of <script>, <template>, and <style> blocks
  // so the regex only matches truly root-level custom blocks (not inner HTML).
  const stripped = source.replace(/<(script|template|style)\b[^>]*>[\s\S]*?<\/\1>/gi, "");

  const blocks: CustomBlockInfo[] = [];
  const blockRegex = /<([a-z][a-z0-9-]*)([^>]*)>([\s\S]*?)<\/\1>/gi;
  let match;
  let index = 0;

  while ((match = blockRegex.exec(stripped)) !== null) {
    const type = match[1];
    const attrsStr = match[2];
    const content = match[3];
    const src = attrsStr.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null;
    const attrs = parseAttributes(attrsStr);

    blocks.push({ type, content, src, attrs, index });
    index++;
  }

  return blocks;
}

/**
 * Parse HTML-like attributes from a tag's attribute string.
 * Returns a map of attribute name → value (or `true` for boolean attributes).
 */
function parseAttributes(attrsStr: string): Record<string, string | true> {
  const attrs: Record<string, string | true> = {};
  const attrRegex = /\b([a-z][a-z0-9-]*)(?:=["']([^"']*)["'])?/gi;
  let match;

  while ((match = attrRegex.exec(attrsStr)) !== null) {
    attrs[match[1]] = match[2] ?? true;
  }

  return attrs;
}

/**
 * Extract <script src> and <template src> references from an SFC source.
 * Returns null for each if no src attribute is present.
 */
export function extractSrcInfo(source: string): SfcSrcInfo {
  const scriptMatch = source.match(/<script([^>]*)>/i);
  const templateMatch = source.match(/<template([^>]*)>/i);

  const scriptSrc = scriptMatch?.[1]?.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null;
  const templateSrc = templateMatch?.[1]?.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null;

  return { scriptSrc, templateSrc };
}

/**
 * Replace <script src="..."> or <template src="..."> with inline content
 * read from external files. This produces a self-contained SFC string
 * that can be passed to compileSfc.
 */
export function inlineSrcBlocks(
  source: string,
  scriptContent: string | null,
  templateContent: string | null,
): string {
  let result = source;

  if (scriptContent !== null) {
    // Replace <script ... src="..."> ... </script> with <script ...>content</script>
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
    // Replace <template ... src="..."> ... </template> with <template ...>content</template>
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

/**
 * Create a conditional logger.
 *
 * Copied from vite-plugin-vize/src/transform.ts
 */
export function createLogger(debug: boolean) {
  return {
    log: (...args: unknown[]) => debug && console.log("[vize:rspack]", ...args),
    info: (...args: unknown[]) => console.log("[vize:rspack]", ...args),
    warn: (...args: unknown[]) => console.warn("[vize:rspack]", ...args),
    error: (...args: unknown[]) => console.error("[vize:rspack]", ...args),
  };
}

/**
 * Match a file path against include/exclude patterns.
 * Used by both the main loader and the plugin for consistent filtering.
 *
 * Normalizes backslashes to forward slashes before matching,
 * so patterns like /src\/.*\.vue$/ work on Windows.
 */
export function matchesPattern(
  file: string,
  pattern: string | RegExp | (string | RegExp)[] | undefined,
  defaultValue: boolean,
): boolean {
  if (!pattern) {
    return defaultValue;
  }

  // Normalize Windows backslashes to forward slashes for cross-platform matching
  const normalizedFile = file.replace(/\\/g, "/");

  const patterns = Array.isArray(pattern) ? pattern : [pattern];
  return patterns.some((item) => {
    if (typeof item === "string") {
      return normalizedFile.includes(item) || file.includes(item);
    }
    return item.test(normalizedFile);
  });
}

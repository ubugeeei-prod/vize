/** Shared utilities for @vizejs/rspack-plugin. */

import {
  collectSfcTemplateAssetUrls,
  extractSfcCustomBlocks,
  extractSfcSrcInfo,
  extractSfcStyleBlocks,
  generateSfcScopeId,
  isSfcImportableAssetUrl,
  scopeViteCssForPipeline,
  stripSfcScopedCssComments,
} from "@vizejs/native";
import type {
  StyleBlockInfo,
  CustomBlockInfo,
  SfcSrcInfo,
  TemplateAssetUrl,
} from "../types/index.ts";

export interface NativeStyleBlock {
  content: string;
  src?: string;
  lang?: string;
  scoped: boolean;
  module: boolean;
  moduleName?: string;
  index: number;
}

export interface NativeBlockAttribute {
  name: string;
  value?: string;
}

export interface NativeCustomBlock {
  blockType: string;
  content: string;
  src?: string;
  attrs: NativeBlockAttribute[];
  index: number;
}

interface NativeTemplateAssetTagRule {
  tag: string;
  attrs: string[];
}

/** Generate scope ID (8-char SHA256 prefix). Uses relative path for cross-env consistency. */
export function generateScopeId(
  filename: string,
  rootContext?: string,
  isProduction?: boolean,
  source?: string,
): string {
  return generateSfcScopeId(filename, rootContext, isProduction, source);
}

/** Extract style block metadata from SFC source. */
export function extractStyleBlocks(source: string): StyleBlockInfo[] {
  return extractSfcStyleBlocks(source).map(toStyleBlockInfo);
}

/** Scope-loader prepass: strip CSS block comments without touching strings. */
export function stripCssCommentsForScoped(css: string): string {
  return stripSfcScopedCssComments(css);
}

/** Back-compat export for scoped CSS transformation. Delegates to the native CSS pipeline. */
export function addScopeToCssFallback(css: string, scopeId: string): string {
  const fullScopeId = scopeId.startsWith("data-v-") ? scopeId : `data-v-${scopeId}`;
  return scopeViteCssForPipeline(css, fullScopeId);
}

/** Extract custom block metadata from SFC source (non-script/template/style tags). */
export function extractCustomBlocks(source: string): CustomBlockInfo[] {
  return extractSfcCustomBlocks(source).map(toCustomBlockInfo);
}

/** Extract <script src> and <template src> references from SFC source. */
export function extractSrcInfo(source: string): SfcSrcInfo {
  const info = extractSfcSrcInfo(source);
  return {
    scriptSrc: info.scriptSrc ?? null,
    templateSrc: info.templateSrc ?? null,
  };
}

/** Replace <script src> or <template src> with inline content from external files. */
export function inlineSrcBlocks(
  source: string,
  scriptContent: string | null,
  templateContent: string | null,
): string {
  let result = source;

  if (scriptContent !== null) {
    result = result.replace(
      /(<script)([^>]*)\bsrc=["'][^"']+["']([^>]*>)[\s\S]*?(<\/script>)/i,
      (_, open, beforeSrc, afterSrc, close) => {
        const attrs = (beforeSrc + afterSrc).replace(/\bsrc=["'][^"']+["']\s*/g, "");
        return `${open}${attrs}\n${scriptContent}\n${close}`;
      },
    );
  }

  if (templateContent !== null) {
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

  const normalizedFile = file.replace(/\\/g, "/");
  const patterns = Array.isArray(pattern) ? pattern : [pattern];
  return patterns.some((item) => {
    if (typeof item === "string") {
      return normalizedFile.includes(item) || file.includes(item);
    }
    return testRegExp(item, normalizedFile);
  });
}

function testRegExp(pattern: RegExp, value: string): boolean {
  pattern.lastIndex = 0;
  const matches = pattern.test(value);
  pattern.lastIndex = 0;
  return matches;
}

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
  return isSfcImportableAssetUrl(url);
}

/** Scan SFC template for static asset URLs that should become import bindings. Deduplicated. */
export function collectTemplateAssetUrls(
  source: string,
  tags?: boolean | Record<string, string[]>,
): TemplateAssetUrl[] {
  if (tags === false) {
    return [];
  }

  return collectSfcTemplateAssetUrls(source, toNativeAssetTagRules(tags));
}

function toNativeAssetTagRules(
  tags?: boolean | Record<string, string[]>,
): NativeTemplateAssetTagRule[] | undefined {
  if (tags == null || tags === true) {
    return undefined;
  }

  return Object.entries(tags).map(([tag, attrs]) => ({ tag, attrs }));
}

export function toStyleBlockInfo(block: NativeStyleBlock): StyleBlockInfo {
  return {
    content: block.content,
    src: block.src ?? null,
    lang: block.lang ?? null,
    scoped: block.scoped,
    module: block.module ? (block.moduleName ?? true) : false,
    index: block.index,
  };
}

export function toCustomBlockInfo(block: NativeCustomBlock): CustomBlockInfo {
  const attrs: Record<string, string | true> = {};
  for (const attr of block.attrs) {
    attrs[attr.name] = attr.value ?? true;
  }

  return {
    type: block.blockType,
    content: block.content,
    src: block.src ?? null,
    attrs,
    index: block.index,
  };
}

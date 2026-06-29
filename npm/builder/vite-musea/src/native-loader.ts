/**
 * Native binding loader for @vizejs/native.
 *
 * Provides lazy-loading of the native Rust-based parser and a JS fallback
 * for SFC analysis when the native `analyzeSfc` function is unavailable.
 */

import { createRequire } from "node:module";

// Native binding types
export interface NativeBinding {
  parseArt: (
    source: string,
    options?: { filename?: string },
  ) => {
    filename: string;
    metadata: {
      title: string;
      description?: string;
      component?: string;
      category?: string;
      tags: string[];
      status: string;
      order?: number;
      actionEvents?: string[];
    };
    variants: Array<{
      name: string;
      template: string;
      isDefault: boolean;
      skipVrt: boolean;
    }>;
    hasScriptSetup: boolean;
    hasScript: boolean;
    styleCount: number;
  };
  artToCsf: (
    source: string,
    options?: { filename?: string },
  ) => {
    code: string;
    filename: string;
  };
  generateArtPalette?: (
    source: string,
    artOptions?: { filename?: string },
    paletteOptions?: { infer_options?: boolean; group_by_type?: boolean },
  ) => {
    title: string;
    controls: Array<{
      name: string;
      control: string;
      default_value?: unknown;
      description?: string;
      required: boolean;
      options: Array<{ label: string; value: unknown }>;
      range?: { min: number; max: number; step?: number };
      group?: string;
    }>;
    groups: string[];
    json: string;
    typescript: string;
  };
  generateArtDoc?: (
    source: string,
    artOptions?: { filename?: string },
    docOptions?: {
      include_source?: boolean;
      include_templates?: boolean;
      include_metadata?: boolean;
    },
  ) => {
    markdown: string;
    filename: string;
    title: string;
    category?: string;
    variant_count: number;
  };
  parseDesignTokensFromPath?: (path: string) => unknown;
  buildDesignTokenMap?: (categories: unknown) => Record<string, unknown>;
  resolveDesignTokenReferences?: (categories: unknown) => unknown;
  flattenDesignTokenCategories?: (categories: unknown) => unknown[];
  generateDesignTokensMarkdown?: (categories: unknown, generatedAt?: string) => string;
  validateDesignTokenReference?: (
    tokenMap: Record<string, unknown>,
    reference: string,
    selfPath?: string,
  ) => { valid: boolean; error?: string };
  findDependentDesignTokens?: (tokenMap: Record<string, unknown>, targetPath: string) => string[];
  analyzeSfc?: (
    source: string,
    options?: { filename?: string },
  ) => {
    props: Array<{
      name: string;
      type: string;
      required: boolean;
      default_value?: unknown;
    }>;
    emits: string[];
  };
}

// Lazy-load native binding
let native: NativeBinding | null = null;

export function loadNative(): NativeBinding {
  if (native) return native;

  const require = createRequire(import.meta.url);
  try {
    native = require("@vizejs/native") as NativeBinding;
    return native;
  } catch (e) {
    throw new Error(
      `Failed to load @vizejs/native. Make sure it's installed and built:\n${String(e)}`,
    );
  }
}

/**
 * JS-based fallback for SFC analysis when native `analyzeSfc` is not available.
 * Uses regex parsing to extract props and emits from Vue SFC source.
 */
export function analyzeSfcFallback(
  source: string,
  _options?: { filename?: string },
): {
  props: Array<{
    name: string;
    type: string;
    required: boolean;
    default_value?: unknown;
  }>;
  emits: string[];
} {
  try {
    const props: Array<{
      name: string;
      type: string;
      required: boolean;
      default_value?: unknown;
    }> = [];
    const emits: string[] = [];

    // Extract the <script setup> block
    const scriptSetupMatch = source.match(/<script\s+[^>]*setup[^>]*>([\s\S]*?)<\/script>/);
    if (!scriptSetupMatch) {
      // Try regular <script> block
      const scriptMatch = source.match(/<script[^>]*>([\s\S]*?)<\/script>/);
      if (!scriptMatch) return { props: [], emits: [] };
    }
    const scriptContent = scriptSetupMatch?.[1] || "";

    // Extract defineProps type parameter
    // Handles: defineProps<{ ... }>()  and  defineProps<{ ... }>
    const propsMatch = scriptContent.match(/defineProps\s*<\s*\{([\s\S]*?)\}>\s*\(/);
    const propsMatch2 = scriptContent.match(/defineProps\s*<\s*\{([\s\S]*?)\}>/);
    const propsBody = propsMatch?.[1] || propsMatch2?.[1];
    const withDefaults = extractWithDefaults(scriptContent);

    if (propsBody) {
      // Parse each prop line: name?: Type;  or  name: Type;
      // Handle multiline JSDoc comments before props
      const lines = propsBody.split("\n");
      let i = 0;
      while (i < lines.length) {
        const line = lines[i].trim();
        // Skip JSDoc comments
        if (line.startsWith("/**") || line.startsWith("*") || line.startsWith("*/")) {
          i++;
          continue;
        }

        // Match prop definition: name?: Type  or  name: Type
        const propMatch = line.match(/^(\w+)(\?)?:\s*(.+?)(?:;?\s*)$/);
        if (propMatch) {
          const name = propMatch[1];
          const optional = !!propMatch[2];
          let type = propMatch[3].replace(/;$/, "").trim();

          const defaultValue = withDefaults.get(name);

          props.push({
            name,
            type,
            required: !optional && defaultValue === undefined,
            ...(defaultValue !== undefined ? { default_value: defaultValue } : {}),
          });
        }
        i++;
      }
    }

    // Extract defineEmits
    const emitsMatch = scriptContent.match(/defineEmits\s*<\s*\{([\s\S]*?)\}>/);
    if (emitsMatch) {
      const emitsBody = emitsMatch[1];
      const emitRegex = /(\w+)\s*:/g;
      let match;
      while ((match = emitRegex.exec(emitsBody)) !== null) {
        emits.push(match[1]);
      }
    }

    return { props, emits };
  } catch {
    return { props: [], emits: [] };
  }
}

function extractWithDefaults(scriptContent: string): Map<string, string> {
  const defaults = new Map<string, string>();
  let searchIndex = 0;

  while (searchIndex < scriptContent.length) {
    const calleeIndex = scriptContent.indexOf("withDefaults", searchIndex);
    if (calleeIndex === -1) break;
    searchIndex = calleeIndex + "withDefaults".length;

    if (!isIdentifierBoundary(scriptContent[calleeIndex - 1])) continue;
    if (!isIdentifierBoundary(scriptContent[searchIndex])) continue;

    const parenIndex = skipWhitespace(scriptContent, searchIndex);
    if (scriptContent[parenIndex] !== "(") continue;

    const endParen = findMatching(scriptContent, parenIndex, "(", ")");
    if (endParen === -1) continue;

    const args = splitTopLevel(scriptContent.slice(parenIndex + 1, endParen));
    const defaultsArg = args[1]?.trim();
    if (!defaultsArg?.startsWith("{")) continue;

    const objectStart = scriptContent.indexOf(defaultsArg, parenIndex + 1);
    const objectEnd = findMatching(scriptContent, objectStart, "{", "}");
    if (objectEnd === -1) continue;

    for (const [name, value] of objectProperties(scriptContent.slice(objectStart + 1, objectEnd))) {
      defaults.set(name, value);
    }
  }

  return defaults;
}

function objectProperties(objectBody: string): Array<[string, string]> {
  const properties: Array<[string, string]> = [];

  for (const item of splitTopLevel(objectBody)) {
    const colonIndex = topLevelColon(item);
    if (colonIndex === -1) continue;

    const rawKey = item.slice(0, colonIndex).trim();
    const key = rawKey.match(/^[$A-Z_a-z][$\w]*$/)?.[0] ?? rawKey.match(/^["']([^"']+)["']$/)?.[1];
    if (!key) continue;

    const value = item.slice(colonIndex + 1).trim();
    if (value) properties.push([key, value]);
  }

  return properties;
}

function splitTopLevel(source: string): string[] {
  const parts: string[] = [];
  let start = 0;
  let round = 0;
  let curly = 0;
  let square = 0;
  let quote: '"' | "'" | "`" | null = null;
  let escaped = false;

  for (let index = 0; index < source.length; index++) {
    const char = source[index]!;

    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === "(") round++;
    else if (char === ")") round--;
    else if (char === "{") curly++;
    else if (char === "}") curly--;
    else if (char === "[") square++;
    else if (char === "]") square--;
    else if (char === "," && round === 0 && curly === 0 && square === 0) {
      parts.push(source.slice(start, index).trim());
      start = index + 1;
    }
  }

  const tail = source.slice(start).trim();
  if (tail) parts.push(tail);
  return parts;
}

function topLevelColon(source: string): number {
  let round = 0;
  let curly = 0;
  let square = 0;
  let quote: '"' | "'" | "`" | null = null;
  let escaped = false;

  for (let index = 0; index < source.length; index++) {
    const char = source[index]!;

    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === "(") round++;
    else if (char === ")") round--;
    else if (char === "{") curly++;
    else if (char === "}") curly--;
    else if (char === "[") square++;
    else if (char === "]") square--;
    else if (char === ":" && round === 0 && curly === 0 && square === 0) {
      return index;
    }
  }

  return -1;
}

function findMatching(source: string, start: number, open: string, close: string): number {
  let depth = 0;
  let quote: '"' | "'" | "`" | null = null;
  let escaped = false;

  for (let index = start; index < source.length; index++) {
    const char = source[index]!;

    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === open) depth++;
    if (char === close) {
      depth--;
      if (depth === 0) return index;
    }
  }

  return -1;
}

function skipWhitespace(source: string, index: number): number {
  while (index < source.length && /\s/.test(source[index]!)) index++;
  return index;
}

function isIdentifierBoundary(char: string | undefined): boolean {
  return char === undefined || !/[$\w]/.test(char);
}

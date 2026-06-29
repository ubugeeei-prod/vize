/**
 * Parsing helpers for extracting prop defaults from a Vue SFC `<script setup>`
 * block. Defaults are read only from `withDefaults(defineProps(...), { ... })`
 * so unrelated local identifiers are never treated as prop defaults.
 */

export function extractWithDefaults(scriptContent: string): Map<string, string> {
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

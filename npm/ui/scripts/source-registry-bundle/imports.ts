/**
 * Dependency-free module specifier scanner for registry sources.
 *
 * The registry must be reproducible from the published package alone, so this
 * scanner avoids the TypeScript compiler API. It removes comments with a small
 * lexer that understands strings, template literals, and regular expression
 * literals, then matches static, re-export, side-effect, and dynamic imports.
 */

const regexPrecedingCharacters = new Set([
  "(",
  ",",
  "=",
  ":",
  "[",
  "!",
  "&",
  "|",
  "?",
  "{",
  "}",
  ";",
]);
const regexPrecedingKeywords = /(?:^|[^\w$])(?:return|typeof|case|do|else|in|of|void|yield|await)$/;

function previousSignificant(output: string): string {
  for (let index = output.length - 1; index >= 0; index -= 1) {
    const character = output.charAt(index);
    if (!/\s/.test(character)) return character;
  }
  return "";
}

function startsRegexLiteral(output: string): boolean {
  const previous = previousSignificant(output);
  if (previous === "" || regexPrecedingCharacters.has(previous)) return true;
  return regexPrecedingKeywords.test(output.trimEnd());
}

/** Copy a quoted string or template literal starting at `start`; returns the end index. */
function skipQuoted(source: string, start: number, quote: string): number {
  let index = start + 1;
  while (index < source.length) {
    const character = source.charAt(index);
    if (character === "\\") {
      index += 2;
      continue;
    }
    if (character === quote) return index + 1;
    if (quote !== "`" && character === "\n") return index;
    index += 1;
  }
  return index;
}

function skipRegex(source: string, start: number): number {
  let index = start + 1;
  let inClass = false;
  while (index < source.length) {
    const character = source.charAt(index);
    if (character === "\\") {
      index += 2;
      continue;
    }
    if (character === "\n") return index;
    if (character === "[") inClass = true;
    else if (character === "]") inClass = false;
    else if (character === "/" && !inClass) return index + 1;
    index += 1;
  }
  return index;
}

/**
 * Remove line and block comments and blank regular expression bodies while
 * keeping every string literal intact.
 * Newlines inside block comments are preserved so offsets stay readable.
 */
export function stripComments(source: string): string {
  let output = "";
  let index = 0;
  while (index < source.length) {
    const character = source.charAt(index);
    const next = source.charAt(index + 1);
    if (character === "/" && next === "/") {
      const end = source.indexOf("\n", index);
      index = end === -1 ? source.length : end;
      continue;
    }
    if (character === "/" && next === "*") {
      const end = source.indexOf("*/", index + 2);
      const stop = end === -1 ? source.length : end + 2;
      output += source.slice(index, stop).replace(/[^\n]/g, "");
      output += " ";
      index = stop;
      continue;
    }
    if (character === '"' || character === "'" || character === "`") {
      const end = skipQuoted(source, index, character);
      output += source.slice(index, end);
      index = end;
      continue;
    }
    if (character === "/" && startsRegexLiteral(output)) {
      // Regex bodies can never import anything; blank them so quoted text
      // inside a pattern cannot look like an import to the matchers below.
      const end = skipRegex(source, index);
      output += `/${" ".repeat(Math.max(0, end - index - 2))}/`;
      index = end;
      continue;
    }
    output += character;
    index += 1;
  }
  return output;
}

/** Concatenate the contents of every `<script>` block of a Vue SFC. */
export function extractVueScripts(source: string): string {
  const blocks: string[] = [];
  for (const match of source.matchAll(/<script\b[^>]*>([\s\S]*?)<\/script>/g)) {
    blocks.push(match[1] ?? "");
  }
  return blocks.join("\n");
}

const staticImportPattern =
  /(?:^|[^\w$.])(?:import|export)\s*(?:type\s+)?(?:[\w$*{}\s,]*?\s*from\s*)?(["'])([^"'\n]+)\1/g;
const dynamicImportPattern = /(?:^|[^\w$.])import\s*\(\s*(["'])([^"'\n]+)\1\s*\)/g;

/**
 * Return every module specifier imported by a `.ts` or `.vue` source, in
 * first-occurrence order and without duplicates.
 */
export function scanModuleSpecifiers(fileName: string, source: string): readonly string[] {
  const code = stripComments(fileName.endsWith(".vue") ? extractVueScripts(source) : source);
  const specifiers = new Set<string>();
  for (const pattern of [staticImportPattern, dynamicImportPattern]) {
    for (const match of code.matchAll(pattern)) {
      const specifier = match[2];
      if (specifier != null) specifiers.add(specifier);
    }
  }
  return [...specifiers];
}

/** Relative `@import` targets of a CSS file (comments removed first). */
export function scanCssImports(source: string): readonly string[] {
  const code = source.replace(/\/\*[\s\S]*?\*\//g, "");
  const specifiers = new Set<string>();
  for (const match of code.matchAll(/@import\s+(?:url\(\s*)?(["'])([^"'\n]+)\1/g)) {
    const specifier = match[2];
    if (specifier != null) specifiers.add(specifier);
  }
  return [...specifiers];
}

/** npm package name for a bare specifier (`vue/server-renderer` -> `vue`). */
export function packageNameOfSpecifier(specifier: string): string {
  const segments = specifier.split("/");
  const [first = "", second = ""] = segments;
  return first.startsWith("@") ? `${first}/${second}` : first;
}

/** Whether a specifier is relative to the importing file. */
export function isRelativeSpecifier(specifier: string): boolean {
  return specifier.startsWith("./") || specifier.startsWith("../");
}

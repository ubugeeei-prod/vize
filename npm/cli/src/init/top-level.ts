/**
 * Depth-aware lookup of a top-level key in a `defineConfig({ ... })` call.
 *
 * A plain regex cannot tell the config's own `plugins` key from the `plugins`
 * key inside a `lint: { ... }` block, and picking the wrong one rewrites a part
 * of the user's config they never asked to change. This scanner tracks bracket
 * depth and skips strings, template literals and comments, so a key only matches
 * at depth 0 of the config object.
 *
 * It is not a JavaScript parser and does not try to be: template-literal
 * substitutions and regex literals are treated as ordinary text. Both make the
 * scan give up or miss, which turns into a refusal to edit -- the safe direction.
 */

const IDENTIFIER = /^[$A-Z_a-z][$\w]*/u;
const KEY_SEPARATOR = /^\s*:/u;

export interface TopLevelKey {
  /** Index of the first character of the key. */
  readonly keyStart: number;
  /** Index of the first character after the `:`. */
  readonly valueStart: number;
}

/** Finds `key` at the top level of `callee({ ... })`, or `null`. */
export function findTopLevelKey(source: string, callee: string, key: string): TopLevelKey | null {
  const opening = new RegExp(`\\b${callee}\\s*\\(\\s*\\{`, "u").exec(source);
  if (opening === null) {
    return null;
  }
  let index = opening.index + opening[0].length;
  let depth = 0;
  while (index < source.length) {
    const char = source[index]!;
    const skipped = skipNonCode(source, index);
    if (skipped !== index) {
      index = skipped;
      continue;
    }
    if (char === "{" || char === "[" || char === "(") {
      depth += 1;
      index += 1;
      continue;
    }
    if (char === "}" || char === "]" || char === ")") {
      if (depth === 0) {
        // Closing brace of the config object itself: the key is not here.
        return null;
      }
      depth -= 1;
      index += 1;
      continue;
    }
    const identifier = IDENTIFIER.exec(source.slice(index));
    if (identifier === null) {
      index += 1;
      continue;
    }
    const separator = KEY_SEPARATOR.exec(source.slice(index + identifier[0].length));
    if (depth === 0 && identifier[0] === key && separator !== null) {
      return {
        keyStart: index,
        valueStart: index + identifier[0].length + separator[0].length,
      };
    }
    index += identifier[0].length;
  }
  return null;
}

/** Number of `callee({` openings in the source. */
export function countConfigCalls(source: string, callee: string): number {
  return [...source.matchAll(new RegExp(`\\b${callee}\\s*\\(\\s*\\{`, "gu"))].length;
}

export interface ArrayValue {
  /** Index just after the opening `[`. */
  readonly contentStart: number;
  /** True when the array holds nothing but whitespace. */
  readonly empty: boolean;
}

/**
 * Reads the array literal a top-level key is assigned to.
 *
 * Returns `null` when the value is not an array literal -- a spread from a
 * variable, or a helper call -- because inserting into those would change what
 * the config evaluates to.
 */
export function readTopLevelArray(source: string, callee: string, key: string): ArrayValue | null {
  const found = findTopLevelKey(source, callee, key);
  if (found === null) {
    return null;
  }
  const rest = source.slice(found.valueStart);
  const leading = /^\s*/u.exec(rest)![0];
  if (rest[leading.length] !== "[") {
    return null;
  }
  const contentStart = found.valueStart + leading.length + 1;
  return { contentStart, empty: /^\s*\]/u.test(source.slice(contentStart)) };
}

/**
 * Advances past a string, template literal or comment starting at `index`.
 *
 * Returns `index` unchanged when nothing at that position needs skipping.
 */
function skipNonCode(source: string, index: number): number {
  const char = source[index]!;
  if (char === '"' || char === "'" || char === "`") {
    return skipQuoted(source, index, char);
  }
  if (char !== "/") {
    return index;
  }
  const next = source[index + 1];
  if (next === "/") {
    const end = source.indexOf("\n", index);
    return end === -1 ? source.length : end;
  }
  if (next === "*") {
    const end = source.indexOf("*/", index + 2);
    return end === -1 ? source.length : end + 2;
  }
  return index;
}

function skipQuoted(source: string, index: number, quote: string): number {
  let cursor = index + 1;
  while (cursor < source.length) {
    const char = source[cursor]!;
    if (char === "\\") {
      cursor += 2;
      continue;
    }
    if (char === quote) {
      return cursor + 1;
    }
    cursor += 1;
  }
  return source.length;
}

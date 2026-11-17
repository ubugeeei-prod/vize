// html/no-duplicate-class differences where vue/no-duplicate-class-names is
// the one that is wrong. Both read only the element's start tag.
//
// - An identifier object key renders its class (`:class="{ hidden: cond }"`),
//   so Patina counts it. Upstream counts only string-literal keys, so a name
//   repeated through such a key is missed there.
// - Upstream splits template quasis on whitespace, so a fragment glued to an
//   interpolation (`-600` in `` `text-${color}-600` ``) counts as a class.
//   It is part of a computed class, so Patina skips it.

export const identifierKeyClassReason =
  "Patina counts an identifier object key in :class as the class it renders; eslint-plugin-vue's vue/no-duplicate-class-names counts only string-literal keys, so it misses a name repeated through such a key.";
export const classFragmentReason =
  "A template-literal fragment glued to a ${} interpolation is part of a computed class, not a whole class; eslint-plugin-vue's vue/no-duplicate-class-names counts it as one.";

interface ClassSources {
  /** Names from the static `class` value, string literals and whole quasi tokens. */
  readonly literal: string[];
  /** Quasi tokens glued to an interpolation. */
  readonly glued: string[];
  /** Identifier object keys, including shorthand properties. */
  readonly identifierKeys: string[];
}

/** A Patina finding whose every name repeats only through identifier keys. */
export function identifierKeyDuplicate(message: string, offset: number, source: string): boolean {
  const sources = startTagClassSources(source, offset);
  const names = quotedNames(message);
  return (
    sources !== null &&
    names.length > 0 &&
    names.every(
      (name) =>
        sources.identifierKeys.includes(name) &&
        count(sources.literal, name) + count(sources.glued, name) <= 1,
    )
  );
}

/** An upstream finding whose every name is a fragment glued to an interpolation. */
export function gluedFragmentDuplicate(message: string, offset: number, source: string): boolean {
  const sources = startTagClassSources(source, offset);
  const names = quotedNames(message);
  return (
    sources !== null &&
    names.length > 0 &&
    names.every(
      (name) =>
        sources.glued.includes(name) &&
        count(sources.literal, name) + count(sources.identifierKeys, name) <= 1,
    )
  );
}

function quotedNames(message: string): string[] {
  const list = message.replace(/^\[vize:[^\]]+\] /u, "");
  return [...list.matchAll(/'([^']+)'/gu)].map((match) => match[1] ?? "");
}

function count(names: readonly string[], name: string): number {
  return names.filter((candidate) => candidate === name).length;
}

/** The class sources of the start tag that contains `offset`. */
function startTagClassSources(source: string, offset: number): ClassSources | null {
  const open = source.lastIndexOf("<", offset);
  if (open < 0 || !/[A-Za-z]/u.test(source[open + 1] ?? "")) return null;
  const end = tagEnd(source, open);
  if (end === null || end <= offset) return null;
  const tag = source.slice(open, end);
  const sources: ClassSources = { literal: [], glued: [], identifierKeys: [] };
  let staticSeen = false;
  for (const match of tag.matchAll(
    /\s(v-bind:class|:class|class)\s*=\s*(?:"([^"]*)"|'([^']*)')/gu,
  )) {
    const value = match[2] ?? match[3] ?? "";
    if (match[1] === "class") {
      if (!staticSeen) sources.literal.push(...words(value));
      staticSeen = true;
    } else scanExpression(value, sources);
  }
  return sources;
}

/** The index after the `>` closing the tag that opens at `open`. */
function tagEnd(source: string, open: number): number | null {
  let quote: string | null = null;
  for (let index = open + 1; index < source.length; index++) {
    const character = source[index];
    if (quote !== null) {
      if (character === quote) quote = null;
    } else if (character === '"' || character === "'") quote = character;
    else if (character === "<") return null;
    else if (character === ">") return index + 1;
  }
  return null;
}

function words(text: string): string[] {
  return text.split(/\s+/u).filter(Boolean);
}

/** Collect string, quasi and identifier-key names from a `:class` expression. */
function scanExpression(expression: string, sources: ClassSources): void {
  let masked = "";
  for (let index = 0; index < expression.length;) {
    const character = expression[index] ?? "";
    if (character === "'" || character === '"') {
      const close = stringClose(expression, index);
      sources.literal.push(...words(expression.slice(index + 1, close)));
      masked += " ".repeat(close + 1 - index);
      index = close + 1;
    } else if (character === "`") {
      const close = templateLiteral(expression, index, sources);
      masked += " ".repeat(close + 1 - index);
      index = close + 1;
    } else {
      masked += character;
      index++;
    }
  }
  for (const match of masked.matchAll(/[{,]\s*([A-Za-z_$][\w$]*)\s*(?=[:,}])/gu))
    sources.identifierKeys.push(match[1] ?? "");
}

function stringClose(text: string, open: number): number {
  for (let index = open + 1; index < text.length; index++) {
    if (text[index] === "\\") index++;
    else if (text[index] === text[open]) return index;
  }
  return text.length - 1;
}

/** Split a template literal into quasis; interpolations are scanned as expressions. */
function templateLiteral(text: string, open: number, sources: ClassSources): number {
  const quasis: string[] = [];
  let quasi = "";
  let index = open + 1;
  for (; index < text.length; index++) {
    const character = text[index];
    if (character === "\\") {
      quasi += text.slice(index, index + 2);
      index++;
    } else if (character === "`") break;
    else if (character === "$" && text[index + 1] === "{") {
      quasis.push(quasi);
      quasi = "";
      let depth = 1;
      const start = index + 2;
      for (index = start; index < text.length && depth > 0; index++) {
        if (text[index] === "{") depth++;
        else if (text[index] === "}") depth--;
      }
      scanExpression(text.slice(start, index - 1), sources);
      index--;
    } else quasi += character;
  }
  quasis.push(quasi);
  quasis.forEach((part, position) => {
    const tokens = words(part);
    tokens.forEach((token, tokenIndex) => {
      const gluedLeft = position > 0 && tokenIndex === 0 && !/^\s/u.test(part);
      const gluedRight =
        position < quasis.length - 1 && tokenIndex === tokens.length - 1 && !/\s$/u.test(part);
      (gluedLeft || gluedRight ? sources.glued : sources.literal).push(token);
    });
  });
  return index;
}

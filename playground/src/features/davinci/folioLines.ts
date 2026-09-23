// Folio page text -> display lines: syntax tokens plus the authored span each
// line points at. The grammar being read is the committed folio format
// (`docs/davinci/plan/folio-format.md`, `folio-format-impeto.md`); nothing
// here re-derives compiler facts, it only reads the spans the pages print.

import type { Range } from "./offsets";

export type PageKind =
  | "surface"
  | "disegno"
  | "plan"
  | "provenance"
  | "impeto"
  | "partition"
  | "values";

export type TokenType =
  | "section"
  | "key"
  | "mnemonic"
  | "string"
  | "span"
  | "number"
  | "call"
  | "static"
  | "dynamic"
  | "absent"
  | "punct"
  | "tag"
  | "attr"
  | "directive"
  | "mustache"
  | "text";

export interface Token {
  type: TokenType;
  text: string;
}

export interface FolioLine {
  /** 0-based line index within the page. */
  index: number;
  text: string;
  tokens: Token[];
  /** The authored template byte range this line describes, if any. */
  span: Range | null;
  /** Nesting depth for tree pages (two spaces per level). */
  depth: number;
  /**
   * For an S2 op line, the op's dense page-order id (the `ui.*` lines in
   * document order) - the id provenance records name. Null elsewhere.
   */
  node: number | null;
}

const FOLIO_TOKEN =
  /("(?:[^"\\]|\\.)*")|(@\d+:\d+)|(\b[a-z][\w-]*=)|(\b(?:ui|impeto|vue)\.[\w.-]+|\b(?:attr|branch)\b)|(\b(?:js|opaque|foreign|compound)\()|(\bstatic\b)|(\bdynamic\b)|(\d+:\d+)|(\bnull\b|(?<==)-(?=\s|$))|(\d+)|([()[\],])/g;

const FOLIO_GROUPS: TokenType[] = [
  "string",
  "span",
  "key",
  "mnemonic",
  "call",
  "static",
  "dynamic",
  "span",
  "absent",
  "number",
  "punct",
];

function scan(text: string, pattern: RegExp, groups: TokenType[]): Token[] {
  const tokens: Token[] = [];
  let last = 0;
  pattern.lastIndex = 0;
  for (let match = pattern.exec(text); match; match = pattern.exec(text)) {
    if (match.index > last) tokens.push({ type: "text", text: text.slice(last, match.index) });
    const group = match.slice(1).findIndex((value) => value !== undefined);
    tokens.push({ type: groups[group] ?? "text", text: match[0] });
    last = match.index + match[0].length;
  }
  if (last < text.length) tokens.push({ type: "text", text: text.slice(last) });
  return tokens;
}

/** Tokens for one folio page line. */
export function folioTokens(line: string): Token[] {
  if (/^\[[\w.-]+\]$/.test(line)) return [{ type: "section", text: line }];
  const tokens = scan(line, FOLIO_TOKEN, FOLIO_GROUPS);
  // A provenance rule name, or a plan's pass name, reads as a mnemonic.
  for (let index = 0; index + 1 < tokens.length; index += 1) {
    const [key, value] = [tokens[index], tokens[index + 1]];
    if (key.type !== "key" || (key.text !== "rule=" && key.text !== "pass=")) continue;
    if (value.type !== "text" || !/^\S/.test(value.text)) continue;
    const [, name, rest] = /^(\S+)(.*)$/s.exec(value.text)!;
    tokens.splice(index + 1, 1, { type: "mnemonic", text: name });
    if (rest) tokens.splice(index + 2, 0, { type: "text", text: rest });
  }
  // The element/component name right after its mnemonic reads as a tag.
  for (let index = 0; index + 1 < tokens.length; index += 1) {
    const [mnemonic, gap] = [tokens[index], tokens[index + 1]];
    if (
      mnemonic.type === "mnemonic" &&
      (mnemonic.text === "ui.element" || mnemonic.text === "ui.component") &&
      gap.type === "text" &&
      /^ \S/.test(gap.text)
    ) {
      const [, word, rest] = /^ (\S+)(.*)$/s.exec(gap.text)!;
      tokens.splice(index + 1, 1, { type: "text", text: " " }, { type: "tag", text: word });
      if (rest) tokens.splice(index + 3, 0, { type: "text", text: rest });
    }
  }
  return tokens;
}

const SURFACE_TOKEN =
  /(\{\{[\s\S]*?\}\})|(<\/?[A-Za-z][\w.:-]*|\/?>)|((?:v-[\w-]+|[:@#][\w.[\]-]*)(?:[:.][\w-]+)*)(?==)|([A-Za-z_][\w-]*)(?==)|("[^"]*"|'[^']*')/g;

const SURFACE_GROUPS: TokenType[] = ["mustache", "tag", "directive", "attr", "string"];

/** Tokens for one line of the S1 surface page (authored template text). */
export function surfaceTokens(line: string): Token[] {
  return scan(line, SURFACE_TOKEN, SURFACE_GROUPS);
}

function lastMatch(pattern: RegExp, line: string): RegExpExecArray | null {
  let found: RegExpExecArray | null = null;
  const global = new RegExp(pattern.source, "g");
  for (let match = global.exec(line); match; match = global.exec(line)) found = match;
  return found;
}

/** The authored span a folio line describes, by page kind. */
export function lineSpan(kind: PageKind, line: string): Range | null {
  switch (kind) {
    case "disegno":
    case "provenance": {
      // An op (or record) line ends in its own `@start:end`; inner `@` spans
      // belong to its expressions.
      const match = /@(\d+):(\d+)\s*$/.exec(line);
      return match ? { start: Number(match[1]), end: Number(match[2]) } : null;
    }
    case "impeto":
    case "partition": {
      const match = lastMatch(/\bspan=(\d+):(\d+)/, line);
      return match ? { start: Number(match[1]), end: Number(match[2]) } : null;
    }
    case "values": {
      const match = /,(\d+),(\d+)\]$/.exec(line);
      return match ? { start: Number(match[1]), end: Number(match[2]) } : null;
    }
    case "surface":
    case "plan":
      return null;
  }
}

function utf8Bytes(text: string): number {
  return new TextEncoder().encode(text).length;
}

/** Split a page into display lines with tokens and spans. */
export function folioLines(kind: PageKind, text: string): FolioLine[] {
  const raw = text.endsWith("\n") ? text.slice(0, -1).split("\n") : text.split("\n");
  let byteCursor = 0;
  let nextNode = 0;
  return raw.map((line, index) => {
    let span: Range | null;
    if (kind === "surface") {
      const length = utf8Bytes(line);
      span = length === 0 ? null : { start: byteCursor, end: byteCursor + length };
      byteCursor += length + 1;
    } else {
      span = lineSpan(kind, line);
    }
    return {
      index,
      text: line,
      tokens: kind === "surface" ? surfaceTokens(line) : folioTokens(line),
      span,
      depth: kind === "disegno" ? Math.floor((/^ */.exec(line)?.[0].length ?? 0) / 2) : 0,
      node: kind === "disegno" && /^\s*ui\./.test(line) ? nextNode++ : null,
    };
  });
}

/**
 * Lines whose span covers template byte offset `offset`, narrowest first -
 * the reverse provenance query (source position -> stage lines).
 */
export function linesCovering(lines: FolioLine[], offset: number): number[] {
  return lines
    .filter((line) => line.span && line.span.start <= offset && offset < line.span.end)
    .sort((left, right) => {
      const a = left.span!;
      const b = right.span!;
      return a.end - a.start - (b.end - b.start) || left.index - right.index;
    })
    .map((line) => line.index);
}

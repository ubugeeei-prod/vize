// A Rust lexer for source-text checks that must not compile anything.
//
// It splits a file into tokens (identifiers, lifetimes, literals, punctuation
// and doc comments) and plain comments, with the line and column of each. The
// token stream follows proc-macro2's model: punctuation is one character per
// token with a `joint` flag when another punctuation character follows at
// once, so `&&` and `& &` stay different while every other run of whitespace
// is dropped. Doc comments are tokens because rustc turns them into
// `#[doc = "…"]` attributes; plain comments are not.

export type RustTokenKind = "ident" | "lifetime" | "literal" | "punct" | "doc";

export interface RustToken {
  readonly kind: RustTokenKind;
  readonly text: string;
  readonly line: number;
  readonly column: number;
  /** UTF-16 offsets of the token in the source. */
  readonly start: number;
  readonly end: number;
  /** Punctuation only: the next character is punctuation too. */
  readonly joint?: boolean;
}

export interface RustComment {
  /** `doc` is an outer or inner doc comment, line or block; the rest are `line` or `block`. */
  readonly kind: "line" | "block" | "doc";
  /** Comment body without its delimiters, one entry per source line. */
  readonly lines: readonly string[];
  readonly line: number;
  readonly start: number;
  readonly end: number;
}

export interface RustLexResult {
  readonly tokens: readonly RustToken[];
  readonly comments: readonly RustComment[];
}

const punctuation = new Set("!#$%&*+,-./:;<=>?@^|~");
const asciiEnd = 0x80;
const lastBmpCodePoint = 0xffff;
const isDigit = (char: string) => char >= "0" && char <= "9";
const isAsciiLetter = (char: string) =>
  (char >= "a" && char <= "z") || (char >= "A" && char <= "Z");
const isIdentStart = (char: string) => {
  if (char.charCodeAt(0) < asciiEnd) return isAsciiLetter(char) || char === "_";
  return /\p{L}/u.test(char);
};
const isIdentContinue = (char: string) => {
  if (char.charCodeAt(0) < asciiEnd) return isAsciiLetter(char) || isDigit(char) || char === "_";
  return /[\p{L}\p{N}]/u.test(char);
};
const isSpace = (char: string) =>
  char === " " ||
  char === "\n" ||
  char === "\t" ||
  char === "\r" ||
  (char > "~" && /\s/u.test(char));

class RustLexer {
  readonly tokens: RustToken[] = [];
  readonly comments: RustComment[] = [];
  private index = 0;
  private line = 1;
  private lineStart = 0;

  private readonly source: string;

  constructor(source: string) {
    this.source = source;
  }

  run(): RustLexResult {
    const { source } = this;
    if (source.startsWith("#!") && !/^#!\s*\[/u.test(source)) {
      const end = source.indexOf("\n");
      this.advanceTo(end < 0 ? source.length : end);
    }
    while (this.index < source.length) this.step();
    return { tokens: this.tokens, comments: this.comments };
  }

  private step(): void {
    const char = this.source[this.index];
    const next = this.source[this.index + 1] ?? "";
    if (isSpace(char)) this.advanceTo(this.index + 1);
    else if (char === "/" && next === "/") this.lineComment();
    else if (char === "/" && next === "*") this.blockComment();
    else if (char === '"') this.push("literal", this.quotedEnd(this.index + 1, '"'));
    else if ((char === "r" || char === "b" || char === "c") && this.prefixedLiteral()) return;
    else if (char === "'" && this.quoteMark(next)) return;
    else if (isDigit(char)) this.push("literal", this.numberEnd());
    else if (isIdentStart(char)) this.push("ident", this.identEnd(this.index));
    // Unknown symbols are kept as punctuation so they can never hide.
    else
      this.push("punct", this.index + 1, {
        joint:
          punctuation.has(next) &&
          punctuation.has(char) &&
          !(next === "/" && ["/", "*"].includes(this.source[this.index + 2])),
      });
  }

  private advanceTo(end: number): void {
    for (let at = this.index; at < end; at++)
      if (this.source[at] === "\n") {
        this.line++;
        this.lineStart = at + 1;
      }
    this.index = end;
  }

  private push(kind: RustTokenKind, end: number, extra?: { joint: boolean }): void {
    const { index, line } = this;
    const text = this.source.slice(index, end);
    const column = index - this.lineStart + 1;
    this.tokens.push({ kind, text, line, column, start: index, end, ...extra });
    this.advanceTo(end);
  }

  private lineComment(): void {
    const newline = this.source.indexOf("\n", this.index);
    const end = newline < 0 ? this.source.length : newline;
    const body = this.source.slice(this.index, end);
    const doc = (body.startsWith("///") && !body.startsWith("////")) || body.startsWith("//!");
    const kind = doc ? "doc" : "line";
    const lines = [body.slice(doc ? 3 : 2)];
    this.comments.push({ kind, lines, line: this.line, start: this.index, end });
    if (doc) this.push("doc", end);
    else this.advanceTo(end);
  }

  private blockComment(): void {
    const { source, index: start, line } = this;
    let depth = 1;
    let at = start + 2;
    while (at < source.length && depth > 0) {
      const pair = source.slice(at, at + 2);
      if (pair === "/*") depth++;
      else if (pair === "*/") depth--;
      at += pair === "/*" || pair === "*/" ? 2 : 1;
    }
    const body = source.slice(start, at);
    const doc =
      (body.startsWith("/**") && !body.startsWith("/***") && body !== "/**/") ||
      body.startsWith("/*!");
    const inner = body.slice(doc ? 3 : 2, body.endsWith("*/") ? -2 : undefined);
    this.comments.push({
      kind: doc ? "doc" : "block",
      lines: inner.split("\n"),
      line,
      start,
      end: at,
    });
    if (doc) this.push("doc", at);
    else this.advanceTo(at);
  }

  /** Raw and prefixed literals: r"…", r#"…"#, b"…", br#"…"#, c"…", cr"…", b'…'. */
  private prefixedLiteral(): boolean {
    const { source, index } = this;
    const prefix = /^(?:br|cr|r|b|c)/u.exec(source.slice(index, index + 2))?.[0] ?? "";
    const after = index + prefix.length;
    if (prefix.endsWith("r") && (source[after] === "#" || source[after] === '"')) {
      const end = this.rawEnd(after);
      if (end > 0) this.push("literal", end);
      else if (prefix === "r" && isIdentStart(source[after + 1] ?? ""))
        this.push("ident", this.identEnd(after + 1));
      return end > 0 || this.index !== index;
    }
    if ((prefix === "b" || prefix === "c") && source[after] === '"')
      this.push("literal", this.quotedEnd(after + 1, '"'));
    else if (prefix === "b" && source[after] === "'")
      this.push("literal", this.quotedEnd(after + 1, "'"));
    return this.index !== index;
  }

  /** A character literal is `'x'` or an escape; anything else is a lifetime or label. */
  private quoteMark(next: string): boolean {
    const point = this.source.codePointAt(this.index + 1);
    const width = point !== undefined && point > lastBmpCodePoint ? 2 : 1;
    if (next === "\\" || (next !== "" && this.source[this.index + 1 + width] === "'"))
      this.push("literal", this.quotedEnd(this.index + 1, "'"));
    else if (isIdentStart(next)) this.push("lifetime", this.identEnd(this.index + 1));
    else return false;
    return true;
  }

  private numberEnd(): number {
    const { source, index } = this;
    let end = this.identEnd(index);
    if (/^0[xob]/iu.test(source.slice(index, index + 2))) return end;
    if (source[end] === "." && isDigit(source[end + 1] ?? "")) end = this.identEnd(end + 1);
    else if (source[end] === "." && source[end + 1] !== "." && !isIdentStart(source[end + 1] ?? ""))
      end += 1;
    if (/[eE]$/u.test(source.slice(index, end)) && /^[+-]\d/u.test(source.slice(end, end + 2)))
      end = this.identEnd(end + 1);
    return end;
  }

  private quotedEnd(start: number, quote: string): number {
    let at = start;
    while (at < this.source.length) {
      const char = this.source[at];
      if (char === "\\") at += 2;
      else if (char === quote) return this.literalEnd(at + 1);
      else at++;
    }
    return this.source.length;
  }

  /** `start` points at the first `#` or the opening quote; -1 when it is no raw string. */
  private rawEnd(start: number): number {
    let hashes = 0;
    while (this.source[start + hashes] === "#") hashes++;
    if (this.source[start + hashes] !== '"') return -1;
    const close = `"${"#".repeat(hashes)}`;
    const end = this.source.indexOf(close, start + hashes + 1);
    return end < 0 ? this.source.length : this.literalEnd(end + close.length);
  }

  private literalEnd(close: number): number {
    return isIdentStart(this.source[close] ?? "") ? this.identEnd(close) : close;
  }

  private identEnd(start: number): number {
    let at = start;
    while (at < this.source.length && isIdentContinue(this.source[at])) at++;
    return at;
  }
}

/** Lex one Rust source file. Unterminated literals run to the end of the file. */
export function lexRust(source: string): RustLexResult {
  return new RustLexer(source).run();
}

/** A stable spelling of one token for comparisons; positions are dropped. */
export function tokenKey(token: RustToken): string {
  if (token.kind === "punct") return `${token.text}${token.joint ? "+" : ""}`;
  return `${token.kind[0]}:${token.text}`;
}

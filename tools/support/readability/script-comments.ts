// Comment and literal masking for TypeScript, JavaScript and Vue `<script>`
// blocks, for line-based source checks that need no parser.
//
// `maskScript` returns the comments and a copy of the source in which every
// comment and the contents of every string, template and regular-expression
// literal are spaces. Newlines survive, so line numbers stay valid. Template
// substitutions (`${…}`) are code and stay visible. A `/` starts a regular
// expression when the previous significant token cannot end an expression;
// that is the usual heuristic and it is deterministic, which is what a
// ratchet needs.

export interface ScriptComment {
  /** `doc` is `/** … *\/`; `line` is `//`; `block` is any other `/* … *\/`. */
  readonly kind: "line" | "block" | "doc";
  readonly lines: readonly string[];
  readonly line: number;
}

export interface MaskedScript {
  readonly comments: readonly ScriptComment[];
  /** The source with comments and literal contents blanked. */
  readonly code: string;
}

const expressionKeywords = new Set([
  "return",
  "typeof",
  "instanceof",
  "case",
  "in",
  "of",
  "new",
  "delete",
  "void",
  "throw",
  "else",
  "do",
  "yield",
  "await",
]);

const isWordChar = (char: string) => /[\p{L}\p{N}_$]/u.test(char);

class ScriptMasker {
  readonly comments: ScriptComment[] = [];
  private readonly source: string;
  private readonly out: string[];
  private index = 0;
  private line = 1;
  /** `true` when the last significant token ends an expression, so `/` divides. */
  private afterExpression = false;
  /** Brace depth at each open template substitution. */
  private readonly templates: number[] = [];
  private braces = 0;

  constructor(source: string) {
    this.source = source;
    this.out = source.split("");
  }

  run(): MaskedScript {
    while (this.index < this.source.length) this.step();
    return { comments: this.comments, code: this.out.join("") };
  }

  private step(): void {
    const char = this.source[this.index];
    const next = this.source[this.index + 1] ?? "";
    if (char === "\n") {
      this.line++;
      this.index++;
    } else if (char === " " || char === "\t" || char === "\r") this.index++;
    else if (char === "/" && next === "/") this.lineComment();
    else if (char === "/" && next === "*") this.blockComment();
    else if (char === '"' || char === "'") this.string(char);
    else if (char === "`") this.template(this.index + 1);
    else if (char === "/" && !this.afterExpression && this.regex()) return;
    else if (char === "}" && this.templates.at(-1) === this.braces - 1) {
      this.templates.pop();
      this.braces--;
      this.template(this.index + 1);
    } else if (isWordChar(char)) this.word();
    else {
      if (char === "{") this.braces++;
      if (char === "}") this.braces--;
      this.afterExpression = char === ")" || char === "]" || char === "}";
      this.index++;
    }
  }

  private blank(start: number, end: number): void {
    for (let at = start; at < end; at++) if (this.out[at] !== "\n") this.out[at] = " ";
  }

  /** Blank `start..end` and move past it, counting the newlines it spans. */
  private consume(start: number, end: number, blankFrom = start, blankTo = end): void {
    this.blank(blankFrom, blankTo);
    for (let at = start; at < end; at++) if (this.source[at] === "\n") this.line++;
    this.index = end;
  }

  private lineComment(): void {
    const newline = this.source.indexOf("\n", this.index);
    const end = newline < 0 ? this.source.length : newline;
    this.comments.push({
      kind: "line",
      lines: [this.source.slice(this.index + 2, end)],
      line: this.line,
    });
    this.consume(this.index, end);
  }

  private blockComment(): void {
    const close = this.source.indexOf("*/", this.index + 2);
    const end = close < 0 ? this.source.length : close + 2;
    const body = this.source.slice(this.index, end);
    const doc = body.startsWith("/**") && body !== "/**/" && !body.startsWith("/***");
    const inner = body.slice(doc ? 3 : 2, body.endsWith("*/") ? -2 : undefined);
    this.comments.push({ kind: doc ? "doc" : "block", lines: inner.split("\n"), line: this.line });
    this.consume(this.index, end);
  }

  private string(quote: string): void {
    let at = this.index + 1;
    while (at < this.source.length) {
      const char = this.source[at];
      if (char === "\\") at += 2;
      else if (char === quote || char === "\n") break;
      else at++;
    }
    const end = Math.min(at + 1, this.source.length);
    this.consume(this.index, end, this.index + 1, end - 1);
    this.afterExpression = true;
  }

  /** Template text from `start` to the closing backtick or the next `${`. */
  private template(start: number): void {
    let at = start;
    let substitution = false;
    while (at < this.source.length) {
      const char = this.source[at];
      if (char === "\\") at += 2;
      else if (char === "`") break;
      else if (char === "$" && this.source[at + 1] === "{") {
        substitution = true;
        break;
      } else at++;
    }
    const end = Math.min(at + (substitution ? 2 : 1), this.source.length);
    this.consume(start, end, start, substitution ? end - 2 : end - 1);
    if (substitution) this.templates.push(this.braces++);
    this.afterExpression = !substitution;
  }

  private regex(): boolean {
    let at = this.index + 1;
    let inClass = false;
    while (at < this.source.length && this.source[at] !== "\n") {
      const char = this.source[at];
      if (char === "\\") {
        at += 2;
        continue;
      }
      if (char === "[") inClass = true;
      else if (char === "]") inClass = false;
      else if (char === "/" && !inClass) {
        at++;
        while (/[a-z]/iu.test(this.source[at] ?? "")) at++;
        this.consume(this.index, at, this.index + 1, at);
        this.afterExpression = true;
        return true;
      }
      at++;
    }
    return false;
  }

  private word(): void {
    let end = this.index + 1;
    while (end < this.source.length && isWordChar(this.source[end])) end++;
    this.afterExpression = !expressionKeywords.has(this.source.slice(this.index, end));
    this.index = end;
  }
}

/** Mask one script. */
export function maskScript(source: string): MaskedScript {
  return new ScriptMasker(source).run();
}

/** The `<script>` blocks of a Vue SFC, with everything else blanked to spaces. */
export function vueScriptSource(source: string): string {
  const out: string[] = [];
  let inside = false;
  for (const text of source.split("\n")) {
    if (!inside && /^\s*<script\b/u.test(text)) {
      inside = !/<\/script>/u.test(text);
      out.push("");
      continue;
    }
    if (inside && /^\s*<\/script>/u.test(text)) {
      inside = false;
      out.push("");
      continue;
    }
    out.push(inside ? text : "");
  }
  return out.join("\n");
}
